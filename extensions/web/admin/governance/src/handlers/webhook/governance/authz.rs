//! `POST /govern/authz` — extension webhook implementing
//! [`systemprompt_security::authz::AuthzDecisionHook`] as an HTTP endpoint.
//!
//! Core's gateway and MCP enforcement sites POST an [`AuthzRequest`] here;
//! this handler loads the matching rules from `access_control_rules`, runs
//! the pure deny-overrides resolver, audits the decision to
//! `governance_decisions`, and returns an [`AuthzDecision`] for core to act
//! on. The audit row's `policy` is `authz` regardless of `entity_type`, so
//! `infra logs audit` can correlate gateway and MCP decisions in one stream.
//!
//! The resolver runs over core's `user` / `role` dimensions plus every subject
//! dimension this extension declares in [`systemprompt_web_admin_core::authz`]
//! — today that means a `group` rule binds here, not just in the access matrix.
//! Subject attributes and roles are looked up per call rather than read off the
//! request, so a group change or a revocation binds on the next call, not on
//! token refresh.
//!
//! Core composes its own `RuleBasedHook` ahead of this one and that hook
//! audits every evaluation as `authz_rule_based`, over the same
//! `access_control_rules` rows. An allow recorded here is therefore the same
//! evaluation written twice: production held 4,695 such pairs, identical in
//! decision and reason, two thirds of `gov_allow` on every conversation. Only
//! outcomes this hook adds — a deny, a warn, a hold it had to refuse, or an
//! unavailable resolver — are audited, so no refusal loses its row and no
//! reader counts one evaluation as two.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::borrow::Cow;
use systemprompt::identifiers::{Actor, ContextId};

use systemprompt_security::authz::{
    AccessControlRepository, AccessRule, AuthzDecision, AuthzRequest, ChainSources, Decision,
    DecisionTag, DenyReason, EntityRow, ParentChainIndex, ResolveBase, SubjectAttributes,
};

use systemprompt_security::authz::{GovernanceDecisionRecord, insert_governance_decision};
use systemprompt_web_admin_core::authz::{dimensions, subject_attributes_for};
use systemprompt_web_admin_core::repositories::principal::identity::{
    IdentityEnvelope, find_identity_envelope,
};

const POLICY_NAME: &str = "authz";

// Why: core's hook caller reads a non-200 status as "hook unavailable" and
// lets the call through, so every failure here answers 200 with a deny body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Unavailable;

impl IntoResponse for Unavailable {
    fn into_response(self) -> Response {
        Json(AuthzDecision::Deny {
            reason: unavailable_reason(),
            policy: POLICY_NAME.into(),
        })
        .into_response()
    }
}

fn unavailable_reason() -> DenyReason {
    DenyReason::PolicyViolation {
        policy: "authorization_unavailable".into(),
        detail: Cow::Borrowed("Authorization temporarily unavailable; retry later"),
    }
}

struct Subject {
    rules: Vec<AccessRule>,
    entity: Option<EntityRow>,
    chains: ParentChainIndex,
    attributes: SubjectAttributes,
    identity: IdentityEnvelope,
}

struct DecisionAudit<'a> {
    id: &'a str,
    req: &'a AuthzRequest,
    tag: DecisionTag,
    reason: &'a str,
    // JSON: variable-shape governance audit evaluated_rules payload
    evaluated: &'a serde_json::Value,
    context_id: &'a ContextId,
    actor: &'a Actor,
}

async fn audit_unavailable(pool: &PgPool, req: &AuthzRequest) {
    tracing::error!(user_id = %req.user_id, trace_id = %req.trace_id, entity = %req.entity, "authorization_unavailable");
    let decision = Decision::Deny {
        reason: unavailable_reason(),
    };
    audit_decision(pool, req, &[], None, &decision).await;
}

async fn parent_index(repo: &AccessControlRepository) -> Result<ParentChainIndex, Unavailable> {
    let services = systemprompt::loader::ServicesBootstrap::get().map_err(|error| {
        tracing::error!(%error, "authorization_unavailable: services");
        Unavailable
    })?;
    ParentChainIndex::load(repo, Arc::new(ChainSources::from_services(services)))
        .await
        .map_err(|error| {
            tracing::error!(%error, "authorization_unavailable: parent policies");
            Unavailable
        })
}

async fn load_rules(
    repo: &AccessControlRepository,
    req: &AuthzRequest,
) -> Result<(Vec<AccessRule>, Option<EntityRow>), Unavailable> {
    let kind = req.entity.kind();
    let id = req.entity.id_str();
    let rules = repo
        .list_rules_for_entity(kind, id)
        .await
        .map_err(|error| {
            tracing::error!(%error, %id, "authorization_unavailable: entity rules");
            Unavailable
        })?;
    let entity = repo.get_entity(kind, id).await.map_err(|error| {
        tracing::error!(%error, %id, "authorization_unavailable: entity");
        Unavailable
    })?;
    Ok((rules, entity))
}

