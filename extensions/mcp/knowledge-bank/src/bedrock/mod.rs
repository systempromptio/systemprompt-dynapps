//! Bedrock runtime client: bearer auth, region guard, retry, and concurrency.
//!
//! Authentication is a Bedrock long-term API key sent as
//! `Authorization: Bearer <key>`, not sig-v4 — so this is a plain `reqwest`
//! client and the crate needs no `aws-sdk-*` dependency.
//!
//! These calls do NOT pass through the AI gateway, so the `governance:`
//! metadata and route `requires:` mechanism that enforces REQ-037/038 never
//! sees them. [`assert_eu_region`] is therefore the only residency control on
//! this path, and it runs at construction so a non-EU region cannot be
//! configured silently.

mod embed;
mod rerank;

pub use embed::embed_text;
pub use rerank::rerank;

use crate::config::BedrockConfig;
use crate::error::KnowledgeBankError;
// JSON: protocol boundary. Bedrock's request and response bodies differ per
// model family — Titan, Cohere, and the Converse API share no shape — so the
// generic transport carries them untyped and each caller module parses its
// own model's response into a real type.
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

const MAX_ATTEMPTS: u32 = 10;
const BASE_BACKOFF_MS: u64 = 200;
const MAX_BACKOFF_MS: u64 = 20_000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// An authenticated Bedrock runtime client for one region.
pub struct BedrockClient {
    http: reqwest::Client,
    api_key: String,
    endpoint: String,
    pub config: BedrockConfig,
    calls: Arc<Semaphore>,
    embeds: Arc<Semaphore>,
}

impl std::fmt::Debug for BedrockClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BedrockClient")
            .field("endpoint", &self.endpoint)
            .finish_non_exhaustive()
    }
}

pub fn assert_eu_region(region: &str) -> Result<(), KnowledgeBankError> {
    if region.starts_with("eu-") {
        return Ok(());
    }
    Err(KnowledgeBankError::Internal(format!(
        "refusing to start: bedrock.region '{region}' is outside the EU. Indexed content \
         is Astound customer data and REQ-037 data-residency routing does not cover this \
         path, because knowledge-bank calls Bedrock directly rather than through the AI \
         gateway. Use eu-central-1 — it is the only EU region carrying a rerank model."
    )))
}

impl BedrockClient {
    pub fn new(config: BedrockConfig, api_key: String) -> Result<Self, KnowledgeBankError> {
        assert_eu_region(&config.region)?;
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()?;
        let endpoint = format!("https://bedrock-runtime.{}.amazonaws.com", config.region);
        let calls = Arc::new(Semaphore::new(config.max_concurrency));
        let embeds = Arc::new(Semaphore::new(config.embed_concurrency));
        Ok(Self {
            http,
            api_key,
            endpoint,
            config,
            calls,
            embeds,
        })
    }

    pub async fn invoke_model(
        &self,
        model_id: &str,
        body: &Value,
    ) -> Result<Value, KnowledgeBankError> {
        let url = format!(
            "{}/model/{}/invoke",
            self.endpoint,
            urlencoding::encode(model_id)
        );
        self.send_with_retry(&url, body).await
    }

    pub async fn converse(
        &self,
        model_id: &str,
        body: &Value,
    ) -> Result<Value, KnowledgeBankError> {
        let url = format!(
            "{}/model/{}/converse",
            self.endpoint,
            urlencoding::encode(model_id)
        );
        self.send_with_retry(&url, body).await
    }

    pub(crate) const fn embed_permits(&self) -> &Arc<Semaphore> {
        &self.embeds
    }

    async fn send_with_retry(&self, url: &str, body: &Value) -> Result<Value, KnowledgeBankError> {
        let mut last = String::new();
        for attempt in 0..MAX_ATTEMPTS {
            if attempt > 0 {
                tokio::time::sleep(backoff(attempt)).await;
            }
            let _permit = self
                .calls
                .acquire()
                .await
                // Why: AcquireError is variant-less.
                // lint-ok: error-adapt
                .map_err(|e| KnowledgeBankError::Internal(format!("semaphore closed: {e}")))?;
            match self.send_once(url, body).await {
                Ok(value) => return Ok(value),
                Err(Retryable::Fatal(msg)) => return Err(KnowledgeBankError::Backend(msg)),
                Err(Retryable::Transient(msg)) => {
                    tracing::warn!(attempt, error = %msg, "bedrock call failed, retrying");
                    last = msg;
                },
            }
        }
        Err(KnowledgeBankError::Backend(format!(
            "bedrock call failed after {MAX_ATTEMPTS} attempts: {last}"
        )))
    }

    async fn send_once(&self, url: &str, body: &Value) -> Result<Value, Retryable> {
        let response = self
            .http
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            // Why: this IS the re-classification — a transport failure is
            // retryable, and Retryable is the local decision type, never an
            // error surfaced to a caller.
            // lint-ok: error-adapt
            .map_err(|e| Retryable::Transient(format!("transport: {e}")))?;

        let status = response.status();
        if status.is_success() {
            return response
                .json::<Value>()
                .await
                // Why: a malformed body will not become well-formed on a
                // retry, so it is classified fatal rather than transient.
                // lint-ok: error-adapt
                .map_err(|e| Retryable::Fatal(format!("decode: {e}")));
        }
        let detail = response.text().await.unwrap_or_default();
        let message = format!("HTTP {status}: {}", truncate(&detail));
        if status.as_u16() == 429 || status.is_server_error() {
            Err(Retryable::Transient(message))
        } else {
            Err(Retryable::Fatal(message))
        }
    }
}

enum Retryable {
    Transient(String),
    Fatal(String),
}

fn backoff(attempt: u32) -> Duration {
    let exponential = BASE_BACKOFF_MS.saturating_mul(1u64 << attempt.min(10));
    let capped = exponential.min(MAX_BACKOFF_MS);
    let jitter = rand::random::<u64>() % (capped / 2).max(1);
    Duration::from_millis(capped / 2 + jitter)
}

fn truncate(text: &str) -> String {
    const LIMIT: usize = 300;
    if text.len() <= LIMIT {
        return text.to_owned();
    }
    let cut = text
        .char_indices()
        .map(|(i, _)| i)
        .take_while(|i| *i <= LIMIT)
        .last()
        .unwrap_or(0);
    format!("{}…", &text[..cut])
}
