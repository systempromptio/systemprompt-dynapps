//! HTTP handlers for the access-control matrix and rule editing.

use std::str::FromStr;
use std::sync::Arc;
use systemprompt::traits::BackgroundTasks;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::governance::authz::{EntityKind, RegisteredEntities};
use systemprompt::identifiers::UserId;

use systemprompt_web_admin_core::activity::{ActivityEntity, NewActivity};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::gateway_catalog::registered_routes_from_services;
use systemprompt_web_admin_core::types::UserContext;
use systemprompt_web_admin_core::types::access_control::{
    AccessControlQuery, AccessControlRule, BulkAssignRequest, UpdateEntityRulesRequest,
};
use systemprompt_web_admin_core::{activity, catalog_index, request as shared};

#[derive(Debug, Serialize)]
pub(crate) struct RulesResponse {
    pub rules: Vec<AccessControlRule>,
}

#[derive(Debug, Serialize)]
pub(crate) struct BulkAssignResponse {
    pub updated_count: usize,
    pub rules_per_entity: usize,
}

pub(crate) async fn list_access_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AccessControlQuery>,
) -> AdminResult<Response> {
    let rules = if let (Some(et), Some(eid)) = (&query.entity_type, &query.entity_id) {
        systemprompt_web_admin_core::repositories::entitlement::list_rules_for_entity(
            &pool, et, eid,
        )
        .await?
    } else {
        systemprompt_web_admin_core::repositories::entitlement::list_all_rules(&pool).await?
    };

    Ok(Json(RulesResponse { rules }).into_response())
}

pub(crate) async fn update_entity_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Extension(tasks): Extension<BackgroundTasks>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpdateEntityRulesRequest>,
) -> AdminResult<Response> {
    let kind = editable_entity_kind(&entity_type)?;
    registered_routes_from_services()?.require(kind, &entity_id)?;
    require_band_justification(&body.rules)?;

    let rules = systemprompt_web_admin_core::repositories::entitlement::set_entity_rules(
        &pool,
        kind,
        &entity_id,
        &body.rules,
        &user_ctx.user_id,
    )
    .await?;

    if entity_type == "gateway_route" {
        let uid = user_ctx.user_id.clone();
        let eid = entity_id.clone();
        let pool_arc = Arc::clone(&pool);
        tasks.spawn("access_control_activity", async move {
            activity::record(
                &pool_arc,
                NewActivity::entity_updated(&uid, ActivityEntity::GatewayRoute, &eid, &eid),
            )
            .await;
        });
    }
    Ok(Json(RulesResponse { rules }).into_response())
}

pub(crate) async fn bulk_assign_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<BulkAssignRequest>,
) -> AdminResult<Response> {
    let registered = registered_routes_from_services()?;
    let entities = body
        .entities
        .iter()
        .map(|e| registered_entity(&registered, &e.entity_type, &e.entity_id))
        .collect::<AdminResult<Vec<(EntityKind, String)>>>()?;

    require_band_justification(&body.rules)?;
    let rules_per_entity = body.rules.len();
    let updated_count = systemprompt_web_admin_core::repositories::entitlement::bulk_set_rules(
        &pool,
        &entities,
        &body.rules,
        &user_ctx.user_id,
    )
    .await?;
    Ok(Json(BulkAssignResponse {
        updated_count,
        rules_per_entity,
    })
    .into_response())
}

pub(crate) fn require_band_justification(
    rules: &[systemprompt_web_admin_core::types::access_control::AccessControlRuleInput],
) -> AdminResult<()> {
    let missing = rules.iter().find(|r| {
        r.rule_type != systemprompt_web_admin_core::types::access_control::RuleType::USER
            && r.justification
                .as_deref()
                .is_none_or(|j| j.trim().is_empty())
    });
    missing.map_or(Ok(()), |rule| {
        Err(AdminError::BadRequest(format!(
            "a reason (justification) is required for the {} rule on '{}'",
            rule.rule_type, rule.rule_value
        )))
    })
}

fn editable_entity_kind(entity_type: &str) -> AdminResult<EntityKind> {
    let kind = EntityKind::from_str(entity_type)?;
    if matches!(
        kind,
        EntityKind::Plugin
            | EntityKind::Agent
            | EntityKind::McpServer
            | EntityKind::Marketplace
            | EntityKind::Skill
            | EntityKind::GatewayRoute
    ) {
        Ok(kind)
    } else {
        Err(AdminError::BadRequest(
            "Invalid entity_type. Must be plugin, agent, mcp_server, marketplace, skill, or \
             gateway_route."
                .to_owned(),
        ))
    }
}

