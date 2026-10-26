//! `GET /agents`: every agent in a farm whose marketplace the caller holds.

use std::collections::BTreeSet;
use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Serialize;
use systemprompt::identifiers::PluginId;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::types::UserContext;

use crate::repositories::{agents, farms};
use crate::services::entitlement::{is_farm_owner, list_entitled_marketplaces};
use crate::state::FarmState;
use crate::types::{FarmAgentId, FarmId};

#[derive(Debug, Serialize)]
struct AgentJson {
    farm_id: FarmId,
    id: FarmAgentId,
    name: String,
    description: String,
    model: Option<String>,
    plugin_id: PluginId,
    skills: Vec<String>,
    fixture: Option<String>,
    max_turns: Option<i32>,
    max_budget_usd: Option<f64>,
}

#[derive(Debug, Serialize)]
struct AgentsResponse {
    agents: Vec<AgentJson>,
}

pub(crate) async fn list_agents(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    let pool = state.pool.as_ref();
    let all_farms = farms::list_farms(pool).await?;
    let marketplaces: BTreeSet<String> = all_farms
        .iter()
        .map(|f| f.marketplace_id.as_str().to_owned())
        .collect();
    let held = if ctx.is_console {
        marketplaces
    } else {
        list_entitled_marketplaces(pool, &state.services, &ctx, &marketplaces).await?
    };
    let usable: BTreeSet<String> = all_farms
        .iter()
        .filter(|f| held.contains(f.marketplace_id.as_str()) || is_farm_owner(&ctx, f))
        .map(|f| f.id.as_str().to_owned())
        .collect();
    let agents = agents::list_all_agents(pool)
        .await?
        .into_iter()
        .filter(|a| usable.contains(a.farm_id.as_str()))
        .map(|a| AgentJson {
            farm_id: a.farm_id,
            id: a.id,
            name: a.name,
            description: a.description,
            model: a.model,
            plugin_id: a.plugin_id,
            skills: a.skills,
            fixture: a.fixture,
            max_turns: a.max_turns,
            max_budget_usd: a.max_budget_usd,
        })
        .collect();
    Ok(Json(AgentsResponse { agents }).into_response())
}
