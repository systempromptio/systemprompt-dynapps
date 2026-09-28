//! `/admin/governance` — what the two enforcement planes decided.
//!
//! Three tabs over one window and one scope. **Decisions** is the governance
//! chain's log: one row per evaluated tool call, with the stage that decided
//! it. **Safety** is the gateway scanners' log, which runs in both directions
//! and records findings whether or not they refused anything. **Hooks** is the
//! plumbing underneath — which hook fired, how often, and who is tripping it
//! most.
//!
//! The two planes are configured separately (`services/governance/config.yaml`
//! and `services/gateway/policies.yaml`) and an operator retuning one has to
//! see what the other is doing, so the KPI strip spans both on every tab: the
//! four chain stages' deny counts beside the scanners' findings and blocks. A
//! findings count with no blocks under it is warn mode absorbing the traffic,
//! and that reading is the point of putting them side by side.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::repositories::governance::decision_log::{DecisionFilter, DecisionLogRow, DecisionSort};
use crate::repositories::governance::findings::{FindingFilter, SafetyFindingLogRow};
use crate::repositories::governance::{DecisionPage, PageSlice};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::scope::{ScopeRequest, SubjectScope, membership};
use systemprompt_web_admin_core::ssr::list_view::{DEFAULT_PAGE_SIZE, scope_filter_view};
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};
use systemprompt_web_admin_core::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

mod columns;
mod context;
mod data;
mod kpis;
mod urls;
mod view;


pub(crate) const BASE_URL: &str = "/admin/governance";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GovernanceTab {
    Decisions,
    Safety,
    Hooks,
}

impl GovernanceTab {
    fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("safety") => Self::Safety,
            Some("hooks") => Self::Hooks,
            _ => Self::Decisions,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Decisions => "decisions",
            Self::Safety => "safety",
            Self::Hooks => "hooks",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Decisions => "Decisions",
            Self::Safety => "Safety findings",
            Self::Hooks => "Hook events",
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct GovernanceQuery {
    pub tab: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub category: Option<String>,
    pub blocked: Option<String>,
    pub q: Option<String>,
    pub attention: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(super) struct TabLink {
    label: &'static str,
    href: String,
    is_active: bool,
    count: i64,
}

fn require_console(user_ctx: &UserContext) -> Result<(), AdminError> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()))
}

fn sort_from(query: &GovernanceQuery) -> DecisionSort {
    let key = match query.sort.as_deref() {
        Some("policy") => "policy",
        Some("decision") => "decision",
        Some("tool") => "tool",
        Some("user") => "user",
        _ => "when",
    };
    DecisionSort {
        key,
        ascending: query.dir.as_deref() == Some("asc"),
    }
}

fn decision_filter(query: &GovernanceQuery) -> DecisionFilter {
    DecisionFilter {
        policy: non_empty(query.policy.as_deref()),
        decision: non_empty(query.decision.as_deref()),
        search: non_empty(query.q.as_deref()),
        attention: attention_on(query),
    }
}

fn attention_on(query: &GovernanceQuery) -> bool {
    non_empty(query.attention.as_deref()).is_some_and(|v| v != "0")
}

fn finding_filter(query: &GovernanceQuery) -> FindingFilter {
    FindingFilter {
        category: non_empty(query.category.as_deref()),
        blocked: match query.blocked.as_deref() {
            Some("blocked") => Some(true),
            Some("audited") => Some(false),
            _ => None,
        },
        search: non_empty(query.q.as_deref()),
    }
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty()).map(str::to_owned)
}

fn range_of(query: &GovernanceQuery) -> TimeRange {
    parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    })
}

async fn resolve_scope(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: &GovernanceQuery,
) -> Result<(ScopeRequest, SubjectScope), AdminError> {
    let request =
        ScopeRequest::from_query(user_ctx, query.group.as_deref(), query.project.as_deref());
    let scope = membership::get_subject_scope(pool, &request).await?;
    Ok((request, scope))
}

pub(crate) async fn export_decisions(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: GovernanceQuery,
    range: TimeRange,
    limit: i64,
) -> Result<(Vec<DecisionLogRow>, i64), AdminError> {
    require_console(user_ctx)?;
    let (_, scope) = resolve_scope(pool, user_ctx, &query).await?;
    Ok(
        crate::repositories::governance::decision_log::list_governance_decisions_paged(
            pool,
            range,
            &scope,
            &decision_filter(&query),
            DecisionPage {
                sort: sort_from(&query),
                slice: PageSlice::first(limit),
            },
        )
        .await?,
    )
}

pub(crate) async fn export_findings(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: GovernanceQuery,
    range: TimeRange,
    limit: i64,
) -> Result<(Vec<SafetyFindingLogRow>, i64), AdminError> {
    require_console(user_ctx)?;
    let (_, scope) = resolve_scope(pool, user_ctx, &query).await?;
    Ok(
        crate::repositories::governance::findings::list_safety_findings_paged(
            pool,
            range,
            &scope,
            &finding_filter(&query),
            PageSlice::first(limit),
        )
        .await?,
    )
}

pub(crate) async fn governance_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GovernanceQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&user_ctx)?;

    let tab = GovernanceTab::parse(query.tab.as_deref());
    let range = range_of(&query);
    let (request, scope) = resolve_scope(&pool, &user_ctx, &query).await?;
    let sort = sort_from(&query);
    let page = query.page.unwrap_or(0).max(0);

    let loaded = data::load(
        &pool,
        data::GovernanceRead {
            tab,
            range,
            scope: &scope,
            decision_filter: &decision_filter(&query),
            finding_filter: &finding_filter(&query),
            sort,
            page_size: DEFAULT_PAGE_SIZE,
            offset: page * DEFAULT_PAGE_SIZE,
        },
    )
    .await;

    let scope_filter = scope_filter_view(
        &pool,
        &user_ctx,
        &request,
        BASE_URL,
        vec![("tab".to_owned(), tab.as_str().to_owned())],
    )
    .await;

    let ctx = context::build(context::Build {
        query: &query,
        tab,
        range,
        page,
        sort,
        scope_filter,
        data: &loaded,
    });

    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &engine,
            "governance-warnings",
            &ctx,
            &user_ctx,
            &mkt_ctx,
        ),
    )
}
