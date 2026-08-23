use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;

/// Something that can deliver a text message. Exists so tests can substitute
/// a recording fake instead of hitting Telegram.
#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send(&self, message: &str) -> anyhow::Result<()>;
}

/// Sends messages via the Telegram Bot API's `sendMessage` endpoint. This is
/// the only endpoint this project uses, so a small direct HTTP call is
/// simpler than depending on a full bot framework.
pub struct Telegram {
    client: reqwest::Client,
    token: String,
    chat_id: String,
}

impl Telegram {
    pub fn new(token: String, chat_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
            chat_id,
        }
    }
}

#[async_trait]
impl Notifier for Telegram {
    async fn send(&self, message: &str) -> anyhow::Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let response = self
            .client
            .post(&url)
            .json(&json!({ "chat_id": self.chat_id, "text": message }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("telegram sendMessage failed ({status}): {body}");
        }

        // Stay well under Telegram's per-chat rate limit for bursts of posts.
        tokio::time::sleep(Duration::from_millis(250)).await;
        Ok(())
    }
}
