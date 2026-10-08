//! HTTP handlers for managed resource listings.

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::sync::Arc;

use systemprompt::identifiers::AgentId;

use systemprompt_web_admin_core::catalog_index::agents;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::request as shared;
use systemprompt_web_admin_core::types::UserContext;

use systemprompt_web_admin_core::types::responses::AgentsListResponse;

pub(crate) async fn list_agents_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    let agents = agents::list_configured_agents(&services_path).map_err(AdminError::internal)?;
    if user_ctx.is_admin {
        return Ok(Json(AgentsListResponse { agents }).into_response());
    }
    let access =
        systemprompt_web_admin_core::authz::catalog::CatalogAccess::load(&pool, &user_ctx.user_id)
            .await?;
    let ids = agents
        .iter()
        .filter(|a| a.enabled)
        .map(|a| a.id.to_string())
        .collect::<Vec<_>>();
    let allowed = access
        .allowed(systemprompt_security::authz::EntityKind::Agent, &ids)
        .await?;
    let filtered: Vec<_> = agents
        .into_iter()
        .filter(|a| a.enabled && allowed.contains(a.id.as_str()))
        .collect();
    Ok(Json(AgentsListResponse { agents: filtered }).into_response())
}

pub(crate) async fn get_agent_handler(Path(agent_id): Path<String>) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    let agent = agents::find_agent(&services_path, &AgentId::new(agent_id))
        .map_err(AdminError::internal)?
        .ok_or_else(|| AdminError::NotFound("Agent not found".to_owned()))?;
    Ok(Json(agent).into_response())
}
