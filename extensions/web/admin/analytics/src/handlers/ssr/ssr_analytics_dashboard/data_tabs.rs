//! Data collection for the five entity tabs.
//!
//! Each tab loads only its own queries, so opening Models never pays for the
//! skill view. Every failed `Result` goes to the page's fault collector: one
//! failing aggregate degrades its own table and is named on the page, rather
//! than taking the page down.

use sqlx::PgPool;

use crate::repositories::analytics::site::cost::{
    ContainerAxis, ContainerUsageRow, CostDayRow, SupplierCostRow, list_container_usage,
    list_provider_cost_by_day, list_provider_costs,
};
use crate::repositories::analytics::site::models::{ModelStatsRow, list_model_stats};
use crate::repositories::analytics::site::sessions::{SessionCostRow, list_session_costs_paged};
use crate::repositories::analytics::site::skills::{
    SkillStatsRow, SkillTotals, get_skill_totals, list_skill_stats,
};
use crate::repositories::analytics::site::tools::{
    ToolServerRow, ToolStatsRow, list_tool_servers, list_tool_stats,
};
use crate::repositories::analytics::site::{SiteScope, distribution};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::util::time_range::TimeRange;

#[derive(Default)]
pub(super) struct TabData {
    pub models: Vec<ModelStatsRow>,
    pub skills: Vec<SkillStatsRow>,
    pub skills_total: i64,
    pub skill_totals: SkillTotals,
    pub tool_servers: Vec<ToolServerRow>,
    pub tools: Vec<ToolStatsRow>,
    pub tools_total: i64,
    pub sessions: Vec<SessionCostRow>,
    pub sessions_total: i64,
    pub cost_days: Vec<CostDayRow>,
    pub cost_providers: Vec<SupplierCostRow>,
    pub cost_models: Vec<SupplierCostRow>,
    pub cost_containers: Vec<ContainerUsageRow>,
}

pub(super) struct TabPlan<'a> {
    pub scope: &'a SiteScope,
    pub range: TimeRange,
    pub limit: i64,
    pub offset: i64,
    pub axis: ContainerAxis,
}

pub(super) async fn load_models(pool: &PgPool, plan: &TabPlan<'_>, faults: &mut Faults) -> TabData {
    let stats = list_model_stats(pool, plan.range, plan.scope).await;
    TabData {
        models: faults.take("model stats", stats),
        ..TabData::default()
    }
}

pub(super) async fn load_skills(pool: &PgPool, plan: &TabPlan<'_>, faults: &mut Faults) -> TabData {
    let (paged, totals) = tokio::join!(
        list_skill_stats(pool, plan.range, plan.scope, plan.limit, plan.offset),
        get_skill_totals(pool, plan.range, plan.scope),
    );
    let (skills, skills_total) = faults.take("skill stats", paged);
    TabData {
        skills,
        skills_total,
        skill_totals: faults.take("skill totals", totals),
        ..TabData::default()
    }
}

pub(super) async fn load_tools(pool: &PgPool, plan: &TabPlan<'_>, faults: &mut Faults) -> TabData {
    let (servers, paged) = tokio::join!(
        list_tool_servers(pool, plan.range, plan.scope),
        list_tool_stats(pool, plan.range, plan.scope, plan.limit, plan.offset),
    );
    let (tools, tools_total) = faults.take("tool stats", paged);
    TabData {
        tool_servers: faults.take("tool servers", servers),
        tools,
        tools_total,
        ..TabData::default()
    }
}

pub(super) async fn load_sessions(
    pool: &PgPool,
    plan: &TabPlan<'_>,
    faults: &mut Faults,
) -> TabData {
    let paged =
        list_session_costs_paged(pool, plan.range, plan.scope, plan.limit, plan.offset).await;
    let (sessions, sessions_total) = faults.take("session costs", paged);
    TabData {
        sessions,
        sessions_total,
        ..TabData::default()
    }
}

pub(super) async fn load_cost(pool: &PgPool, plan: &TabPlan<'_>, faults: &mut Faults) -> TabData {
    let (days, providers, models, containers) = tokio::join!(
        list_provider_cost_by_day(pool, plan.range, plan.scope),
        list_provider_costs(pool, plan.range, plan.scope),
        distribution::list_model_distribution(pool, plan.range, plan.scope),
        list_container_usage(pool, plan.range, plan.scope, plan.axis),
    );
    TabData {
        cost_days: faults.take("cost by day", days),
        cost_providers: faults.take("cost by provider", providers),
        cost_models: faults
            .take("cost by model", models)
            .into_iter()
            .map(|m| SupplierCostRow {
                label: m.model,
                requests: m.requests,
                tokens: m.tokens,
                cost_microdollars: m.cost_microdollars,
            })
            .collect(),
        cost_containers: faults.take("cost by container", containers),
        ..TabData::default()
    }
}
