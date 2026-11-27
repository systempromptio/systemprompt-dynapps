//! The Cohere reranker (`rerank-v3.5`), over `reqwest` with a 5 s limit.
//!
//! The key is a secret of the instance (`cohere`); without it there is no
//! reranker and searches keep the fused order with `confidence: unrated`.
//! The endpoint passes core's outbound URL guard once, when the client is
//! built.

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use systemprompt::models::net::{LoopbackPolicy, OutboundUrlError, validate_outbound_url};

use super::rerank::{RerankFailure, Reranker};

pub const PROVIDER: &str = "cohere";
pub const ENDPOINT: &str = "https://api.cohere.com/v2/rerank";
pub const MODEL: &str = "rerank-v3.5";
pub const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum CohereSetupError {
    #[error("the reranker endpoint is not an allowed destination")]
    Endpoint(#[source] Box<OutboundUrlError>),
    #[error("the reranker's HTTP client could not be built")]
    Client(#[from] reqwest::Error),
}

/// Cohere's rerank endpoint, called with the instance's key.
pub struct CohereReranker {
    http: reqwest::Client,
    endpoint: url::Url,
    key: String,
    model: String,
}

impl fmt::Debug for CohereReranker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CohereReranker")
            .field("endpoint", &self.endpoint.as_str())
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

#[derive(Serialize)]
struct RerankRequest<'a> {
    model: &'a str,
    query: &'a str,
    documents: &'a [String],
    top_n: usize,
}

#[derive(Deserialize)]
struct RerankResponse {
    results: Vec<RerankResult>,
}

#[derive(Deserialize)]
struct RerankResult {
    index: usize,
    relevance_score: f64,
}

impl CohereReranker {
    pub fn new(key: String, endpoint: &str, model: &str) -> Result<Self, CohereSetupError> {
        let endpoint = validate_outbound_url(endpoint, LoopbackPolicy::Allow)
            .map_err(|error| CohereSetupError::Endpoint(Box::new(error)))?;
        let http = reqwest::Client::builder().timeout(TIMEOUT).build()?;
        Ok(Self {
            http,
            endpoint,
            key,
            model: model.to_owned(),
        })
    }
}

#[async_trait]
impl Reranker for CohereReranker {
    fn provider(&self) -> &str {
        PROVIDER
    }

    fn endpoint(&self) -> &str {
        self.endpoint.as_str()
    }

    async fn rerank(
        &self,
        query: &str,
        documents: &[String],
    ) -> Result<Vec<(usize, f64)>, RerankFailure> {
        let request = RerankRequest {
            model: &self.model,
            query,
            documents,
            top_n: documents.len(),
        };
        let response = self
            .http
            .post(self.endpoint.clone())
            .bearer_auth(&self.key)
            .json(&request)
            .send()
            .await
            .map_err(failure)?;
        let status = response.status();
        if !status.is_success() {
            return Err(RerankFailure::Status(status.as_u16()));
        }
        let body: RerankResponse = response.json().await.map_err(failure)?;
        Ok(body
            .results
            .into_iter()
            .map(|result| (result.index, result.relevance_score))
            .collect())
    }
}

fn failure(error: reqwest::Error) -> RerankFailure {
    if error.is_timeout() {
        return RerankFailure::TimedOut(TIMEOUT);
    }
    RerankFailure::Unreachable(error)
}
