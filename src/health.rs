use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

// A job is flagged stale if it hasn't run within this many multiples of its
// own interval...
const STALE_MULTIPLIER: u32 = 2;
// ...plus a flat grace window to absorb scheduler jitter.
const GRACE: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone)]
struct RunOutcome {
    at: Instant,
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct JobDescriptor {
    interval: Duration,
}

/// Tracks the most recent outcome of each named job, in memory. Replaces the
/// per-job `status-*.json` files the Python version used for cross-process
/// IPC — with one process, a shared map is simpler.
#[derive(Clone)]
pub struct JobRegistry {
    started_at: Instant,
    jobs: HashMap<&'static str, JobDescriptor>,
    outcomes: Arc<RwLock<HashMap<&'static str, RunOutcome>>>,
}

impl JobRegistry {
    pub fn new(jobs: &[(&'static str, Duration)]) -> Self {
        Self {
            started_at: Instant::now(),
            jobs: jobs
                .iter()
                .map(|(name, interval)| {
                    (
                        *name,
                        JobDescriptor {
                            interval: *interval,
                        },
                    )
                })
                .collect(),
            outcomes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record the outcome of a job's most recent run. `error` is `None` on
    /// success, or a description of what went wrong on failure.
    pub fn record(&self, job_name: &'static str, ok: bool, error: Option<String>) {
        let mut outcomes = self.outcomes.write().expect("registry lock poisoned");
        outcomes.insert(
            job_name,
            RunOutcome {
                at: Instant::now(),
                ok,
                error,
            },
        );
    }

    fn threshold(&self, interval: Duration) -> Duration {
        interval * STALE_MULTIPLIER + GRACE
    }

    fn job_report(&self, name: &str, descriptor: &JobDescriptor) -> JobReport {
        let threshold = self.threshold(descriptor.interval);
        let outcomes = self.outcomes.read().expect("registry lock poisoned");

        match outcomes.get(name) {
            None => {
                if self.started_at.elapsed() < threshold {
                    JobReport {
                        ok: true,
                        reason: "pending first run".into(),
                        last_run_secs_ago: None,
                        error: None,
                    }
                } else {
                    JobReport {
                        ok: false,
                        reason: "no run recorded yet".into(),
                        last_run_secs_ago: None,
                        error: None,
                    }
                }
            }
            Some(outcome) if !outcome.ok => JobReport {
                ok: false,
                reason: "last run failed".into(),
                last_run_secs_ago: Some(outcome.at.elapsed().as_secs()),
                error: outcome.error.clone(),
            },
            Some(outcome) => {
                let age = outcome.at.elapsed();
                if age > threshold {
                    JobReport {
                        ok: false,
                        reason: format!(
                            "stale ({}s since last run, expected within {}s)",
                            age.as_secs(),
                            threshold.as_secs()
                        ),
                        last_run_secs_ago: Some(age.as_secs()),
                        error: None,
                    }
                } else {
                    JobReport {
                        ok: true,
                        reason: "ok".into(),
                        last_run_secs_ago: Some(age.as_secs()),
                        error: None,
                    }
                }
            }
        }
    }

    fn build_report(&self) -> (bool, HealthReport) {
        let jobs: HashMap<String, JobReport> = self
            .jobs
            .iter()
            .map(|(name, descriptor)| (name.to_string(), self.job_report(name, descriptor)))
            .collect();
        let healthy = jobs.values().all(|job| job.ok);
        (
            healthy,
            HealthReport {
                status: if healthy { "ok" } else { "error" },
                jobs,
            },
        )
    }
}

#[derive(Debug, Serialize)]
struct JobReport {
    ok: bool,
    reason: String,
    last_run_secs_ago: Option<u64>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthReport {
    status: &'static str,
    jobs: HashMap<String, JobReport>,
}

async fn healthz(State(registry): State<JobRegistry>) -> impl IntoResponse {
    let (healthy, report) = registry.build_report();
    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(report))
}

pub fn router(registry: JobRegistry) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(registry)
}
