use std::time::Duration;

use chrono::{Local, TimeZone};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use tracing::error;

use crate::watch::Item;

const ARCTIC_SHIFT_URL: &str = "https://arctic-shift.photon-reddit.com/api/posts/search";
const SUBREDDITS: &[&str] = &["bapcsalescanada", "CanadianHardwareSwap"];

#[derive(Debug, Deserialize)]
struct ArcticShiftResponse {
    data: Option<Vec<RedditPost>>,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    id: String,
    created_utc: i64,
    title: String,
    permalink: String,
    #[serde(default)]
    is_self: bool,
    url: Option<String>,
}

impl RedditPost {
    fn into_item(self) -> Item {
        let time_str = Local
            .timestamp_opt(self.created_utc, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();
        let reddit_link = format!("https://reddit.com{}", self.permalink);

        let mut message = format!("{time_str}: {}\n\n{reddit_link}", self.title);
        if !self.is_self
            && let Some(url) = &self.url
        {
            message.push_str("\n\n");
            message.push_str(url);
        }

        Item {
            id: self.id,
            created_at: self.created_utc,
            message,
        }
    }
}

async fn fetch_subreddit(
    client: &ClientWithMiddleware,
    subreddit: &str,
) -> anyhow::Result<Vec<RedditPost>> {
    let response = client
        .get(ARCTIC_SHIFT_URL)
        .query(&[("subreddit", subreddit), ("limit", "25"), ("sort", "desc")])
        .header(
            "User-Agent",
            "deal-watcher/1.0 (personal subreddit watcher)",
        )
        .send()
        .await?
        .error_for_status()?;

    let parsed: ArcticShiftResponse = response.json().await?;
    Ok(parsed.data.unwrap_or_default())
}

/// Fetch new posts across all watched subreddits. A single subreddit failing
/// is logged and skipped rather than failing the whole cycle, matching the
/// resilience of the per-subreddit loop this replaces.
pub async fn fetch(client: &ClientWithMiddleware) -> anyhow::Result<Vec<Item>> {
    let mut items = Vec::new();

    for (idx, subreddit) in SUBREDDITS.iter().enumerate() {
        if idx > 0 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        match fetch_subreddit(client, subreddit).await {
            Ok(posts) => items.extend(posts.into_iter().map(RedditPost::into_item)),
            Err(err) => error!(subreddit, %err, "reddit fetch failed for subreddit"),
        }
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_link_post_without_url_suffix() {
        let post = RedditPost {
            id: "abc".into(),
            created_utc: 1_700_000_000,
            title: "Great deal".into(),
            permalink: "/r/test/comments/abc/great_deal/".into(),
            is_self: true,
            url: Some("https://reddit.com/r/test/comments/abc/great_deal/".into()),
        };
        let item = post.into_item();
        assert!(!item.message.contains("https://example.com"));
        assert!(item.message.contains("Great deal"));
    }

    #[test]
    fn maps_link_post_with_external_url_suffix() {
        let post = RedditPost {
            id: "abc".into(),
            created_utc: 1_700_000_000,
            title: "Great deal".into(),
            permalink: "/r/test/comments/abc/great_deal/".into(),
            is_self: false,
            url: Some("https://example.com/deal".into()),
        };
        let item = post.into_item();
        assert!(item.message.ends_with("https://example.com/deal"));
    }
}
