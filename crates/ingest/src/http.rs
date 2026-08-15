//! Shared HTTP fetching with retry, backoff, jitter and conditional
//! requests (AGENTS.md §11).
//!
//! All connectors use this single fetcher; transport policy (timeout,
//! retries, rate-limit detection, ETag caching) lives here, not per source.

use crate::connector::ConnectorError;
use ferris_core::Source;
use rand::Rng;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tracing::{debug, warn};

const MAX_ATTEMPTS: u32 = 3;
const BASE_RETRY_MS: u64 = 500;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// A retrying HTTP client shared by all connectors.
#[derive(Debug)]
pub struct HttpFetcher {
    client: reqwest::Client,
    /// Optional `Authorization: Bearer` header (e.g. a GitHub token).
    bearer_token: Option<String>,
    /// ETag cache: url -> etag, enabling If-None-Match / 304 (conditional
    /// requests) for feeds and API responses that support it.
    etags: Mutex<HashMap<String, String>>,
}

impl HttpFetcher {
    /// Create a fetcher. `user_agent` identifies this application to
    /// sources; sources expect a descriptive UA.
    pub fn new(user_agent: &str) -> Result<Self, reqwest::Error> {
        Self::with_bearer_token(user_agent, None)
    }

    /// Create a fetcher that sends an `Authorization: Bearer <token>` header
    /// (used for GitHub's authenticated rate limit).
    pub fn with_bearer_token(
        user_agent: &str,
        bearer_token: Option<String>,
    ) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(user_agent)
            .build()?;
        Ok(Self {
            client,
            bearer_token,
            etags: Mutex::new(HashMap::new()),
        })
    }

    /// GET `url` as text with retries and conditional requests.
    ///
    /// Returns the cached body when the source answers 304.
    pub async fn get_text(&self, source: Source, url: &str) -> Result<String, ConnectorError> {
        self.get(source, url).await
    }

    /// GET `url` and deserialize the JSON body.
    pub async fn get_json<T: DeserializeOwned>(
        &self,
        source: Source,
        url: &str,
    ) -> Result<T, ConnectorError> {
        let body = self.get(source, url).await?;
        serde_json::from_str(&body).map_err(|e| ConnectorError::Parse {
            source_name: source,
            message: format!("invalid JSON from {url}: {e}"),
        })
    }

    async fn get(&self, source: Source, url: &str) -> Result<String, ConnectorError> {
        let mut attempt: u32 = 0;
        loop {
            attempt += 1;
            match self.try_get(source, url).await {
                Ok(body) => return Ok(body),
                Err(err) => {
                    let retryable = matches!(
                        err,
                        ConnectorError::Http { .. } | ConnectorError::RateLimited { .. }
                    );
                    if !retryable || attempt >= MAX_ATTEMPTS {
                        return Err(err);
                    }
                    let delay = self.backoff_delay(attempt);
                    warn!(url, attempt, delay_ms = delay.as_millis(), error = %err, "retrying fetch");
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn try_get(&self, source: Source, url: &str) -> Result<String, ConnectorError> {
        let etag = self.etags.lock().unwrap().get(url).cloned();
        let mut request = self.client.get(url);
        if let Some(token) = &self.bearer_token {
            request = request.bearer_auth(token);
        }
        if let Some(etag) = &etag {
            request = request.header(reqwest::header::IF_NONE_MATCH, etag);
        }

        let response = request.send().await.map_err(|e| ConnectorError::Http {
            source_name: source,
            message: e.to_string(),
        })?;

        let status = response.status();
        match status {
            reqwest::StatusCode::NOT_MODIFIED => {
                // Conditional request hit: cached body is still valid.
                return self.etags.lock().unwrap().get(url).cloned().ok_or_else(|| {
                    ConnectorError::Http {
                        source_name: source,
                        message: "304 without cached body".into(),
                    }
                });
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS | reqwest::StatusCode::FORBIDDEN => {
                let retry_after = response
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(Duration::from_secs);
                return Err(ConnectorError::RateLimited {
                    source_name: source,
                    retry_after,
                });
            }
            _ if !status.is_success() => {
                return Err(ConnectorError::Http {
                    source_name: source,
                    message: format!("unexpected status {status}"),
                });
            }
            _ => {}
        }

        if let Some(new_etag) = response.headers().get(reqwest::header::ETAG) {
            if let Ok(new_etag) = new_etag.to_str() {
                self.etags
                    .lock()
                    .unwrap()
                    .insert(url.to_string(), new_etag.to_string());
                debug!(url, "cached etag");
            }
        }

        response.text().await.map_err(|e| ConnectorError::Http {
            source_name: source,
            message: format!("reading body: {e}"),
        })
    }

    /// Exponential backoff with jitter: `500ms * 2^(attempt-1) ± 25%`.
    fn backoff_delay(&self, attempt: u32) -> Duration {
        let base = BASE_RETRY_MS.saturating_mul(1u64 << (attempt - 1).min(4));
        let jitter = rand::thread_rng().gen_range(0.75..=1.25);
        Duration::from_millis((base as f64 * jitter) as u64)
    }
}
