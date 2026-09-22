//! The conversation list behind `/admin/history` and `/admin/conversations`.
//!
//! A conversation is a `/v1/messages` context: the gateway's `ai_requests`
//! rows grouped by `context_id` and classified by `conversation_requests`, so
//! a turn here is a turn everywhere and probes and utility calls are side
//! calls. A conversation made of side calls alone is listed only when the
//! caller asks for them. The client session the conversation ran in (a
//! Claude Code session id) is carried when the client said which one.
//!
//! Search matches by `ILIKE` over the title, the opening prompt, the id and
//! the model; the paged rows and the total are computed in one statement.
//! The window is a conversation's last activity in `[since, until)`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

/// One conversation in the history list, with the client session it ran in
/// when the client named one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem<Id = ContextId> {
    pub context_id: Id,
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub user_label: String,
    pub title: Option<String>,
    pub preview: Option<String>,
    pub model: Option<String>,
    // Why: `ai_requests.client_kind` / `client_attestation` of the latest
    // request.
    pub client_kind: String,
    pub client_attestation: String,
    pub started_at: Option<DateTime<Utc>>,
    pub last_at: DateTime<Utc>,
    pub turns: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub cost_microdollars: i64,
    pub side_call_count: i64,
}

/// What narrows one history page: whose conversations, matching what text,
/// whether side calls count, and when they were last active.
#[derive(Debug, Clone, Copy)]
pub struct HistoryFilter<'a> {
    pub scope_user_ids: Option<&'a [String]>,
    pub search: Option<&'a str>,
    pub include_side_calls: bool,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

pub async fn list_history_items(
    pool: &PgPool,
    filter: HistoryFilter<'_>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<HistoryItem>, i64), sqlx::Error> {
    let HistoryFilter {
        scope_user_ids,
        search,
        include_side_calls,
        since,
        until,
    } = filter;
    let query = search.map(str::trim).filter(|q| !q.is_empty());
    let pattern = query.map(|q| format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%")));
    let legacy = ContextId::legacy();

    let mut transaction = crate::repositories::dashboard_read::begin(pool).await?;
    let row = sqlx::query_file!(
        "src/repositories/analytics/conversations/history_page.sql",
        scope_user_ids,
        limit,
        offset,
        legacy.as_str(),
        pattern,
        include_side_calls,
        since,
        until,
    )
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;
    let items = row
        .payload
        .0
        .items
        .into_iter()
        .map(|r| {
            Ok(HistoryItem {
                context_id: crate::repositories::dashboard_read::context_id(
                    &r.context_id,
                    &row.context_ids,
                )?,
                session_id: r.session_id,
                user_id: r.user_id,
                user_label: r.user_label,
                title: r.title,
                preview: r.preview,
                model: r.model,
                client_kind: r.client_kind,
                client_attestation: r.client_attestation,
                started_at: r.started_at,
                last_at: r.last_at,
                turns: r.turns,
                total_input_tokens: r.total_input_tokens,
                total_output_tokens: r.total_output_tokens,
                cost_microdollars: r.cost_microdollars,
                side_call_count: r.side_call_count,
            })
        })
        .collect::<Result<_, sqlx::Error>>()?;
    Ok((items, row.payload.0.total))
}

#[derive(Debug, Deserialize)]
struct HistoryPageResult {
    items: Vec<HistoryItem<String>>,
    total: i64,
}
