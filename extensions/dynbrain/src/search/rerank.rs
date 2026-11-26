//! Reranking: the cross-encoder in front of the fused order, failing open.
//!
//! [`Reranker`] is the provider (Cohere in production, a double in tests).
//! [`rerank`] reorders the passages by its scores and, whatever happens,
//! returns a [`ScoringTrace`] saying which scoring the scores came from:
//! `cross_encoder` only when the provider answered, `fusion` with the reason
//! otherwise, so nobody has to guess it from the numbers. A failure keeps the
//! fused order.
//!
//! [`RerankBreaker`] writes a provider off after a failure on the wire: 60 s,
//! doubling with each failure in a row up to 900 s, keyed by provider and
//! endpoint. The first call after the window is the trial: a success forgets
//! the failures, another failure reopens it wider. A request the provider
//! answered and refused (400, 401, 403, 422) does not open it; 404 and 410
//! (a retired endpoint), 429, 5xx, timeouts and transport errors do. The
//! extension holds one per process.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::Serialize;

use super::service::Passage;

pub const COOLDOWN: Duration = Duration::from_mins(1);
pub const MAX_COOLDOWN: Duration = Duration::from_mins(15);

#[derive(Debug, thiserror::Error)]
pub enum RerankFailure {
    #[error("the reranker could not be reached")]
    Unreachable(#[source] reqwest::Error),
    #[error("the reranker answered with status {0}")]
    Status(u16),
    #[error("the reranker did not answer within {0:?}")]
    TimedOut(Duration),
}

impl RerankFailure {
    pub fn opens_breaker(&self) -> bool {
        match self {
            Self::Status(code) => !(400..500).contains(code) || matches!(code, 404 | 410 | 429),
            Self::Unreachable(_) | Self::TimedOut(_) => true,
        }
    }
}

/// A cross-encoder that scores documents against a query. Held as
/// `Arc<dyn Reranker>` so tests can swap it, hence `#[async_trait]`.
#[async_trait]
pub trait Reranker: Send + Sync + Debug {
    fn provider(&self) -> &str;

    fn endpoint(&self) -> &str;

    async fn rerank(
        &self,
        query: &str,
        documents: &[String],
    ) -> Result<Vec<(usize, f64)>, RerankFailure>;
}

/// Which scoring produced the scores of a result list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreSource {
    Fusion,
    CrossEncoder,
}

/// Why a result list kept the fused order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FusionReason {
    NotRequested,
    NoHits,
    NoProvider,
    BreakerOpen,
    Error,
    EmptyResponse,
}

/// What scored a result list, and why when it was not the reranker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScoringTrace {
    pub scoring: ScoreSource,
    pub provider: Option<String>,
    pub reason: Option<FusionReason>,
}

impl ScoringTrace {
    pub const fn fused(reason: FusionReason) -> Self {
        Self {
            scoring: ScoreSource::Fusion,
            provider: None,
            reason: Some(reason),
        }
    }

    fn fused_by(provider: &str, reason: FusionReason) -> Self {
        Self {
            scoring: ScoreSource::Fusion,
            provider: Some(provider.to_owned()),
            reason: Some(reason),
        }
    }
}

/// The reranking providers written off, by provider and endpoint.
#[derive(Debug, Default)]
pub struct RerankBreaker {
    open: Mutex<HashMap<(String, String), (u32, Instant)>>,
}

impl RerankBreaker {
    pub fn remaining(&self, provider: &str, endpoint: &str, now: Instant) -> Duration {
        let key = (provider.to_owned(), endpoint.to_owned());
        let until = self
            .open
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key)
            .map(|(_, until)| *until);
        until.map_or(Duration::ZERO, |until| until.saturating_duration_since(now))
    }

    pub fn fail(&self, provider: &str, endpoint: &str, now: Instant) -> Duration {
        let key = (provider.to_owned(), endpoint.to_owned());
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        let failures = open.get(&key).map_or(0, |(failures, _)| *failures) + 1;
        let wait = cooldown(failures);
        open.insert(key, (failures, now + wait));
        drop(open);
        wait
    }

    pub fn succeed(&self, provider: &str, endpoint: &str) {
        let key = (provider.to_owned(), endpoint.to_owned());
        self.open
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&key);
    }
}

pub fn cooldown(failures: u32) -> Duration {
    let doublings = failures.saturating_sub(1).min(10);
    COOLDOWN.saturating_mul(1 << doublings).min(MAX_COOLDOWN)
}

pub async fn rerank(
    reranker: Option<&dyn Reranker>,
    breaker: &RerankBreaker,
    query: &str,
    passages: Vec<Passage>,
) -> (Vec<Passage>, ScoringTrace) {
    if passages.is_empty() {
        return (passages, ScoringTrace::fused(FusionReason::NoHits));
    }
    let Some(reranker) = reranker else {
        return (passages, ScoringTrace::fused(FusionReason::NoProvider));
    };
    let (provider, endpoint) = (reranker.provider(), reranker.endpoint());
    if !breaker
        .remaining(provider, endpoint, Instant::now())
        .is_zero()
    {
        tracing::debug!(
            provider,
            "reranking skipped while its provider is written off"
        );
        return (
            passages,
            ScoringTrace::fused_by(provider, FusionReason::BreakerOpen),
        );
    }
    let documents: Vec<String> = passages
        .iter()
        .map(|passage| passage.content.clone())
        .collect();
    let ordered = match reranker.rerank(query, &documents).await {
        Ok(ordered) => ordered,
        Err(failure) => {
            if failure.opens_breaker() {
                let wait = breaker.fail(provider, endpoint, Instant::now());
                tracing::warn!(provider, error = %failure, wait_secs = wait.as_secs(), "reranking failed on the wire; keeping the fused order");
            } else {
                tracing::error!(provider, error = %failure, "reranking was refused; keeping the fused order");
            }
            return (
                passages,
                ScoringTrace::fused_by(provider, FusionReason::Error),
            );
        },
    };
    breaker.succeed(provider, endpoint);
    let reranked: Vec<Passage> = ordered
        .into_iter()
        .filter_map(|(index, score)| {
            passages.get(index).map(|passage| Passage {
                score: Some(score),
                ..passage.clone()
            })
        })
        .collect();
    if reranked.is_empty() {
        return (
            passages,
            ScoringTrace::fused_by(provider, FusionReason::EmptyResponse),
        );
    }
    let trace = ScoringTrace {
        scoring: ScoreSource::CrossEncoder,
        provider: Some(provider.to_owned()),
        reason: None,
    };
    (reranked, trace)
}
