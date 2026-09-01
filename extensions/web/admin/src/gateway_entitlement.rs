//! Gateway guard enforcing which routes a caller is entitled to.
//!
//! Registered through `register_gateway_guard!` and consulted by the gateway
//! on every `/v1/messages` request right after the quota precheck. The
//! resolved route is a `gateway_route` entity, so this is the ordinary authz
//! resolver run over the same rules the access matrix shows, with every
//! subject dimension in the ladder. A denial is 403: no amount of retrying
//! buys a model the caller is not entitled to.

use sqlx::PgPool;
use systemprompt::extension::{
    GatewayDenyReason, GatewayGuardRequest, GatewayRequestGuard, register_gateway_guard,
};
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::resolver::{ResolveInput, resolve};
use systemprompt_security::authz::{Decision, EntityRef};

use crate::authz;
use crate::repositories::config::gateway_acl;

#[derive(Debug, Clone, Copy, Default)]
pub struct RouteEntitlementGuard;

#[async_trait::async_trait]
impl GatewayRequestGuard for RouteEntitlementGuard {
    // Why: Fails **open** on a lookup error and on an unresolved route, and
    // **closed** on an actual deny decision.
    //
    // The asymmetry is deliberate. A deny is an answer — the resolver read
    // the rules and the customer is not entitled — and honouring it is the
    // whole point. A database error is not an answer, and treating it as one
    // would turn a blip into a total inference outage for every customer at
    // once. An unmatched route means the request never reached a governed
    // entity and the gateway will fail it on its own terms.
    async fn check(
        &self,
        pool: &PgPool,
        request: &GatewayGuardRequest<'_>,
    ) -> Result<(), GatewayDenyReason> {
        let Some(route_id) = request.route_id else {
            return Ok(());
        };
        let user_id = UserId::new(request.user_id.to_owned());
        let Some(decision) = resolve_route(pool, route_id, &user_id).await else {
            return Ok(());
        };
        let Decision::Deny { reason } = decision else {
            return Ok(());
        };

        tracing::warn!(
            user_id = request.user_id,
            route_id,
            model = request.model,
            ?reason,
            "gateway request denied: route not granted to the caller",
        );
        Err(GatewayDenyReason::forbidden(format!(
            "{} is not available to your account.",
            request.model
        )))
    }
}

async fn resolve_route(pool: &PgPool, route_id: &str, user_id: &UserId) -> Option<Decision> {
    let entity = gateway_acl::find_entity(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, route_id, "route entity lookup failed"))
        .ok()?;
    let rules = gateway_acl::list_rules_for_route(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, route_id, "route rule lookup failed"))
        .ok()?;
    let user_roles = load_roles(pool, user_id).await?;
    let attributes = authz::subject_attributes_for(pool, user_id).await;

    let entity_ref =
        EntityRef::GatewayRoute(systemprompt::identifiers::RouteId::new(route_id.to_owned()));

    Some(resolve(ResolveInput {
        entity: &entity_ref,
        rules: &rules,
        user_id,
        user_roles: &user_roles,
        default_included: entity.map(|e| e.default_included),
        parents: &[],
        attributes: &attributes,
        dimensions: authz::dimensions(pool),
    }))
}

async fn load_roles(pool: &PgPool, user_id: &UserId) -> Option<Vec<String>> {
    sqlx::query_scalar!(
        r#"SELECT roles AS "roles!: Vec<String>" FROM users WHERE id = $1"#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, user_id = %user_id, "role lookup failed"))
    .ok()
    .flatten()
}

register_gateway_guard!(RouteEntitlementGuard);
