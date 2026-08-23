use std::time::Duration;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{
    RetryTransientMiddleware, Retryable, RetryableStrategy, default_on_request_failure,
    default_on_request_success, policies::ExponentialBackoff,
};

/// Arctic Shift (the Reddit mirror this project depends on) returns 422 under
/// load for otherwise-valid requests. The default retry strategy treats any
/// 4xx as fatal, so we extend it to additionally retry 422.
struct ArcticShiftRetryStrategy;

impl RetryableStrategy for ArcticShiftRetryStrategy {
    fn handle(
        &self,
        res: &Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<Retryable> {
        match res {
            Ok(response) if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
                Some(Retryable::Transient)
            }
            Ok(response) => default_on_request_success(response),
            Err(error) => default_on_request_failure(error),
        }
    }
}

/// Build the shared HTTP client used for both Arctic Shift and RedFlagDeals
/// requests: 3 total attempts (2 retries) with exponential backoff and
/// jitter on transient failures (5xx, 408, 429, and Arctic Shift's 422).
pub fn build_client() -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(Duration::from_secs(1), Duration::from_secs(30))
        .jitter(reqwest_retry::Jitter::Bounded)
        .base(2)
        .build_with_max_retries(2);

    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy_and_strategy(
            retry_policy,
            ArcticShiftRetryStrategy,
        ))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn retries_422_then_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(ResponseTemplate::new(422))
            .up_to_n_times(2)
            .expect(2)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let client = build_client();
        let response = client
            .get(format!("{}/x", server.uri()))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn gives_up_after_repeated_server_errors() {
        let server = MockServer::start().await;
        // 2 retries configured => 3 total attempts.
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(ResponseTemplate::new(500))
            .expect(3)
            .mount(&server)
            .await;

        let client = build_client();
        let response = client
            .get(format!("{}/x", server.uri()))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 500);
    }

    #[tokio::test]
    async fn does_not_retry_a_fatal_client_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/x"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;

        let client = build_client();
        let response = client
            .get(format!("{}/x", server.uri()))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 404);
    }
}
