//! Per-user gateway catalog and defense-in-depth detector.
//!
//! Real enforcement happens in core via the `AuthzDecisionHook` (template's
//! `/govern/authz` webhook). This module provides two redundant surfaces:
//!
//! * [`for_user_handler`] — `GET /api/admin/gateway/catalog/for-user/{user_id}`
//!   filters the catalog to routes the user can see, reducing UI clutter.
//! * [`detect_handler`] — sweeps recent `ai_requests` and writes
//!   `governance_decisions` rows for any request that should not have been
//!   allowed. Acts as a redundancy check that core enforcement actually fired.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use systemprompt::governance::authz::resolver::ResolveInput;
use systemprompt::governance::authz::{
    AccessControlRepository, ChainSources, DecisionFacts, DenyReason, EntityKind, EntityRef,
    ParentChainIndex, ResolveBase,
};
use systemprompt::identifiers::{RouteId, UserId};

use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use crate::repositories::config::acl_detect;
use crate::repositories::config::gateway_acl::{self, Decision};

use systemprompt_web_admin_core::types::{GatewayRouteView, UserContext};

#[derive(Debug, Serialize)]
pub(crate) struct CatalogEntry {
    pub id: String,
    pub model_pattern: String,
    pub provider: String,
    pub upstream_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CatalogResponse {
    pub user_id: UserId,
    pub routes: Vec<CatalogEntry>,
}

pub(crate) async fn for_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id);
    if !user_ctx.is_console && user_ctx.user_id != user_id {
        return Err(AdminError::Forbidden("Forbidden".to_owned()));
    }
    let config = services.config();
    let routes =
        systemprompt_web_admin_core::repositories::gateway_catalog::client_facing_routes(&config)
            .map_err(AdminError::internal)?;

    let user_roles =
        systemprompt_web_admin_core::repositories::principal::role::find_user_access_profile(
            &pool, &user_id,
        )
        .await?
        .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?
        .roles;

    let routes = collect_allowed_routes(&services, &pool, &routes, &user_id, &user_roles).await?;
    Ok(Json(CatalogResponse { user_id, routes }).into_response())
}

async fn collect_allowed_routes(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    pool: &PgPool,
    routes: &[GatewayRouteView],
    user_id: &UserId,
    user_roles: &[String],
) -> AdminResult<Vec<CatalogEntry>> {
    let attributes = (services).subject_attributes(user_id).await?;
    let mut allowed = Vec::with_capacity(routes.len());
    for route in routes {
        let route_id = RouteId::new(route.id.clone());
        let rules = gateway_acl::list_rules_for_route(pool, &route_id).await?;
        let default_included = gateway_acl::find_entity(pool, &route_id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, %route_id, "Failed to load catalog entity");
                None
            })
            .map(|e| e.default_included);
        let entity = EntityRef::GatewayRoute(route_id);
        if gateway_acl::resolve(ResolveInput {
            entity: &entity,
            rules: &rules,
            user_id,
            user_roles,
            default_included,
            parents: &[],
            attributes: &attributes,
            dimensions: (services).dimensions(),
        })
        .permits()
        {
            allowed.push(CatalogEntry {
                id: route.id.clone(),
                model_pattern: route.models_line(),
                provider: route
                    .deployments
                    .first()
                    .map(|d| d.provider().to_owned())
                    .unwrap_or_default(),
                upstream_model: route.alias.as_ref().map(|alias| alias.model.clone()),
            });
        }
    }
    Ok(allowed)
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DetectQuery {
    #[serde(default = "default_since_minutes")]
    pub since_minutes: i64,
}

const fn default_since_minutes() -> i64 {
    60
}

#[derive(Debug, Serialize)]
pub(crate) struct DetectResponse {
    pub emitted: usize,
    pub since_minutes: i64,
}

pub(crate) async fn detect_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    Extension(user_ctx): Extension<UserContext>,
    axum::extract::Query(query): axum::extract::Query<DetectQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin only".to_owned()));
    }
    let config = services.config();
    let routes =
        systemprompt_web_admin_core::repositories::gateway_catalog::dispatchable_routes(&config)
            .map_err(AdminError::internal)?;
    let emitted = detect_after_the_fact(&pool, &services, &routes, query.since_minutes).await?;
    Ok(Json(DetectResponse {
        emitted,
        since_minutes: query.since_minutes,
    })
    .into_response())
}

