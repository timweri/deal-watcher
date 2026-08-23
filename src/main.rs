mod cache;
mod config;
mod health;
mod heartbeat;
mod http;
mod notify;
mod reddit;
mod rfd;
mod watch;

use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

use cache::SeenCache;
use config::Config;
use health::JobRegistry;
use notify::{Notifier, Telegram};

// Single source of truth for how often each job runs: used both to build
// the schedule below and to compute /healthz staleness thresholds.
//
// Expressions are 6-field (seconds first, tokio-cron-scheduler's format) and
// evaluated in UTC. That's fine for wall-clock alignment (":00", ":15", ...)
// as long as the deployment's timezone has a whole-hour UTC offset, which
// this project's does.
const FETCH_REDDIT_CRON: &str = "0 0/15 * * * *";
const FETCH_RFD_CRON: &str = "0 0/10 * * * *";
const HEARTBEAT_CRON: &str = "0 0 * * * *";

const FETCH_REDDIT_INTERVAL: Duration = Duration::from_secs(15 * 60);
const FETCH_RFD_INTERVAL: Duration = Duration::from_secs(10 * 60);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60 * 60);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cfg = Arc::new(Config::load()?);
    let http_client = http::build_client();
    let notifier: Arc<dyn Notifier> = Arc::new(Telegram::new(
        cfg.telegram_access_token.clone(),
        cfg.telegram_chat_id.clone(),
    ));

    let registry = JobRegistry::new(&[
        ("fetch_reddit", FETCH_REDDIT_INTERVAL),
        ("fetch_rfd", FETCH_RFD_INTERVAL),
        ("heartbeat", HEARTBEAT_INTERVAL),
    ]);

    let reddit_cache = Arc::new(Mutex::new(SeenCache::load(&cfg.data, "cache.json")));
    let rfd_cache = Arc::new(Mutex::new(SeenCache::load(&cfg.data, "cache-rfd.json")));

    let scheduler = JobScheduler::new().await?;

    scheduler
        .add(Job::new_async(FETCH_REDDIT_CRON, {
            let registry = registry.clone();
            let cache = reddit_cache.clone();
            let notifier = notifier.clone();
            let client = http_client.clone();
            let time_window = cfg.time_window;
            move |_uuid, _lock| {
                let registry = registry.clone();
                let cache = cache.clone();
                let notifier = notifier.clone();
                let client = client.clone();
                Box::pin(async move {
                    watch::run_cycle(
                        "fetch_reddit",
                        &registry,
                        &cache,
                        &notifier,
                        time_window,
                        || reddit::fetch(&client),
                    )
                    .await;
                })
            }
        })?)
        .await?;

    scheduler
        .add(Job::new_async(FETCH_RFD_CRON, {
            let registry = registry.clone();
            let cache = rfd_cache.clone();
            let notifier = notifier.clone();
            let client = http_client.clone();
            let time_window = cfg.time_window;
            move |_uuid, _lock| {
                let registry = registry.clone();
                let cache = cache.clone();
                let notifier = notifier.clone();
                let client = client.clone();
                Box::pin(async move {
                    watch::run_cycle(
                        "fetch_rfd",
                        &registry,
                        &cache,
                        &notifier,
                        time_window,
                        || rfd::fetch(&client),
                    )
                    .await;
                })
            }
        })?)
        .await?;

    scheduler
        .add(Job::new_async(HEARTBEAT_CRON, {
            let registry = registry.clone();
            let notifier = notifier.clone();
            move |_uuid, _lock| {
                let registry = registry.clone();
                let notifier = notifier.clone();
                Box::pin(async move {
                    heartbeat::run(&registry, &notifier).await;
                })
            }
        })?)
        .await?;

    scheduler.start().await?;

    let router = health::router(registry);
    let listener = TcpListener::bind(("0.0.0.0", cfg.healthz_port)).await?;
    info!(port = cfg.healthz_port, "serving /healthz");
    axum::serve(listener, router).await?;

    Ok(())
}