async fn read_subject(
    pool: &PgPool,
    repo: &AccessControlRepository,
    req: &AuthzRequest,
) -> Result<Subject, Unavailable> {
    let (rules, entity) = load_rules(repo, req).await?;
    let chains = parent_index(repo).await?;
    let attributes = subject_attributes_for(pool, &req.user_id)
        .await
        .map_err(|error| {
            tracing::error!(%error, user_id = %req.user_id, trace_id = %req.trace_id, "authorization_unavailable: attributes");
            Unavailable
        })?;
    let identity = find_identity_envelope(pool, &req.user_id)
        .await
        .ok()
        .flatten()
        .filter(|identity| identity.status == "active")
        .ok_or(Unavailable)?;
    Ok(Subject {
        rules,
        entity,
        chains,
        attributes,
        identity,
    })
}

async fn load_subject(
    pool: &PgPool,
    repo: &AccessControlRepository,
    req: &AuthzRequest,
) -> Result<Subject, Unavailable> {
    let subject = read_subject(pool, repo, req).await;
    if subject.is_err() {
        audit_unavailable(pool, req).await;
    }
    subject
}

fn audit_tag(decision: &Decision) -> Option<(DecisionTag, String)> {
    match decision {
        Decision::Allow { .. } => None,
        Decision::Warn { reason } => Some((DecisionTag::Warn, reason.to_string())),
        Decision::Deny { reason } => Some((DecisionTag::Deny, reason.to_string())),
        Decision::Pending { reason } => {
            tracing::error!(
                %reason,
                "a governance hold reached the PreToolUse webhook, which answers \
                 synchronously and cannot park a call; refusing it"
            );
            Some((DecisionTag::Deny, reason.to_string()))
        },
    }
}

fn context_joining_session(req: &AuthzRequest) -> ContextId {
    req.context_id.clone().unwrap_or_else(|| {
        req.session_id
            .as_ref()
            .map_or_else(ContextId::generate, ContextId::derived_from_session)
    })
}

fn decision_record<'a>(audit: &'a DecisionAudit<'a>) -> GovernanceDecisionRecord<'a> {
    let req = audit.req;
    GovernanceDecisionRecord {
        id: audit.id,
        actor: audit.actor,
        session_id: req.session_id.as_ref(),
        tool_name: req.entity.id_str(),
        agent_id: None,
        agent_scope: None,
        decision: audit.tag,
        policy: POLICY_NAME,
        reason: audit.reason,
        evaluated_rules: audit.evaluated,
        client_id: req.client_id.as_ref(),
        plugin_id: None,
        tool_use_id: None,
        act_chain: &req.act_chain,
        trace_id: Some(&req.trace_id),
        context_id: &audit.context_id,
        task_id: req.task_id.as_ref(),
    }
}

async fn audit_decision(
    pool: &PgPool,
    req: &AuthzRequest,
    rules: &[AccessRule],
    entity: Option<&EntityRow>,
    decision: &Decision,
) {
    let Some((tag, reason)) = audit_tag(decision) else {
        return;
    };
    let id = uuid::Uuid::new_v4().to_string();
    // JSON: variable-shape governance audit evaluated_rules payload
    let evaluated = serde_json::json!({
        "entity_type": req.entity.kind().as_str(),
        "entity_id": req.entity.id_str(),
        "trace_id": req.trace_id.as_str(),
        "roles": req.roles,
        "attributes": req.attributes,
        "context": req.context,
        "entity": entity,
        "rules": rules,
    });
    let actor = Actor::user(req.user_id.clone());
    let context_id = context_joining_session(req);
    let audit = DecisionAudit {
        id: &id,
        req,
        tag,
        reason: &reason,
        evaluated: &evaluated,
        context_id: &context_id,
        actor: &actor,
    };
    if let Err(e) = insert_governance_decision(pool, &decision_record(&audit)).await {
        tracing::error!(error = %e, "Failed to record authz decision");
    }
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

pub(crate) async fn govern_authz(
    State(pool): State<Arc<PgPool>>,
    Json(mut req): Json<AuthzRequest>,
) -> Result<Response, Unavailable> {
    let repo = AccessControlRepository::from_pool(Arc::clone(&pool));
    let subject = load_subject(&pool, &repo, &req).await?;
    req.roles = subject.identity.roles;
    let decision = subject.chains.resolve(
        req.entity.kind(),
        req.entity.id_str(),
        ResolveBase {
            rules: &subject.rules,
            user_id: &req.user_id,
            user_roles: &req.roles,
            default_included: subject.entity.as_ref().map(|e| e.default_included),
            attributes: &subject.attributes,
            dimensions: dimensions(&pool),
        },
    );
    audit_decision(
        &pool,
        &req,
        &subject.rules,
        subject.entity.as_ref(),
        &decision,
    )
    .await;
    Ok((StatusCode::OK, Json(sync_verdict(decision))).into_response())
}