#[derive(Debug, Serialize)]
struct AclDetectAudit<'a> {
    ai_request_id: &'a str,
    model: &'a str,
    matched_route_id: Option<&'a str>,
    reason: &'a DenyReason,
}

pub(crate) async fn detect_after_the_fact(
    pool: &PgPool,
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    routes: &[GatewayRouteView],
    since_minutes: i64,
) -> Result<usize, systemprompt_web_admin_core::authz::AuthzReadError> {
    let rows = acl_detect::list_recent_unrejected_requests(pool, since_minutes).await?;
    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    let chains = ParentChainIndex::load(
        &repo,
        Arc::new(ChainSources::from_services(&services.config())),
    )
    .await?;

    let mut emitted = 0usize;
    for row in rows {
        let route = systemprompt_web_admin_core::repositories::gateway_catalog::find_matching_route(
            routes, &row.model,
        );
        let Some(reason) = deny_reason(
            pool,
            PolicyRead {
                services,
                repo: &repo,
                chains: &chains,
            },
            route,
            &row,
        )
        .await?
        else {
            continue;
        };
        record_detected_denial(pool, &row, route, &reason).await?;
        emitted += 1;
    }
    Ok(emitted)
}

struct PolicyRead<'a> {
    services: &'a systemprompt_web_admin_core::extension_services::ExtensionServices,
    repo: &'a AccessControlRepository,
    chains: &'a ParentChainIndex,
}

async fn deny_reason(
    pool: &PgPool,
    PolicyRead {
        services,
        repo,
        chains,
    }: PolicyRead<'_>,
    route: Option<&GatewayRouteView>,
    row: &acl_detect::RecentAiRequest,
) -> Result<Option<DenyReason>, systemprompt_web_admin_core::authz::AuthzReadError> {
    let Some(profile) =
        systemprompt_web_admin_core::repositories::principal::role::find_user_access_profile(
            pool,
            &row.user_id,
        )
        .await?
    else {
        return Ok(None);
    };
    let attributes = (services).subject_attributes(&row.user_id).await?;
    let (kind, entity_id) = route.map_or((EntityKind::Model, row.model.as_str()), |route| {
        (EntityKind::GatewayRoute, route.id.as_str())
    });
    let rules = repo.list_rules_for_entity(kind, entity_id).await?;
    let default_included = repo
        .get_entity(kind, entity_id)
        .await?
        .map(|e| e.default_included);
    let decision = chains.resolve(
        kind,
        entity_id,
        ResolveBase {
            rules: &rules,
            user_id: &row.user_id,
            user_roles: &profile.roles,
            default_included,
            attributes: &attributes,
            dimensions: (services).dimensions(),
            facts: DecisionFacts {
                model: Some(&row.model),
                provider: None,
            },
        },
    );
    Ok(match decision {
        Decision::Deny { reason } => Some(reason),
        Decision::Allow { .. } | Decision::Warn { .. } | Decision::Pending { .. } => None,
    })
}

async fn record_detected_denial(
    pool: &PgPool,
    row: &acl_detect::RecentAiRequest,
    route: Option<&GatewayRouteView>,
    reason: &DenyReason,
) -> Result<(), systemprompt_web_admin_core::authz::AuthzReadError> {
    let decision_id = uuid::Uuid::new_v4().to_string();
    let reason_str = reason.to_string();
    let session_id = row
        .session_id
        .as_ref()
        .map(|s| s.as_str().to_owned())
        .unwrap_or_default();
    // JSON: JSONB `evaluated_rules`, from typed `AclDetectAudit`
    let evaluated = serde_json::to_value(AclDetectAudit {
        ai_request_id: &row.id,
        model: &row.model,
        matched_route_id: route.map(|route| route.id.as_str()),
        reason,
    })
    .map_err(|error| sqlx::Error::Encode(Box::new(error)))?;
    acl_detect::insert_gateway_acl_decision(
        pool,
        acl_detect::GatewayAclDecision {
            decision_id: &decision_id,
            user_id: row.user_id.as_str(),
            session_id: &session_id,
            model: &row.model,
            agent_scope: "inference",
            decision: "deny",
            reason: &reason_str,
            evaluated_rules: &evaluated,
        },
    )
    .await?;
    Ok(())
}
