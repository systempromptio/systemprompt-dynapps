//! `/admin/analysis/conversations` — every conversation's record with the
//! judge's label, and the breakdown buckets over the same filtered set.

use async_trait::async_trait;

use super::requests::time_range;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::analysis::conversations::export::export_rows;
use crate::repositories::analysis::conversations::{ConversationBucketRow, ConversationFactRow};
use crate::types::UserContext;
use systemprompt::identifiers::{SessionId, UserId};

pub(crate) struct ClassifiedConversations;
pub(crate) struct Breakdown;

const COLUMNS: &[Column] = &[
    Column::new("first_at", "Started", CellKind::Timestamp),
    Column::new("last_at", "Last activity", CellKind::Timestamp),
    Column::new("context_id", "Context", CellKind::Text),
    Column::new("session_id", "Session", CellKind::Text).optional(),
    Column::new("client_session_id", "Harness session", CellKind::Text).optional(),
    Column::new("title", "Title", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("display_name", "Name", CellKind::Text),
    Column::new("group_name", "Group", CellKind::Text),
    Column::new("project_name", "Project", CellKind::Text),
    Column::new("client", "Client", CellKind::Text),
    Column::new("attestation", "Attestation", CellKind::Text).optional(),
    Column::new("wire_protocol", "Wire", CellKind::Text).optional(),
    Column::new("model", "Model", CellKind::Text),
    Column::new("models", "Models", CellKind::Text).optional(),
    Column::new("provider", "Provider", CellKind::Text).optional(),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("turns", "Turns", CellKind::Integer),
    Column::new("side_calls", "Side calls", CellKind::Integer).optional(),
    Column::new("errors", "Failed requests", CellKind::Integer),
    Column::new("rejected", "Rejected", CellKind::Integer).optional(),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cache_read_tokens", "Cache read tokens", CellKind::Integer).optional(),
    Column::new(
        "cache_creation_tokens",
        "Cache write tokens",
        CellKind::Integer,
    )
    .optional(),
    Column::new("reasoning_tokens", "Reasoning tokens", CellKind::Integer).optional(),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("p50_latency_ms", "p50 latency (ms)", CellKind::Integer).optional(),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Integer).optional(),
    Column::new("tool_calls", "Tool calls", CellKind::Integer),
    Column::new("tool_calls_executed", "Tools executed", CellKind::Integer).optional(),
    Column::new("tool_calls_failed", "Tools failed", CellKind::Integer).optional(),
    Column::new("artifacts", "Artifacts", CellKind::Integer).optional(),
    Column::new("gov_allow", "Allowed", CellKind::Integer).optional(),
    Column::new("gov_warn", "Warned", CellKind::Integer).optional(),
    Column::new("gov_deny", "Denied", CellKind::Integer),
    Column::new("safety_findings", "Safety findings", CellKind::Integer).optional(),
    Column::new("safety_blocked", "Safety blocked", CellKind::Integer).optional(),
    Column::new("prompts", "Hook prompts", CellKind::Integer).optional(),
    Column::new("skill_invocations", "Skill invocations", CellKind::Integer).optional(),
    Column::new("skills", "Skills", CellKind::Text),
    Column::new("duration_seconds", "Duration (s)", CellKind::Integer),
    Column::new("category", "Intent", CellKind::Text),
    Column::new("outcome", "Outcome", CellKind::Text),
    Column::new("completion", "Completion", CellKind::Integer),
    Column::new("summary", "Summary", CellKind::Text),
    Column::new("tags", "Tags", CellKind::Text).optional(),
    Column::new("classified_at", "Judged", CellKind::Timestamp).optional(),
    Column::new("judge_model", "Judge model", CellKind::Text).optional(),
];

