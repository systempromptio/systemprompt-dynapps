//! The governance CSV export.
//!
//! Split from the page so the handler module stays inside its size ceiling, and
//! because the export answers a different question from the screen: it stays
//! one row per policy evaluation while the console folds them into calls.
//! Evidence wants everything that was decided; a console wants the summary.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::handlers::ssr::csv::CsvBuilder;
use crate::types::UserContext;

use super::{
    GovernanceQuery, decision_filter, finding_filter, range_of, require_console, resolve_scope,
    sort_from, view,
};

// Why: an export is a bounded read, not a stream. Five thousand evaluations is
// several times the largest window an operator opens and still a file a
// spreadsheet will accept.
const CSV_LIMIT: i64 = 5_000;

pub(crate) async fn governance_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GovernanceQuery>,
) -> AdminResult<Response> {
    require_console(&user_ctx)?;

    let range = range_of(&query);
    let (_, scope) = resolve_scope(&pool, &user_ctx, &query).await?;

    let (decisions, decision_total) =
        crate::repositories::governance::decision_log::list_governance_decisions_paged(
            &pool,
            range,
            &scope,
            &decision_filter(&query),
            sort_from(&query),
            CSV_LIMIT,
            0,
        )
        .await?;
    let (findings, finding_total) =
        crate::repositories::governance::findings::list_safety_findings_paged(
            &pool,
            range,
            &scope,
            &finding_filter(&query),
            CSV_LIMIT,
            0,
        )
        .await?;

    // Why: the export stays one row per evaluation while the console folds them
    // into calls. An evidence export wants everything that was decided, not the
    // summary the screen shows, and `trace_id` is here so the grouping the
    // console performs is reproducible from the file.
    let mut csv = CsvBuilder::new(&[
        "plane",
        "at",
        "trace_id",
        "outcome",
        "policy_or_category",
        "stage_or_scanner",
        "tool_or_model",
        "user",
        "scope",
        "reason",
        "record_id",
        "session_id",
        "call_id",
        "request_id",
        "phase",
        "evaluated_rules",
        "export_truncated",
        "matching_decisions",
        "matching_findings",
    ]);
    let truncated = (decision_total > CSV_LIMIT || finding_total > CSV_LIMIT).to_string();
    let decision_total = decision_total.to_string();
    let finding_total = finding_total.to_string();
    for row in &decisions {
        csv.row(&[
            "chain",
            &row.created_at.to_rfc3339(),
            row.trace_id.as_deref().unwrap_or(""),
            &row.decision,
            &row.policy,
            &view::plane_of(&row.policy),
            &row.tool_name,
            row.user_id.as_str(),
            row.agent_scope.as_deref().unwrap_or(""),
            &row.reason,
            &row.id,
            &row.session_id,
            row.call_id.as_deref().unwrap_or(""),
            "",
            "request",
            &row.evidence,
            &truncated,
            &decision_total,
            &finding_total,
        ]);
    }
    for row in &findings {
        csv.row(&[
            "safety",
            &row.created_at.to_rfc3339(),
            row.trace_id.as_deref().unwrap_or(""),
            if row.blocked { "blocked" } else { "audited" },
            &row.category,
            &row.scanner,
            row.model.as_deref().unwrap_or(""),
            row.user_id.as_ref().map_or("", |u| u.as_str()),
            "",
            row.excerpt.as_deref().unwrap_or(""),
            &row.id,
            row.session_id.as_deref().unwrap_or(""),
            "",
            &row.ai_request_id,
            &row.phase,
            "",
            &truncated,
            &decision_total,
            &finding_total,
        ]);
    }

    Ok(csv.into_response(&format!("governance-{}.csv", range.from.format("%Y%m%d"))))
}