fn registered_entity(
    registered: &RegisteredEntities,
    entity_type: &str,
    entity_id: &str,
) -> AdminResult<(EntityKind, String)> {
    let kind = EntityKind::from_str(entity_type)?;
    registered.require(kind, entity_id)?;
    Ok((kind, entity_id.to_owned()))
}

pub(crate) async fn user_matrix_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;

    let sections = build_matrix_sections(&services_path);

    let user_id = UserId::new(user_id);
    let matrix = systemprompt_web_admin_core::repositories::entitlement::resolve_user_matrix(
        &pool, &user_id, sections,
    )
    .await?
    .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?;
    Ok(Json(matrix).into_response())
}

pub(crate) fn build_matrix_sections(
    services_path: &std::path::Path,
) -> Vec<systemprompt_web_admin_core::repositories::entitlement::SectionInput> {
    let mut sections: Vec<systemprompt_web_admin_core::repositories::entitlement::SectionInput> =
        Vec::new();

    let marketplace_rows: Vec<(String, String, Option<String>)> =
        catalog_index::marketplaces::load_marketplaces()
            .into_iter()
            .map(|m| (m.id.to_string(), m.name, Some(m.description)))
            .collect();
    if !marketplace_rows.is_empty() {
        sections.push((
            "marketplace".to_owned(),
            "Marketplaces".to_owned(),
            marketplace_rows,
        ));
    }

    match systemprompt_web_admin_core::repositories::gateway_catalog::get_route_labels_from_services(
    ) {
        Err(e) => {
            tracing::error!(error = %e, "gateway routes unavailable for the access-control matrix");
        },
        Ok(labels) => {
            let rows = labels
                .routes
                .iter()
                .map(|r| (r.id.clone(), r.label.clone(), Some(r.subtitle())))
                .collect();
            sections.push(("gateway_route".to_owned(), "Model routes".to_owned(), rows));
        },
    }

    push_catalog_sections(services_path, &mut sections);
    sections
}

fn push_catalog_sections(
    services_path: &std::path::Path,
    sections: &mut Vec<systemprompt_web_admin_core::repositories::entitlement::SectionInput>,
) {
    if let Ok(servers) = catalog_index::mcp_servers::list_mcp_servers(services_path) {
        let rows: Vec<(String, String, Option<String>)> = servers
            .into_iter()
            .map(|s| {
                let id = s.id.as_str().to_owned();
                let desc = if s.description.is_empty() {
                    None
                } else {
                    Some(s.description)
                };
                (id.clone(), id, desc)
            })
            .collect();
        sections.push(("mcp_server".to_owned(), "MCP servers".to_owned(), rows));
    }
    let admin_roles = vec!["admin".to_owned()];
    if let Ok(plugins) =
        catalog_index::plugin_overview::list_plugins_for_roles(services_path, &admin_roles)
    {
        let rows: Vec<(String, String, Option<String>)> = plugins
            .into_iter()
            .map(|p| (p.id, p.name, Some(p.description)))
            .collect();
        sections.push(("plugin".to_owned(), "Plugins".to_owned(), rows));
    }
    if let Ok(agents) = catalog_index::agents::list_configured_agents(services_path) {
        let rows: Vec<(String, String, Option<String>)> = agents
            .into_iter()
            .map(|a| {
                (
                    a.id.as_str().to_owned(),
                    a.name.clone(),
                    Some(a.description.clone()),
                )
            })
            .collect();
        sections.push(("agent".to_owned(), "Agents".to_owned(), rows));
    }
    if let Ok(skills) = catalog_index::plugin_catalog::list_skill_catalog(services_path) {
        let rows: Vec<(String, String, Option<String>)> = skills
            .into_iter()
            .map(|s| {
                let desc = if s.description.is_empty() {
                    None
                } else {
                    Some(s.description)
                };
                (s.id.as_str().to_owned(), s.name, desc)
            })
            .collect();
        sections.push(("skill".to_owned(), "Skills".to_owned(), rows));
    }
}
