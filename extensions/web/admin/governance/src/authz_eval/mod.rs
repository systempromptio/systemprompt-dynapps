//! The installation's authorization decision, shared by every caller.
//!
//! Core asks the same question three ways: the in-process extension hook
//! ([`crate::authz_hook`], `governance.authz.hook.mode: extension`, which the
//! gateway uses on every `/v1/messages`), the `POST /govern/authz` webhook
//! (the MCP server and external callers), and the route entitlement guard.
//! [`evaluate`] is the one answer: load the entity's rules and default, the
//! parent chains, the caller's subject attributes and active roles, run the
//! deny-overrides resolver, audit what this hook adds, and return a
//! synchronous verdict.
//!
//! Rules and entity, parent chains and roles are cached for 30 s in the
//! database's [`systemprompt_web_admin_core::authz::AuthzCaches`]; subject
//! attributes are cached by their providers on the same generation. An
//! access-control sync apply or a console rule edit drops every entry on the
//! replica that made it.
//!
//! Subject attributes and roles are looked up rather than read off the
//! request, so a group change or a revocation binds within one cache window,
//! not on token refresh. Every failure answers a deny: a hook that cannot
//! decide must not let the call through.

mod audit;

use std::borrow::Cow;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::governance::authz::{
    AccessControlRepository, AuthzDecision, AuthzRequest, ChainSources, Decision, DecisionFacts,
    DenyReason, EntityKind, ParentChainIndex, ResolveBase, SubjectAttributes,
};
use systemprompt::identifiers::{ModelId, ProviderId, UserId};
use systemprompt_web_admin_core::authz::{caches, dimensions, subject_attributes_for};
use systemprompt_web_admin_core::repositories::principal::identity::find_identity_envelope;

pub(crate) use audit::POLICY_NAME;

pub use systemprompt_web_admin_core::authz::EntityGrant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Unavailable;

struct Subject {
    grant: EntityGrant,
    chains: ParentChainIndex,
    attributes: SubjectAttributes,
    roles: Vec<String>,
}

pub fn unavailable_reason() -> DenyReason {
    DenyReason::PolicyViolation {
        policy: "authorization_unavailable".into(),
        detail: Cow::Borrowed("Authorization temporarily unavailable; retry later"),
    }
}

pub async fn entity_grant(
    pool: &Arc<PgPool>,
    kind: EntityKind,
    id: &str,
) -> Result<EntityGrant, sqlx::Error> {
    let cache = &caches(pool).entity_grants;
    let key = (kind.as_str().to_owned(), id.to_owned());
    if let Some(grant) = cache.get(&key) {
        return Ok(grant);
    }
    let repo = AccessControlRepository::from_pool(Arc::clone(pool));
    let to_sqlx = |e: systemprompt::governance::authz::AuthzError| {
        sqlx::Error::Protocol(format!("{kind:?} {id}: {e}"))
    };
    let rules = repo
        .list_rules_for_entity(kind, id)
        .await
        .map_err(to_sqlx)?;
    let entity = repo.get_entity(kind, id).await.map_err(to_sqlx)?;
    let grant = (entity, rules);
    cache.insert(key, grant.clone());
    Ok(grant)
}

pub async fn active_roles(pool: &PgPool, user_id: &UserId) -> Result<Vec<String>, sqlx::Error> {
    let cache = &caches(pool).user_roles;
    if let Some(roles) = cache.get(user_id) {
        return Ok(roles);
    }
    let identity = find_identity_envelope(pool, user_id)
        .await?
        .filter(|identity| identity.status == "active")
        .ok_or(sqlx::Error::RowNotFound)?;
    cache.insert(user_id.clone(), identity.roles.clone());
    Ok(identity.roles)
}

