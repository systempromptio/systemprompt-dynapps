//! `/admin/analytics` — the tables behind the dashboard's Models, Skills,
//! Tools and Sessions tabs; the Cost tab's tables are `dashboard_cost`.
//! Every table is a `live_dataset!` — the trait boilerplate is one macro, so
//! each table is only its column list and the read behind it.

use async_trait::async_trait;
use systemprompt::models::SessionId;

use crate::handlers::ssr::ssr_analytics_dashboard::AnalyticsDashboardQuery;
use crate::repositories::analytics::site::models::list_model_stats;
use crate::repositories::analytics::site::sessions::list_session_costs_paged;
use crate::repositories::analytics::site::skills::list_skill_stats;
use crate::repositories::analytics::site::{SiteScope, resolve_site_scope};
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::export::model::{
    Cell, CellKind, Column, DataSet, ExportContext, Table, Window,
};
use systemprompt_web_admin_core::export::registry::DataSetRegistration;
use systemprompt_web_admin_core::export::window::time_range;
use systemprompt_web_admin_core::util::time_range::TimeRange;

mod tools;

pub(crate) struct ModelStats;
pub(crate) struct SkillStats;
pub(crate) struct SessionCosts;

inventory::submit! {
    DataSetRegistration {
        order: 14,
        dataset: &ModelStats,
    }
}
inventory::submit! {
    DataSetRegistration {
        order: 15,
        dataset: &SkillStats,
    }
}
inventory::submit! {
    DataSetRegistration {
        order: 21,
        dataset: &SessionCosts,
    }
}

pub(super) struct DashboardRead {
    pub(super) query: AnalyticsDashboardQuery,
    pub(super) scope: SiteScope,
    pub(super) range: TimeRange,
}

pub(super) async fn plan(ctx: &ExportContext<'_>) -> AdminResult<DashboardRead> {
    let query: AnalyticsDashboardQuery = ctx.query()?;
    let scope = resolve_site_scope(ctx.pool, &query.scope(), query.attribution()).await?;
    Ok(DashboardRead {
        query,
        scope,
        range: time_range(ctx)?,
    })
}

macro_rules! live_dataset {
    ($ty:ident, $id:literal, $title:literal, $description:literal, $columns:ident,
     |$ctx:ident| $load:expr) => {
        #[async_trait]
        impl DataSet for $ty {
            fn id(&self) -> &'static str {
                $id
            }
            fn title(&self) -> &'static str {
                $title
            }
            fn description(&self) -> &'static str {
                $description
            }
            fn columns(&self) -> &'static [Column] {
                $columns
            }
            fn window(&self) -> Window {
                Window::Live
            }
            async fn load(&self, $ctx: &ExportContext<'_>) -> AdminResult<Table> {
                $load
            }
        }
    };
}
pub(super) use live_dataset;

const MODEL_COLUMNS: &[Column] = &[
    Column::new("model", "Model", CellKind::Text),
    Column::new("provider", "Provider", CellKind::Text),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cache_tokens", "Cache tokens", CellKind::Integer),
    Column::new("reasoning_tokens", "Reasoning tokens", CellKind::Integer).optional(),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("p50_latency_ms", "p50 latency (ms)", CellKind::Decimal),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Decimal),
    Column::new("errors", "Errors", CellKind::Integer),
    Column::new("redirected", "Redirected", CellKind::Integer).optional(),
    Column::new("unrouted", "Unrouted", CellKind::Bool).optional(),
];

live_dataset!(
    ModelStats,
    "analytics-models",
    "Models",
    "One row per model: requests, tokens, cost, latency and errors over the window.",
    MODEL_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let rows = list_model_stats(ctx.pool, p.range, &p.scope).await?;
        Ok(Table::complete(
            rows.iter()
                .map(|r| {
                    vec![
                        r.model.as_str().into(),
                        Cell::opt_text(r.provider.as_deref()),
                        r.requests.into(),
                        r.input_tokens.into(),
                        r.output_tokens.into(),
                        r.cache_tokens.into(),
                        r.reasoning_tokens.into(),
                        Cell::Money(r.cost_microdollars),
                        Cell::opt_decimal(r.p50_latency_ms),
                        Cell::opt_decimal(r.p95_latency_ms),
                        r.errors.into(),
                        r.redirected.into(),
                        r.is_unrouted.into(),
                    ]
                })
                .collect(),
        ))
    }
);

const SKILL_COLUMNS: &[Column] = &[
    Column::new("skill", "Skill", CellKind::Text),
    Column::new("resource_id", "Resource", CellKind::Text).optional(),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new("slash_invocations", "Slash invocations", CellKind::Integer),
    Column::new("tool_invocations", "Tool invocations", CellKind::Integer),
    Column::new("attributed_invocations", "Attributed", CellKind::Integer),
    Column::new("distinct_users", "Users", CellKind::Integer),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("priced_requests", "Priced requests", CellKind::Integer).optional(),
    Column::new(
        "conversation_cost_usd",
        "Conversation cost (USD)",
        CellKind::Money,
    ),
    Column::new(
        "conversation_tokens",
        "Conversation tokens",
        CellKind::Integer,
    ),
];

live_dataset!(
    SkillStats,
    "analytics-skills",
    "Skills",
    "One row per skill: invocations by kind, people, conversations and requests over the window.",
    SKILL_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let (rows, total) = list_skill_stats(ctx.pool, p.range, &p.scope, ctx.limit, 0).await?;
        Ok(Table {
            rows: rows
                .iter()
                .map(|r| {
                    vec![
                        r.skill.as_str().into(),
                        Cell::opt_text(r.resource_id.as_deref()),
                        r.invocations.into(),
                        r.slash_invocations.into(),
                        r.tool_invocations.into(),
                        r.attributed_invocations.into(),
                        r.distinct_users.into(),
                        r.conversations.into(),
                        r.requests.into(),
                        r.priced_requests.into(),
                        Cell::Money(r.conversation_cost_microdollars),
                        r.conversation_tokens.into(),
                    ]
                })
                .collect(),
            total,
        })
    }
);

const SESSION_COLUMNS: &[Column] = &[
    Column::new("updated_at", "Updated", CellKind::Timestamp),
    Column::new("conversation_id", "Conversation", CellKind::Text),
    Column::new("session_id", "Client session", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cache_read_tokens", "Cache read tokens", CellKind::Integer),
];

live_dataset!(
    SessionCosts,
    "analytics-sessions",
    "Session costs",
    "One row per conversation: person, model, requests, tokens and cost over the window.",
    SESSION_COLUMNS,
    |ctx| {
        let p = plan(ctx).await?;
        let (rows, total) =
            list_session_costs_paged(ctx.pool, p.range, &p.scope, ctx.limit, 0).await?;
        Ok(Table {
            rows: rows
                .iter()
                .map(|r| {
                    vec![
                        r.updated_at.into(),
                        r.context_id.as_str().into(),
                        Cell::opt_text(r.session_id.as_ref().map(SessionId::as_str)),
                        r.user_id.as_str().into(),
                        Cell::opt_text(r.model.as_deref()),
                        Cell::Money(r.total_cost_microdollars),
                        r.request_count.into(),
                        r.input_tokens.into(),
                        r.output_tokens.into(),
                        r.cache_read_tokens.into(),
                    ]
                })
                .collect(),
            total,
        })
    }
);
