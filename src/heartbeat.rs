use std::sync::Arc;

use chrono::Local;

use crate::health::JobRegistry;
use crate::notify::Notifier;

/// Send the current local time to Telegram, so a healthy container is
/// noticeably still alive even when there's nothing to report.
pub async fn run(registry: &JobRegistry, notifier: &Arc<dyn Notifier>) {
    let time_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    match notifier.send(&time_str).await {
        Ok(()) => registry.record("heartbeat", true, None),
        Err(err) => registry.record("heartbeat", false, Some(err.to_string())),
    }
}