async fn parent_chains(pool: &Arc<PgPool>) -> Result<ParentChainIndex, Unavailable> {
    let cache = &caches(pool).parent_chains;
    if let Some(chains) = cache.get(&()) {
        return Ok(chains);
    }
    let services = systemprompt::loader::ServicesBootstrap::get().map_err(|error| {
        tracing::error!(%error, "authorization_unavailable: services");
        Unavailable
    })?;
    let repo = AccessControlRepository::from_pool(Arc::clone(pool));
    let chains = ParentChainIndex::load(&repo, Arc::new(ChainSources::from_services(services)))
        .await
        .map_err(|error| {
            tracing::error!(%error, "authorization_unavailable: parent policies");
            Unavailable
        })?;
    cache.insert((), chains.clone());
    Ok(chains)
}

async fn read_subject(pool: &Arc<PgPool>, req: &AuthzRequest) -> Result<Subject, Unavailable> {
    let id = req.entity.id_str();
    let grant = entity_grant(pool, req.entity.kind(), id)
        .await
        .map_err(|error| {
            tracing::error!(%error, %id, "authorization_unavailable: entity rules");
            Unavailable
        })?;
    let chains = parent_chains(pool).await?;
    let attributes = subject_attributes_for(pool, &req.user_id)
        .await
        .map_err(|error| {
            tracing::error!(%error, user_id = %req.user_id, trace_id = %req.trace_id, "authorization_unavailable: attributes");
            Unavailable
        })?;
    let roles = active_roles(pool, &req.user_id).await.map_err(|error| {
        tracing::warn!(%error, user_id = %req.user_id, "authorization_unavailable: identity");
        Unavailable
    })?;
    Ok(Subject {
        grant,
        chains,
        attributes,
        roles,
    })
}

fn deny(reason: DenyReason) -> AuthzDecision {
    AuthzDecision::Deny {
        reason,
        policy: POLICY_NAME.to_owned(),
    }
}

// Why: core's `RuleBasedHook` makes the same call — a synchronous hook cannot
// park a call, so a hold degrades to a deny, never an allow; warn permits.
fn sync_verdict(decision: Decision) -> AuthzDecision {
    match decision {
        Decision::Allow { .. } | Decision::Warn { .. } => AuthzDecision::Allow,
        Decision::Deny { reason } => deny(reason),
        Decision::Pending { .. } => deny(DenyReason::PolicyViolation {
            policy: "require_approval".to_owned(),
            detail: Cow::Borrowed(
                "approval required, but this enforcement point cannot hold a request",
            ),
        }),
    }
}

pub async fn evaluate(pool: &Arc<PgPool>, req: AuthzRequest) -> AuthzDecision {
    decide(pool, req, Audit::Record).await
}

/// The same decision as [`evaluate`], without the audit row: core's probe
/// asks what a request would get (a bridge model list, a preview) and must
/// not record a decision nobody made.
pub async fn probe(pool: &Arc<PgPool>, req: AuthzRequest) -> AuthzDecision {
    decide(pool, req, Audit::Skip).await
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Audit {
    Record,
    Skip,
}

async fn decide(pool: &Arc<PgPool>, mut req: AuthzRequest, audit: Audit) -> AuthzDecision {
    let Ok(subject) = read_subject(pool, &req).await else {
        if audit == Audit::Record {
            audit::audit_unavailable(pool, &req).await;
        }
        return deny(unavailable_reason());
    };
    req.roles = subject.roles;
    let (entity, rules) = &subject.grant;
    let model = req.context.gateway_invocation_model();
    let provider = req.context.gateway_invocation_provider();
    let decision = subject.chains.resolve(
        req.entity.kind(),
        req.entity.id_str(),
        ResolveBase {
            rules,
            user_id: &req.user_id,
            user_roles: &req.roles,
            default_included: entity.as_ref().map(|e| e.default_included),
            attributes: &subject.attributes,
            dimensions: dimensions(pool),
            facts: DecisionFacts {
                model: model.as_ref().map(ModelId::as_str),
                provider: provider.as_ref().map(ProviderId::as_str),
            },
        },
    );
    if audit == Audit::Record {
        audit::audit_decision(pool, &req, rules, entity.as_ref(), &decision).await;
    }
    sync_verdict(decision)
}
