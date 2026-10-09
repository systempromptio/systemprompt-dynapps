//! Gateway guard enforcing which routes a caller is entitled to.
//!
//! Registered through `register_gateway_guard!` and consulted by the gateway
//! on every `/v1/messages` request right after the quota precheck. The
//! resolved route is a `gateway_route` entity, so this is the ordinary authz
//! resolver run over the same rules the access matrix shows, with every
//! subject dimension in the ladder. A denial is 403: no amount of retrying
//! buys a model the caller is not entitled to.
//!
//! The route's entity and rules are cached per route and the caller's roles
//! per user for 30 s (`ttl_cache`); subject attributes are cached by their
//! providers. An access-control sync apply or a console rule edit drops every
//! entry on this replica at once.

use std::sync::LazyLock;

use sqlx::PgPool;
use systemprompt::extension::{
    GatewayDenyReason, GatewayGuardRequest, GatewayRequestGuard, register_gateway_guard,
};
use systemprompt::identifiers::{RouteId, UserId};
use systemprompt_security::authz::resolver::{ResolveInput, resolve};
use systemprompt_security::authz::{AccessRule, Decision, EntityRef, EntityRow};
use systemprompt_web_admin_core::ttl_cache::{AUTHZ_TTL, TtlCache};

use crate::repositories::config::gateway_acl;

type RouteGrant = (Option<EntityRow>, Vec<AccessRule>);

static ROUTE_GRANTS: LazyLock<TtlCache<(String, RouteId), RouteGrant>> =
    LazyLock::new(|| TtlCache::new(AUTHZ_TTL));

static USER_ROLES: LazyLock<TtlCache<(String, UserId), Vec<String>>> =
    LazyLock::new(|| TtlCache::new(AUTHZ_TTL));

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
    let (entity, rules) = route_grant(pool, &database, route_id).await?;
    let user_roles = cached_roles(pool, &database, user_id).await?;
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

async fn route_grant(
    pool: &PgPool,
    database: &str,
    route_id: &RouteId,
) -> Result<RouteGrant, sqlx::Error> {
    let key = (database.to_owned(), route_id.clone());
    if let Some(grant) = ROUTE_GRANTS.get(&key) {
        return Ok(grant);
    }
    let entity = gateway_acl::find_entity(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, %route_id, "route entity lookup failed"))?;
    let rules = gateway_acl::list_rules_for_route(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, %route_id, "route rule lookup failed"))?;
    let grant = (entity, rules);
    ROUTE_GRANTS.insert(key, grant.clone());
    Ok(grant)
}

async fn cached_roles(
    pool: &PgPool,
    database: &str,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    let key = (database.to_owned(), user_id.clone());
    if let Some(roles) = USER_ROLES.get(&key) {
        return Ok(roles);
    }
    let roles = load_roles(pool, user_id).await?;
    USER_ROLES.insert(key, roles.clone());
    Ok(roles)
}

async fn load_roles(pool: &PgPool, user_id: &UserId) -> Result<Vec<String>, sqlx::Error> {
    let identity =
        systemprompt_web_admin_core::repositories::principal::identity::find_identity_envelope(
            pool, user_id,
        )
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    if identity.status != "active" {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(identity.roles)
}

register_gateway_guard!(RouteEntitlementGuard);
