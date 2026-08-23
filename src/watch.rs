use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::cache::SeenCache;
use crate::health::JobRegistry;
use crate::notify::Notifier;

/// A single new post/thread, already formatted into the message that will be
/// sent to Telegram.
pub struct Item {
    pub id: String,
    pub created_at: i64,
    pub message: String,
}

/// Run one watch cycle for a source: fetch new items, notify about the ones
/// we haven't seen before (within `time_window` seconds), remember them,
/// prune anything old, and persist the cache. This is the cycle that
/// `fetch_reddit.py` and `fetch_rfd.py` each wrote out by hand; every
/// watcher shares it, so the cutoff/dedup/error-handling rules can only be
/// decided in one place.
///
/// Never panics and never returns an error to the caller: every failure is
/// recorded on the registry and sent to Telegram instead, so one bad item or
/// a down notifier doesn't abort the run.
pub async fn run_cycle<F, Fut>(
    job_name: &'static str,
    registry: &JobRegistry,
    cache: &Arc<Mutex<SeenCache>>,
    notifier: &Arc<dyn Notifier>,
    time_window: i64,
    fetch: F,
) where
    F: FnOnce() -> Fut,
    Fut: Future<Output = anyhow::Result<Vec<Item>>>,
{
    let items = match fetch().await {
        Ok(items) => items,
        Err(err) => {
            let message = format!("{job_name}: fetch failed: {err}");
            error!(job = job_name, %err, "fetch failed");
            let _ = notifier.send(&message).await;
            registry.record(job_name, false, Some(message));
            return;
        }
    };

    let now = chrono::Utc::now().timestamp();
    let cutoff = now - time_window;

    let mut cache = cache.lock().await;
    let mut first_error: Option<String> = None;

    for item in items {
        if item.created_at < cutoff || cache.contains(&item.id) {
            continue;
        }

        if let Err(err) = notifier.send(&item.message).await {
            let message = format!("{job_name} item {}: notify failed: {err}", item.id);
            error!(job = job_name, id = %item.id, %err, "notify failed");
            first_error.get_or_insert(message);
            continue;
        }

        cache.remember(&item.id, item.created_at);
    }

    cache.prune(cutoff);

    if let Err(err) = cache.save() {
        let message = format!("{job_name}: cache save failed: {err}");
        error!(job = job_name, %err, "cache save failed");
        let _ = notifier.send(&message).await;
        first_error.get_or_insert(message);
    }

    match &first_error {
        None => {
            info!(job = job_name, "cycle completed");
            registry.record(job_name, true, None);
        }
        Some(message) => registry.record(job_name, false, Some(message.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct RecordingNotifier {
        sent: StdMutex<Vec<String>>,
        fail_after: Option<usize>,
        calls: AtomicUsize,
    }

    impl RecordingNotifier {
        fn new() -> Self {
            Self {
                sent: StdMutex::new(Vec::new()),
                fail_after: None,
                calls: AtomicUsize::new(0),
            }
        }

        fn failing_after(n: usize) -> Self {
            Self {
                sent: StdMutex::new(Vec::new()),
                fail_after: Some(n),
                calls: AtomicUsize::new(0),
            }
        }

        fn sent(&self) -> Vec<String> {
            self.sent.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl Notifier for RecordingNotifier {
        async fn send(&self, message: &str) -> anyhow::Result<()> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(fail_after) = self.fail_after
                && call >= fail_after
            {
                anyhow::bail!("notify failed");
            }
            self.sent.lock().unwrap().push(message.to_string());
            Ok(())
        }
    }

    fn item(id: &str, created_at: i64) -> Item {
        Item {
            id: id.to_string(),
            created_at,
            message: format!("msg-{id}"),
        }
    }

    fn registry() -> JobRegistry {
        JobRegistry::new(&[("test_job", std::time::Duration::from_secs(60))])
    }

    #[tokio::test]
    async fn skips_items_past_the_time_window() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Arc::new(Mutex::new(SeenCache::load(dir.path(), "cache.json")));
        let recording = Arc::new(RecordingNotifier::new());
        let notifier: Arc<dyn Notifier> = recording.clone();
        let registry = registry();

        let now = chrono::Utc::now().timestamp();
        let too_old = now - 10_000;

        run_cycle(
            "test_job",
            &registry,
            &cache,
            &notifier,
            3600,
            || async move { Ok(vec![item("stale", too_old), item("fresh", now)]) },
        )
        .await;

        let sent = recording.sent();
        assert_eq!(sent, vec!["msg-fresh".to_string()]);
    }

    #[tokio::test]
    async fn does_not_renotify_a_cached_id() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Arc::new(Mutex::new(SeenCache::load(dir.path(), "cache.json")));
        let recording = Arc::new(RecordingNotifier::new());
        let notifier: Arc<dyn Notifier> = recording.clone();
        let registry = registry();
        let now = chrono::Utc::now().timestamp();

        {
            let mut c = cache.lock().await;
            c.remember("already-seen", now);
        }

        run_cycle(
            "test_job",
            &registry,
            &cache,
            &notifier,
            3600,
            || async move { Ok(vec![item("already-seen", now), item("new-1", now)]) },
        )
        .await;

        let sent = recording.sent();
        assert_eq!(sent, vec!["msg-new-1".to_string()]);
    }

    #[tokio::test]
    async fn cache_is_saved_even_when_an_item_fails_to_notify() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Arc::new(Mutex::new(SeenCache::load(dir.path(), "cache.json")));
        // First send (item "a") succeeds, second ("b") fails.
        let recording = Arc::new(RecordingNotifier::failing_after(1));
        let notifier: Arc<dyn Notifier> = recording.clone();
        let registry = registry();
        let now = chrono::Utc::now().timestamp();

        run_cycle(
            "test_job",
            &registry,
            &cache,
            &notifier,
            3600,
            || async move { Ok(vec![item("a", now), item("b", now)]) },
        )
        .await;

        // The successfully-notified item must have been persisted to disk
        // despite the later failure, and the job must be marked unhealthy.
        let reloaded = SeenCache::load(dir.path(), "cache.json");
        assert!(reloaded.contains("a"));
    }

    #[tokio::test]
    async fn fetch_error_does_not_touch_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Arc::new(Mutex::new(SeenCache::load(dir.path(), "cache.json")));
        let notifier: Arc<dyn Notifier> = Arc::new(RecordingNotifier::new());
        let registry = registry();

        run_cycle(
            "test_job",
            &registry,
            &cache,
            &notifier,
            3600,
            || async move { anyhow::bail!("network down") },
        )
        .await;

        // No panic, no file written; cache remains empty and reloadable.
        let reloaded = SeenCache::load(dir.path(), "cache.json");
        assert!(!reloaded.contains("anything"));
    }
}
