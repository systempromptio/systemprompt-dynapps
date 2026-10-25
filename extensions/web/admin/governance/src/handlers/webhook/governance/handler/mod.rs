//! Governance webhook entrypoint: authenticate, evaluate the policy chain, and
//! record an audit row before returning the `PreToolUse` decision.

mod authn;
mod denial;
mod farm_allowlist;
mod governed;
mod response;
use response::HookVerdict;

use std::sync::Arc;

use axum::extract::rejection::ExtensionRejection;
use axum::extract::{FromRequestParts, Query, State};
use axum::http::HeaderMap;
use axum::http::request::Parts;
use axum::{Extension, Json};
use sqlx::PgPool;
use systemprompt::governance::authz::Decision;
use systemprompt::governance::policy::types::AccessScope;
use systemprompt::governance::policy::{
    AgentScope, AuditOrigin, AuditTarget, DecisionAudit, GovernanceEngine, GovernedInput,
    GovernedTarget, PolicyRequest, PrincipalSnapshot, record_decision,
};
use systemprompt::identifiers::{CallId, ClientId, PluginId, SessionId, UserId};
use systemprompt::oauth::SessionCreationService;

use systemprompt_web_admin_core::repositories::ingestion::usage as ingestion;
use systemprompt_web_admin_core::types::webhook::{GovernQuery, HookEventPayload};

use super::types::AuthDenialParams;

use authn::{authenticate_request, deny_for_auth_failure};
use denial::spawn_auth_denial;
use governed::{governed_input, governed_target};


pub(super) const fn principal_snapshot(
    user_id: UserId,
    session_id: SessionId,
    agent_scope: AccessScope,
    client_id: Option<ClientId>,
) -> PrincipalSnapshot {
    PrincipalSnapshot {
        user_id,
        session_id,
        agent_session: None,
        agent_id: None,
        agent_scope,
        client_id,
    }
}

fn response_event(payload: &HookEventPayload) -> &'static str {
    if payload.prompt().is_some() {
        "UserPromptSubmit"
    } else {
        "PreToolUse"
    }
}

async fn resolve_access_scope(
    pool: &PgPool,
    user_id: &UserId,
    denial: &AuthDenialParams<'_>,
) -> Result<(AccessScope, Vec<String>), &'static str> {
    match systemprompt_web_admin_core::authz::account::account_scope_and_roles(pool, user_id).await
    {
        Ok((scope @ (AccessScope::Admin | AccessScope::User), roles)) => Ok((scope, roles)),
        result => {
            tracing::warn!(?result, %user_id, "authorization_unavailable: active account");
            spawn_auth_denial(denial, "Active account authorization unavailable");
            Err("Active account authorization unavailable")
        },
    }
}

struct Governed<'a> {
    target: &'a GovernedTarget,
    input: &'a GovernedInput,
    session_id: &'a SessionId,
    user_id: UserId,
    client_id: Option<ClientId>,
    access_scope: AccessScope,
    roles: &'a [String],
    plugin_id: Option<&'a PluginId>,
    tool_use_id: Option<&'a str>,
    allowlist_violation: Option<String>,
}

async fn evaluate_and_record(
    pool: &PgPool,
    engine: &GovernanceEngine,
    governed: Governed<'_>,
) -> Result<Decision, &'static str> {
    let call_id = CallId::generate();
    let mut evaluation = engine
        .charge_and_evaluate(PolicyRequest {
            target: governed.target.clone(),
            agent_scope: AgentScope::User {
                user_id: governed.user_id.clone(),
            },
            access_scope: governed.access_scope,
            roles: governed.roles,
            session_id: governed.session_id,
            user_id: &governed.user_id,
            input: governed.input,
            call_id: &call_id,
        })
        .await;
    if let Some(detail) = governed.allowlist_violation {
        farm_allowlist::enforce(&mut evaluation, detail);
    }
    let audit = DecisionAudit {
        id: uuid::Uuid::new_v4().to_string(),
        call_id: call_id.clone(),
        origin: AuditOrigin::Governed,
        decision: evaluation.decision.clone(),
        principal: principal_snapshot(
            governed.user_id,
            governed.session_id.clone(),
            governed.access_scope,
            governed.client_id,
        ),
        target: AuditTarget {
            tool_name: systemprompt::identifiers::McpToolName::new(governed.target.as_str()),
            plugin_id: governed.plugin_id.cloned(),
            tool_use_id: governed.tool_use_id.map(str::to_owned),
        },
        chain: evaluation.chain,
        approver: None,
        act_chain: Vec::new(),
        context_id: Some(systemprompt::identifiers::ContextId::derived_from_session(
            governed.session_id,
        )),
        trace_id: Some(systemprompt::identifiers::TraceId::new(
            governed.session_id.as_str(),
        )),
    };
    record_decision(pool, &audit).await.map_err(|error| {
        tracing::error!(%error, "Governance audit unavailable; denying execution");
        "Governance audit unavailable"
    })?;
    Ok(evaluation.decision)
}

