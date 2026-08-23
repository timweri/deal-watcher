use chrono::DateTime;
use regex::Regex;
use reqwest_middleware::ClientWithMiddleware;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use tracing::error;

use crate::watch::Item;

const FORUM_URL: &str = "https://forums.redflagdeals.com/hot-deals-f9/?rfd_sk=tt&sd=d&sk=tt";
const POW_COOKIE_NAME: &str = "pow_bypass";
// Matches the site's own client-side bound on how many nonces it will try.
const POW_MAX_ITERS: u64 = 10_000_000;

// A handful of real, current browser UA strings. RFD only needs a plausible
// one; there's no Rust equivalent of Python's fake-useragent package.
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
];

fn random_user_agent() -> &'static str {
    USER_AGENTS[rand::random_range(0..USER_AGENTS.len())]
}

fn form_full_rfd_url(relative_path: &str) -> String {
    format!("https://forums.redflagdeals.com{relative_path}")
}

fn parse_pow_field(html: &str, key: &str) -> Option<String> {
    let pattern = format!(r"{key}:\s*'([^']*)'");
    let re = Regex::new(&pattern).expect("static regex is valid");
    re.captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Solve an RFD proof-of-work challenge page, returning the `pow_bypass`
/// cookie value to send on the retry, or `None` if the page isn't a
/// challenge or is missing a required field.
fn solve_pow_challenge(html: &str) -> Option<String> {
    if !html.contains("POW_CHALLENGE_DATA") {
        return None;
    }

    let nonce = parse_pow_field(html, "challenge_nonce")?;
    let hmac = parse_pow_field(html, "challenge_hmac")?;
    let difficulty: usize = parse_pow_field(html, "difficulty")?.parse().ok()?;
    let dchar = parse_pow_field(html, "difficulty_char")?;
    let issued_at = parse_pow_field(html, "issued_at")?;

    let target = dchar.repeat(difficulty);
    let prefix = format!("{nonce}{issued_at}");

    for i in 1..POW_MAX_ITERS {
        let digest = hex::encode(Sha256::digest(format!("{prefix}{i}").as_bytes()));
        if digest.starts_with(&target) {
            return Some(format!("{nonce}|{issued_at}|{i}|{digest}|{hmac}"));
        }
    }
    None
}

/// Fetch a forum page, transparently clearing RFD's PoW anti-bot wall if the
/// first response is a challenge page.
async fn fetch_forum(
    client: &ClientWithMiddleware,
    url: &str,
    user_agent: &str,
) -> anyhow::Result<String> {
    let text = client
        .get(url)
        .header("User-Agent", user_agent)
        .send()
        .await?
        .text()
        .await?;

    let Some(cookie) = solve_pow_challenge(&text) else {
        return Ok(text);
    };

    let text = client
        .get(url)
        .header("User-Agent", user_agent)
        .header("Cookie", format!("{POW_COOKIE_NAME}={cookie}"))
        .send()
        .await?
        .text()
        .await?;
    Ok(text)
}

fn selector(css: &str) -> Selector {
    Selector::parse(css).expect("static selector is valid")
}

/// Parse a fetched hot-deals page into new-thread items. Sticky threads,
/// sponsored/advertorial placements, and threads whose title contains
/// "Merged" are skipped, matching the site's own convention for
/// merged/duplicate threads.
fn parse_forum_page(html: &str) -> Vec<Item> {
    let doc = Html::parse_document(html);
    let list_sel = selector("ul.topics-cards.topics.with_categories");
    let card_sel = selector("li.topic-card");
    let sticky_sel = selector(".sticky");
    // Sponsored campaigns are re-posted as brand new thread ids each time,
    // so the seen-cache can never suppress them on its own — drop them here.
    let sponsored_sel = selector(".sponsored-offer, .sponsored-badge");
    let time_sel = selector("time");
    let link_sel = selector("a.topic-card-info.thread_info");
    let title_sel = selector("h3.thread_title");

    let Some(list) = doc.select(&list_sel).next() else {
        error!("RFD: could not find topics list container");
        return Vec::new();
    };

    let mut items = Vec::new();

    for card in list.select(&card_sel) {
        if card.select(&sticky_sel).next().is_some()
            || card.select(&sponsored_sel).next().is_some()
        {
            continue;
        }

        let Some(thread_id) = card.value().attr("data-thread-id") else {
            continue;
        };

        let Some(time_tag) = card.select(&time_sel).next() else {
            continue;
        };
        let Some(datetime_str) = time_tag.value().attr("datetime") else {
            continue;
        };
        let Ok(post_time) = DateTime::parse_from_rfc3339(datetime_str) else {
            error!(thread_id, datetime_str, "RFD: unparseable datetime");
            continue;
        };

        let Some(link_tag) = card.select(&link_sel).next() else {
            continue;
        };
        let Some(href) = link_tag.value().attr("href") else {
            continue;
        };
        let link = form_full_rfd_url(href);

        let Some(title_tag) = card.select(&title_sel).next() else {
            continue;
        };
        let title = title_tag
            .text()
            .collect::<String>()
            .trim()
            .replace('\n', "");

        if title.contains("Merged") {
            continue;
        }

        let created_at = post_time.timestamp();
        let time_str = post_time
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S");
        let message = format!("{time_str}: {title}\n\n{link}");

        items.push(Item {
            id: thread_id.to_string(),
            created_at,
            message,
        });
    }

    items
}

pub async fn fetch(client: &ClientWithMiddleware) -> anyhow::Result<Vec<Item>> {
    let user_agent = random_user_agent();
    let html = fetch_forum(client, FORUM_URL, user_agent).await?;
    Ok(parse_forum_page(&html))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture(name: &str) -> String {
        fs::read_to_string(format!(
            "{}/tests/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap_or_else(|_| panic!("missing fixture {name}"))
    }

    #[test]
    fn solves_low_difficulty_challenge() {
        let html = fixture("rfd_challenge.html");
        let cookie = solve_pow_challenge(&html).expect("should solve challenge");
        let mut parts = cookie.split('|');
        let nonce = parts.next().unwrap();
        let issued_at = parts.next().unwrap();
        let i: u64 = parts.next().unwrap().parse().unwrap();
        let digest = parts.next().unwrap();

        let recomputed = hex::encode(Sha256::digest(format!("{nonce}{issued_at}{i}").as_bytes()));
        assert_eq!(recomputed, digest);
        assert!(digest.starts_with('0')); // fixture uses difficulty_char '0', difficulty 1
    }

    #[test]
    fn non_challenge_page_returns_none() {
        assert_eq!(solve_pow_challenge("<html>hi</html>"), None);
    }

    #[test]
    fn parses_threads_skipping_sticky_merged_and_sponsored() {
        let html = fixture("rfd_hot_deals.html");
        let items = parse_forum_page(&html);

        assert!(items.iter().any(|i| i.id == "thread-1"));
        assert!(!items.iter().any(|i| i.id == "thread-sticky"));
        assert!(!items.iter().any(|i| i.id == "thread-merged"));
        assert!(!items.iter().any(|i| i.id == "thread-sponsored"));
    }
}