fn row(r: &ConversationFactRow) -> Vec<Cell> {
    vec![
        Cell::opt_time(Some(r.first_at)),
        Cell::opt_time(Some(r.last_at)),
        r.context_id.as_str().into(),
        Cell::opt_text(r.session_id.as_ref().map(SessionId::as_str)),
        Cell::opt_text(r.client_session_id.as_deref()),
        Cell::opt_text(r.judge_title.as_deref().or(Some(r.title.as_str()))),
        r.user_id.as_str().into(),
        Cell::opt_text(r.display_name.as_deref()),
        Cell::opt_text(r.group_name.as_deref()),
        Cell::opt_text(r.project_name.as_deref()),
        r.client_kind.as_str().into(),
        r.client_attestation.as_str().into(),
        r.wire_protocol.as_str().into(),
        Cell::opt_text(r.model.as_deref()),
        Cell::list(&r.models),
        Cell::opt_text(r.provider.as_deref()),
        r.request_count.into(),
        r.turn_count.into(),
        r.side_call_count.into(),
        r.error_count.into(),
        r.rejected_count.into(),
        r.input_tokens.into(),
        r.output_tokens.into(),
        r.cache_read_tokens.into(),
        r.cache_creation_tokens.into(),
        r.reasoning_tokens.into(),
        Cell::Money(r.cost_microdollars),
        Cell::opt_int(r.p50_latency_ms),
        Cell::opt_int(r.p95_latency_ms),
        r.tool_calls_intended.into(),
        r.tool_calls_executed.into(),
        r.tool_calls_failed.into(),
        r.artifact_count.into(),
        r.gov_allow.into(),
        r.gov_warn.into(),
        r.gov_deny.into(),
        r.safety_findings.into(),
        r.safety_blocked.into(),
        r.prompt_count.into(),
        r.skill_invocations.into(),
        Cell::list(&r.skills),
        r.duration_seconds.into(),
        Cell::opt_text(r.category.as_deref()),
        Cell::opt_text(r.outcome.as_deref()),
        Cell::opt_int(r.completion),
        Cell::opt_text(r.summary.as_deref()),
        Cell::list(&r.tags),
        Cell::opt_time(r.classified_at),
        Cell::opt_text(r.judge_model.as_deref()),
    ]
}

const BUCKET_COLUMNS: &[Column] = &[
    Column::new("bucket", "Bucket", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text).optional(),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("users", "Users", CellKind::Integer),
    Column::new("turns", "Turns", CellKind::Integer),
    Column::new("tool_calls", "Tool calls", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("errors", "Failed requests", CellKind::Integer),
    Column::new("denied", "Denied", CellKind::Integer),
    Column::new("achieved", "Achieved", CellKind::Integer),
    Column::new("judged", "Judged", CellKind::Integer),
    Column::new("completion_avg", "Mean completion", CellKind::Decimal).optional(),
    Column::new("top_category", "Top intent", CellKind::Text),
];

fn bucket_row(r: &ConversationBucketRow) -> Vec<Cell> {
    vec![
        r.label.as_str().into(),
        Cell::opt_text(r.user_id.as_ref().map(UserId::as_str)),
        r.conversations.into(),
        r.users.into(),
        r.turns.into(),
        r.tool_calls.into(),
        r.total_tokens.into(),
        Cell::Money(r.total_cost_microdollars),
        r.errors.into(),
        r.denied.into(),
        r.achieved.into(),
        r.judged.into(),
        Cell::opt_decimal(r.completion_avg),
        Cell::opt_text(r.top_category.as_deref()),
    ]
}

#[async_trait]
impl DataSet for ClassifiedConversations {
    fn id(&self) -> &'static str {
        "analysis-conversations"
    }
    fn title(&self) -> &'static str {
        "Conversations"
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    // Why: the loader narrows to the caller's scope, so a participant exports
    // exactly the rows the page shows them.
    fn allows(&self, user: &UserContext) -> bool {
        user.has_scoped_console()
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let data = export_rows(
            ctx.pool,
            ctx.user,
            ctx.query()?,
            time_range(ctx)?,
            ctx.limit,
        )
        .await?;
        Ok(Table {
            rows: data.rows.iter().map(row).collect(),
            total: data.totals.conversations,
        })
    }
}

#[async_trait]
impl DataSet for Breakdown {
    fn id(&self) -> &'static str {
        "analysis-conversation-breakdown"
    }
    fn title(&self) -> &'static str {
        "Conversation breakdown"
    }
    fn columns(&self) -> &'static [Column] {
        BUCKET_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    // Why: the loader narrows to the caller's scope, so a participant exports
    // exactly the rows the page shows them.
    fn allows(&self, user: &UserContext) -> bool {
        user.has_scoped_console()
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let data = export_rows(ctx.pool, ctx.user, ctx.query()?, time_range(ctx)?, 1).await?;
        Ok(Table::complete(
            data.breakdown.iter().map(bucket_row).collect(),
        ))
    }
}