// Why: core injects the gateway's own engine onto every extension router;
// building a second one would give this hook a separate rate-limiter budget.
#[derive(Debug, Clone)]
pub(crate) struct GovernDeps {
    engine: Arc<GovernanceEngine>,
    session_service: Arc<SessionCreationService>,
    tasks: systemprompt::traits::BackgroundTasks,
}

impl<S: Send + Sync> FromRequestParts<S> for GovernDeps {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(engine) = Extension::from_request_parts(parts, state).await?;
        let Extension(session_service) = Extension::from_request_parts(parts, state).await?;
        let Extension(tasks) = Extension::from_request_parts(parts, state).await?;
        Ok(Self {
            engine,
            session_service,
            tasks,
        })
    }
}

pub(crate) async fn govern_tool_use(
    State(pool): State<Arc<PgPool>>,
    GovernDeps {
        engine,
        session_service,
        tasks,
    }: GovernDeps,
    headers: HeaderMap,
    Query(query): Query<GovernQuery>,
    // JSON: protocol boundary — the third-party hook envelope, parsed into typed
    // events by `HookEventPayload::from_value` after the raw copy is retained
    Json(raw): Json<serde_json::Value>,
) -> HookVerdict {
    let (payload, _warnings) = HookEventPayload::from_value(raw);

    let Ok(target) = governed_target(&payload) else {
        return HookVerdict::new(
            deny_for_auth_failure("Invalid tool identifier"),
            response_event(&payload),
        );
    };
    let input = governed_input(&payload);
    let response_event = response_event(&payload);
    let session_id = SessionId::new(payload.session_id());
    let plugin_id = query.plugin_id.as_ref();
    let denial_params = AuthDenialParams {
        pool: &pool,
        session_id: &session_id,
        tool_name: target.as_str(),
        hook_event_name: response_event,
        plugin_id,
        session_service: &session_service,
        headers: &headers,
        tasks: &tasks,
    };

    let principal = match authenticate_request(&headers, &denial_params) {
        Ok(principal) => principal,
        Err(verdict) => return *verdict,
    };
    if let Err(reason) = payload.validate_ingestion() {
        return HookVerdict::new(deny_for_auth_failure(&reason.to_string()), response_event);
    }
    if let Err(error) =
        ingestion::assert_ingestion_owner(&pool, &session_id, &principal.user_id).await
    {
        tracing::error!(%error, "Hook identity binding failed");
        return HookVerdict::new(
            deny_for_auth_failure("Session binding unavailable"),
            response_event,
        );
    }
    let (access_scope, roles) =
        match resolve_access_scope(&pool, &principal.user_id, &denial_params).await {
            Ok(scope) => scope,
            Err(reason) => return HookVerdict::new(deny_for_auth_failure(reason), response_event),
        };

    let allowlist_violation = match farm_allowlist::allowlist_violation(
        &pool,
        principal.gateway_session.as_ref(),
        &target,
    )
    .await
    {
        Ok(violation) => violation,
        Err(reason) => return HookVerdict::new(deny_for_auth_failure(reason), response_event),
    };
    let governed = Governed {
        target: &target,
        input: &input,
        session_id: &session_id,
        user_id: principal.user_id,
        client_id: principal.client_id,
        access_scope,
        roles: &roles,
        plugin_id,
        tool_use_id: payload.tool_use_id(),
        allowlist_violation,
    };
    let decision = evaluate_and_record(&pool, &engine, governed)
        .await
        .unwrap_or_else(deny_for_auth_failure);
    HookVerdict::new(decision, response_event)
}
