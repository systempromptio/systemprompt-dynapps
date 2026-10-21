//! Gateway guard enforcing which routes a caller is entitled to.
//!
//! Registered through `register_gateway_guard!` and consulted by the gateway
//! on every `/v1/messages` request right after the quota precheck. The
//! resolved route is a `gateway_route` entity, so this is the ordinary authz
//! resolver run over the same rules the access matrix shows, with every
//! subject dimension in the ladder. A denial is 403: no amount of retrying
//! buys a model the caller is not entitled to.
//!
//! The route's entity and rules and the caller's roles come from
//! [`crate::authz_eval`]'s 30 s caches; subject attributes are cached by their
//! providers. An access-control sync apply or a console rule edit drops every
//! entry on this replica at once.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::extension::{
    GatewayDenyReason, GatewayGuardRequest, GatewayRequestGuard, register_gateway_guard,
};
use systemprompt::identifiers::{RouteId, UserId};
use systemprompt::governance::authz::resolver::{ResolveInput, resolve};
use systemprompt::governance::authz::{Decision, EntityKind, EntityRef};

use crate::authz_eval;

#[derive(Debug, Clone, Copy, Default)]
pub struct RouteEntitlementGuard;

#[async_trait::async_trait]
impl GatewayRequestGuard for RouteEntitlementGuard {
    async fn check(
        &self,
        database: &dyn systemprompt::traits::DatabaseHandle,
        request: &GatewayGuardRequest<'_>,
    ) -> Result<(), GatewayDenyReason> {
        let Some(route_id) = request.route_id else {
            return Ok(());
        };
        let database = database
            .as_any()
            .downcast_ref::<systemprompt::database::Database>()
            .ok_or_else(|| GatewayDenyReason::unavailable("Authorization database unavailable"))?;
        let pool = database.pool();
        let user_id = request.user_id;
        let decision = resolve_route(&pool, route_id, user_id)
            .await
            .map_err(|error| {
                tracing::error!(%error, %user_id, %route_id, "authorization_unavailable");
                GatewayDenyReason::unavailable("Authorization temporarily unavailable")
            })?;
        if decision.permits() {
            return Ok(());
        }

        tracing::warn!(
            user_id = %request.user_id,
            %route_id,
            model = %request.model,
            ?decision,
            "gateway request denied: route not granted to the caller",
        );
        Err(GatewayDenyReason::forbidden(format!(
            "{} is not available to your account.",
            request.model
        )))
    }
}

async fn resolve_route(
    pool: &PgPool,
    route_id: &RouteId,
    user_id: &UserId,
) -> Result<Decision, sqlx::Error> {
    let database = systemprompt_web_admin_core::authz::database_key(pool);
    let shared = Arc::new(pool.clone());
    let (entity, rules) = authz_eval::entity_grant(
        &shared,
        &database,
        EntityKind::GatewayRoute,
        route_id.as_str(),
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, %route_id, "route grant lookup failed"))?;
    let user_roles = authz_eval::active_roles(pool, &database, user_id).await?;
    let attributes =
        systemprompt_web_admin_core::authz::subject_attributes_for(pool, user_id).await?;

    let entity_ref = EntityRef::GatewayRoute(route_id.clone());

    Ok(resolve(ResolveInput {
        entity: &entity_ref,
        rules: &rules,
        user_id,
        user_roles: &user_roles,
        default_included: entity.map(|e| e.default_included),
        parents: &[],
        attributes: &attributes,
        dimensions: systemprompt_web_admin_core::authz::dimensions(pool),
    }))
}

register_gateway_guard!(RouteEntitlementGuard);
