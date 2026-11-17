//! What systemprompt itself recorded for a farm task: its model calls
//! (`ai_requests`) and its conversation (`conversation_session_contexts`).
//!
//! farm-api says which task ran; systemprompt says what it cost. Every task
//! that ran its CLI through the systemprompt gateway carries two keys in
//! farm-api:
//!
//! - `usage.gateway_session_id` (`sess_<uuid>`) equals
//!   `ai_requests.session_id`: one bridge proxy per task pod, so every model
//!   call of that run and no other. This is the key for calls, tokens and cost.
//! - `session_ref` without its `sess_` prefix equals
//!   `conversation_session_contexts.client_session_id`: the Claude Code
//!   session, shared by a resume chain. This is the key for the conversation,
//!   never for cost.
//!
//! Costs stay in what systemprompt stores: USD, as `cost_microdollars`.
//!
//! The pages read through [`SpUsage`], so tests can serve fixed rows; the
//! console uses [`PgSpUsage`] over the admin router's read pool. Every query
//! is a read.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::types::SessionTotalRow;

/// The most calls the task page lists (the totals by model cover them all).
pub const MAX_CALLS: i64 = 500;

/// One model call of a task, from `ai_requests`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpCall {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub model: Option<String>,
    pub status: String,
    pub request_kind: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cache_read_tokens: Option<i32>,
    pub cache_creation_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
}

/// The calls of one task added up per model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpModelTotal {
    pub model: Option<String>,
    pub calls: i64,
    pub failed: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_microdollars: i64,
}

/// systemprompt's tables could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpError(pub String);

impl fmt::Display for SpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "systemprompt usage could not be read: {}", self.0)
    }
}

impl std::error::Error for SpError {}

impl From<sqlx::Error> for SpError {
    fn from(e: sqlx::Error) -> Self {
        Self(e.to_string())
    }
}

/// Read access to systemprompt's own record of the farm's model calls.
///
/// `#[async_trait]` because the pages hold it as `Arc<dyn SpUsage>` (tests
/// swap in a fake): a dyn-compatible trait cannot use native async fns.
#[async_trait]
pub trait SpUsage: Send + Sync {
    /// The calls of one gateway session, oldest first, at most [`MAX_CALLS`].
    async fn calls(&self, gateway_session_id: &str) -> Result<Vec<SpCall>, SpError>;

    /// The calls of one gateway session added up per model, costliest first.
    async fn totals_by_model(&self, gateway_session_id: &str)
    -> Result<Vec<SpModelTotal>, SpError>;

    /// The conversation contexts of one Claude Code session.
    async fn contexts(&self, client_session_id: &str) -> Result<Vec<String>, SpError>;

    /// The calls of each of the given gateway sessions added up. Sessions
    /// with no calls are absent from the answer.
    async fn session_totals(
        &self,
        gateway_session_ids: &[String],
    ) -> Result<Vec<SessionTotalRow>, SpError>;
}

/// The handle the pages share.
pub type SharedSp = Arc<dyn SpUsage>;

/// [`SpUsage`] over systemprompt's database.
#[derive(Clone)]
pub struct PgSpUsage {
    pool: Arc<PgPool>,
}

impl PgSpUsage {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl fmt::Debug for PgSpUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PgSpUsage")
    }
}

#[async_trait]
impl SpUsage for PgSpUsage {
    async fn calls(&self, gateway_session_id: &str) -> Result<Vec<SpCall>, SpError> {
        Ok(sqlx::query_file_as!(
            SpCall,
            "src/sql/task_calls.sql",
            gateway_session_id,
            MAX_CALLS,
        )
        .fetch_all(self.pool.as_ref())
        .await?)
    }

    async fn totals_by_model(
        &self,
        gateway_session_id: &str,
    ) -> Result<Vec<SpModelTotal>, SpError> {
        Ok(sqlx::query_file_as!(
            SpModelTotal,
            "src/sql/task_totals_by_model.sql",
            gateway_session_id,
        )
        .fetch_all(self.pool.as_ref())
        .await?)
    }

    async fn contexts(&self, client_session_id: &str) -> Result<Vec<String>, SpError> {
        Ok(
            sqlx::query_file_scalar!("src/sql/task_contexts.sql", client_session_id)
                .fetch_all(self.pool.as_ref())
                .await?,
        )
    }

    async fn session_totals(
        &self,
        gateway_session_ids: &[String],
    ) -> Result<Vec<SessionTotalRow>, SpError> {
        Ok(sqlx::query_file_as!(
            SessionTotalRow,
            "src/sql/session_totals.sql",
            gateway_session_ids,
        )
        .fetch_all(self.pool.as_ref())
        .await?)
    }
}
