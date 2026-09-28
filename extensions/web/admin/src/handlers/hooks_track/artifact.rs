//! Hands a hook-reported tool result to the artifact ingest.
//!
//! `PostToolUse` is the one place that sees the result exactly as the agent
//! saw it. The response is lifted out of the envelope before the event row is
//! sanitised and handed to core's ingest, which normalises, scans, stores and
//! links it by the client's `tool_use_id`; the event row is then stamped with
//! the execution the ingest resolved. A failure here never fails the hook:
//! the event is already recorded and the gateway will see the same result
//! again as request history.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::{
    Actor, AgentName, AiToolCallId, ContextId, SkillId, TraceId, UserId,
};
use systemprompt::mcp::{ArtifactIngest, IngestRequest, from_hook_failure, from_hook_response};
use systemprompt::models::RequestContext;
use systemprompt::models::auth::UserType;
use systemprompt::models::mcp::ExecutionSource;

use systemprompt_web_admin_core::repositories::ingestion::hooks_track::find_latest_skill_invocation;
use systemprompt_web_admin_core::repositories::ingestion::webhook;
use systemprompt_web_admin_core::types::webhook::{HookEvent, HookEventPayload};

use super::dedup;

pub(super) struct ToolResultIngestion<'a> {
    pub pool: &'a PgPool,
    pub ingest: &'a Arc<ArtifactIngest>,
    pub user_id: &'a UserId,
    pub payload: &'a HookEventPayload,
    pub host: Option<&'a str>,
}

pub(super) async fn ingest_tool_result(params: ToolResultIngestion<'_>) {
    let (tool_name, tool_use_id, input, result) = match &params.payload.event {
        HookEvent::PostToolUse(d) if !d.name.is_empty() => (
            d.name.clone(),
            d.use_id.clone(),
            d.input.clone(),
            from_hook_response(&d.response),
        ),
        HookEvent::PostToolUseFailure(d) if !d.tool_name.is_empty() => (
            d.tool_name.clone(),
            d.tool_use_id.clone(),
            d.tool_input.clone(),
            from_hook_failure(&d.error),
        ),
        _ => return,
    };

    let session_id = systemprompt::identifiers::SessionId::new(params.payload.session_id());
    let ctx = RequestContext::new(
        session_id.clone(),
        TraceId::new(session_id.as_str()),
        ContextId::derived_from_session(&session_id),
        AgentName::unset(),
    )
    .with_actor(Actor::user(params.user_id.clone()))
    .with_user_type(UserType::User);

    let skill = find_latest_skill_invocation(params.pool, &session_id, params.user_id)
        .await
        .map(|reference| (SkillId::new(reference.clone()), reference));

    let (tool_name, server_name) =
        match systemprompt_web_admin_core::util::mcp_tool_name::parse_mcp_tool_name(&tool_name) {
            Some(mcp) => (mcp.tool, Some(mcp.server)),
            None => (tool_name, None),
        };

    let request = IngestRequest {
        result,
        tool_name,
        server_name,
        ai_tool_call_id: (!tool_use_id.is_empty()).then(|| AiToolCallId::new(tool_use_id)),
        mcp_execution_id: None,
        ctx,
        skill,
        source: params.host.map_or(
            ExecutionSource::HookClaudeCode,
            ExecutionSource::from_hook_host,
        ),
        started_at: None,
        input: Some(input),
    };

    let outcome = match params.ingest.ingest(request).await {
        Ok(outcome) => outcome,
        Err(error) => {
            tracing::warn!(%error, "Hook tool result was not ingested as an artifact");
            return;
        },
    };

    let Ok(dedup_key) = dedup::compute_dedup_key(params.user_id, &session_id, params.payload)
    else {
        return;
    };
    if let Err(error) =
        webhook::update_event_execution(params.pool, &dedup_key, outcome.mcp_execution_id.as_str())
            .await
    {
        tracing::warn!(%error, "Hook event could not be linked to its execution");
    }
}
