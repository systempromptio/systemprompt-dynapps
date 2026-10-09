//! What [`super::evaluate`] writes to `governance_decisions`.
//!
//! Core composes its own `RuleBasedHook` ahead of this one and that hook
//! audits every evaluation as `authz_rule_based`, over the same
//! `access_control_rules` rows. An allow recorded here is therefore the same
//! evaluation written twice: production held 4,695 such pairs, identical in
//! decision and reason, two thirds of `gov_allow` on every conversation. Only
//! outcomes this hook adds — a deny, a warn, a hold it had to refuse, or an
//! unavailable resolver — are audited, so no refusal loses its row and no
//! reader counts one evaluation as two. The row's `policy` is `authz`
//! whatever the entity type, so `infra logs audit` correlates gateway and MCP
//! decisions in one stream.

use sqlx::PgPool;
use systemprompt::identifiers::{Actor, ContextId};
use systemprompt_security::authz::{
    AccessRule, AuthzRequest, Decision, DecisionTag, EntityRow, GovernanceDecisionRecord,
    insert_governance_decision,
};

pub(crate) const POLICY_NAME: &str = "authz";

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

pub(super) async fn audit_unavailable(pool: &PgPool, req: &AuthzRequest) {
    tracing::error!(user_id = %req.user_id, trace_id = %req.trace_id, entity = %req.entity, "authorization_unavailable");
    let decision = Decision::Deny {
        reason: super::unavailable_reason(),
    };
    audit_decision(pool, req, &[], None, &decision).await;
}

fn audit_tag(decision: &Decision) -> Option<(DecisionTag, String)> {
    match decision {
        Decision::Allow { .. } => None,
        Decision::Warn { reason } => Some((DecisionTag::Warn, reason.to_string())),
        Decision::Deny { reason } => Some((DecisionTag::Deny, reason.to_string())),
        Decision::Pending { reason } => {
            tracing::error!(
                %reason,
                "a governance hold reached the synchronous authz hook, which cannot \
                 park a call; refusing it"
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
        context_id: audit.context_id,
        task_id: req.task_id.as_ref(),
    }
}

pub(super) async fn audit_decision(
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
