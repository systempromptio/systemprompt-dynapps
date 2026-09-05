//! `/admin/governance` — what warn mode is currently absorbing.
//!
//! Two planes, one window. The governance chain
//! (`services/governance/config.yaml`) contributes `governance_decisions` rows
//! with `decision = 'warn'`; the gateway safety scanners
//! (`services/gateway/policies.yaml`) contribute `ai_safety_findings` rows
//! whose `blocked` column says whether the finding actually refused anything.
//! Both are shown together because they are configured separately and an
//! operator retuning one needs to see what the other is doing.
//!
//! The page is a measurement instrument, not a dashboard: every number on it
//! answers "what would enforcement have cost here", so the counts are the
//! product and the individual rows are only there to make one credible.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::csv::CsvBuilder;
use crate::handlers::ssr::list_view::TimeRangeContext;
use crate::repositories::governance::warnings::{
    SafetyFindingRollupRow, WarningDecisionRow, WarningGroupBy, WarningRollupRow,
    list_recent_warnings, list_safety_finding_rollup, list_warning_rollup,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

mod view;

use view::{FindingRow, GroupRow, RecentRow};

const BASE_URL: &str = "/admin/governance";
const ROLLUP_LIMIT: i64 = 2000;
const RECENT_LIMIT: i64 = 100;
const FINDING_LIMIT: i64 = 200;

#[derive(Debug, Deserialize)]
pub(crate) struct WarningsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub group_by: Option<String>,
}

#[derive(Debug, Serialize)]
struct WarningsPageContext {
    page: &'static str,
    title: &'static str,
    time_range: TimeRangeContext,
    group_by: &'static str,
    group_label: &'static str,
    group_column: &'static str,
    group_tabs: Vec<GroupTab>,
    csv_url: String,
    total_warnings: i64,
    distinct_policies: i64,
    distinct_users: i64,
    total_findings: i64,
    blocked_findings: i64,
    groups: Vec<GroupRow>,
    has_groups: bool,
    recent: Vec<RecentRow>,
    has_recent: bool,
    findings: Vec<FindingRow>,
    has_findings: bool,
}

#[derive(Debug, Serialize)]
struct GroupTab {
    label: &'static str,
    href: String,
    active: bool,
}

struct WarningsData {
    rollup: Vec<WarningRollupRow>,
    recent: Vec<WarningDecisionRow>,
    findings: Vec<SafetyFindingRollupRow>,
}

// Why: a read surface, so the console predicate applies — a project manager
// reads the governance posture they are accountable for without holding the
// admin role.
fn require_console(user_ctx: &UserContext) -> Result<(), AdminError> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()))
}

async fn load(pool: &PgPool, range: TimeRange) -> WarningsData {
    let rollup = list_warning_rollup(pool, range.from, range.to, ROLLUP_LIMIT)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "governance warning rollup failed");
            Vec::new()
        });
    let recent = list_recent_warnings(pool, range.from, range.to, RECENT_LIMIT)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "recent governance warnings failed");
            Vec::new()
        });
    let findings = list_safety_finding_rollup(pool, range.from, range.to, FINDING_LIMIT)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "safety finding rollup failed");
            Vec::new()
        });
    WarningsData {
        rollup,
        recent,
        findings,
    }
}

pub(crate) async fn governance_warnings_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<WarningsQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&user_ctx)?;

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let group_by = WarningGroupBy::parse(query.group_by.as_deref());
    let data = load(&pool, range).await;

    let groups = view::group_rows(&data.rollup, group_by);
    let recent = view::recent_rows(&data.recent);
    let findings = view::finding_rows(&data.findings);

    let ctx = WarningsPageContext {
        page: "governance-warnings",
        title: "Governance warnings",
        time_range: view::time_range_context(range, query.preset.as_deref(), group_by),
        group_by: group_by.as_str(),
        group_label: group_by.label(),
        group_column: group_by.label(),
        group_tabs: group_tabs(range, query.preset.as_deref(), group_by),
        csv_url: view::csv_url(range, group_by),
        total_warnings: data.rollup.iter().map(|r| r.count).sum(),
        distinct_policies: view::distinct(data.rollup.iter().map(|r| r.policy.as_str())),
        distinct_users: view::distinct(data.rollup.iter().map(|r| r.user_id.as_str())),
        total_findings: data.findings.iter().map(|r| r.count).sum(),
        blocked_findings: data.findings.iter().map(|r| r.blocked_count).sum(),
        has_groups: !groups.is_empty(),
        groups,
        has_recent: !recent.is_empty(),
        recent,
        has_findings: !findings.is_empty(),
        findings,
    };

    Ok(super::render_typed_page(
        &engine,
        "governance-warnings",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn governance_warnings_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<WarningsQuery>,
) -> AdminResult<Response> {
    require_console(&user_ctx)?;

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let group_by = WarningGroupBy::parse(query.group_by.as_deref());
    let data = load(&pool, range).await;

    // Why: one file with both planes rather than two downloads. The whole
    // point of the window is comparing them, and two files lose the fact that
    // they cover the same one.
    let mut csv = CsvBuilder::new(&[
        "section",
        "group",
        "warnings",
        "tools",
        "users",
        "first_seen",
        "last_seen",
        "example_reason",
    ]);
    for row in view::group_rows(&data.rollup, group_by) {
        csv.row(&[
            "warning",
            &row.group,
            &row.warnings.to_string(),
            &row.tools.to_string(),
            &row.users.to_string(),
            &row.first_seen,
            &row.last_seen,
            &row.example_reason,
        ]);
    }
    for row in view::finding_rows(&data.findings) {
        csv.row(&[
            "finding",
            &format!("{} / {}", row.category, row.scanner),
            &row.findings.to_string(),
            &row.phase,
            &row.severity,
            "",
            &row.last_seen,
            &format!("{} blocked", row.blocked),
        ]);
    }

    Ok(csv.into_response(&format!(
        "governance-warnings-{}.csv",
        range.from.format("%Y%m%d")
    )))
}

fn group_tabs(range: TimeRange, preset: Option<&str>, active: WarningGroupBy) -> Vec<GroupTab> {
    [
        WarningGroupBy::Policy,
        WarningGroupBy::Tool,
        WarningGroupBy::User,
    ]
    .into_iter()
    .map(|g| GroupTab {
        label: g.label(),
        href: view::page_url(range, preset, g),
        active: g == active,
    })
    .collect()
}
