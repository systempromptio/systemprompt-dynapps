//! `/hooks/track` ingestion: the endpoint Claude Code hooks post events to.
//!
//! The handler records the event and updates the session's rollups; nothing
//! here calls a model. Session scoring is the `session_evaluation` job, which
//! reads these rows once the session has closed.

mod artifact;
mod auth;
mod dedup;
mod description;
mod entity;
mod helpers;
mod processing;

use crate::error::{AdminError, AdminResult};
use crate::event_hub::EventHub;
use crate::repositories::marketplace::webhook;
use crate::types::webhook::HookEventPayload;
use auth::extract_and_validate_jwt;
use axum::Json;
use axum::extract::rejection::ExtensionRejection;
use axum::extract::{Extension, FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sha2::Digest;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{PluginId, SessionId, UserId};

#[derive(Debug, Clone)]
pub(crate) struct TrackDeps {
    managed: systemprompt::marketplace::managed::ManagedRepository,
    event_hub: EventHub,
    artifact_ingest: Arc<systemprompt::mcp::ArtifactIngest>,
}

impl<S: Send + Sync> FromRequestParts<S> for TrackDeps {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(managed) = Extension::from_request_parts(parts, state).await?;
        let Extension(event_hub) = Extension::from_request_parts(parts, state).await?;
        let Extension(artifact_ingest) = Extension::from_request_parts(parts, state).await?;
        Ok(Self {
            managed,
            event_hub,
            artifact_ingest,
        })
    }
}

pub(crate) async fn handle_hook_track(
    TrackDeps {
        managed,
        event_hub,
        artifact_ingest,
    }: TrackDeps,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    // JSON: protocol boundary — the third-party hook envelope, parsed into typed
    // events by `HookEventPayload::from_value` after the raw copy is retained
    Json(mut raw): Json<serde_json::Value>,
) -> AdminResult<Response> {
    let (user_id, plugin_id) = extract_and_validate_jwt(&headers)?;
    tracing::trace!(payload = %helpers::sanitize_metadata(&raw), "Hook track received payload");
    if let Some(event_id) = headers
        .get("x-ingestion-event-id")
        .and_then(|v| v.to_str().ok())
        && let Some(object) = raw.as_object_mut()
    {
        object
            .entry("event_id")
            .or_insert_with(|| serde_json::Value::String(event_id.to_owned()));
    }
    let (payload, warnings) = HookEventPayload::from_value(raw);
    payload
        .validate_ingestion()
        .map_err(AdminError::BadRequest)?;
    payload.delivery_id().map_err(AdminError::BadRequest)?;
    log_payload_warnings(&payload, &warnings);

    let device = auth::authenticated_device(&managed, &headers, &user_id).await?;
    let was_inserted =
        insert_hook_event(&pool, &user_id, &plugin_id, &payload, device.as_ref()).await?;
    if !was_inserted {
        tracing::trace!(
            plugin_id = %plugin_id,
            event_type = payload.event_name(),
            "Hook event deduplicated"
        );
        return Ok(StatusCode::OK.into_response());
    }

    artifact::ingest_tool_result(artifact::ToolResultIngestion {
        pool: &pool,
        ingest: &artifact_ingest,
        user_id: &user_id,
        payload: &payload,
        host: headers
            .get("x-systemprompt-host")
            .and_then(|value| value.to_str().ok()),
    })
    .await;
    dispatch_inserted_event(&DispatchContext {
        pool: &pool,
        user_id: &user_id,
        plugin_id: &plugin_id,
        payload: &payload,
        event_hub: &event_hub,
    })
    .await;
    Ok(StatusCode::OK.into_response())
}

fn log_payload_warnings(payload: &HookEventPayload, warnings: &[String]) {
    for w in warnings {
        tracing::debug!(
            event_type = payload.event_name(),
            warning = %w,
            "Hook payload validation warning"
        );
    }
}

struct DispatchContext<'a> {
    pool: &'a PgPool,
    user_id: &'a UserId,
    plugin_id: &'a PluginId,
    payload: &'a HookEventPayload,
    event_hub: &'a EventHub,
}

async fn dispatch_inserted_event(ctx: &DispatchContext<'_>) {
    let content_bytes = helpers::compute_content_bytes(ctx.payload);
    processing::process_inserted_event(&processing::ProcessInsertedEventParams {
        pool: ctx.pool,
        user_id: ctx.user_id,
        plugin_id: ctx.plugin_id,
        session_id: &SessionId::new(ctx.payload.session_id()),
        event_type: ctx.payload.event_name(),
        tool_name: ctx.payload.tool_name(),
        content_input_bytes: content_bytes.input,
        content_output_bytes: content_bytes.output,
        payload: ctx.payload,
        event_hub: ctx.event_hub,
    })
    .await;
}

async fn insert_hook_event(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &PluginId,
    payload: &HookEventPayload,
    device: Option<&webhook::AuthenticatedHookDevice>,
) -> AdminResult<bool> {
    let session_id = SessionId::new(payload.session_id());
    let description = description::generate_description(payload);
    let prompt_preview = helpers::generate_prompt_preview(payload);
    let dedup_key =
        dedup::compute_dedup_key(user_id, &session_id, payload).map_err(AdminError::BadRequest)?;
    let content_bytes = helpers::compute_content_bytes(payload);
    let mut sanitized_metadata = helpers::sanitize_metadata(&payload.raw);
    let mut content = payload.raw.clone();
    if let Some(object) = content.as_object_mut() {
        object.remove("event_id");
    }
    let digest = hex::encode(sha2::Sha256::digest(
        serde_json::to_vec(&content).map_err(AdminError::internal)?,
    ));
    if let Some(object) = sanitized_metadata.as_object_mut() {
        // JSON: fixed-shape metadata added at the inbound hook protocol boundary.
        object.insert(
            "_ingestion_digest".to_owned(),
            serde_json::Value::String(digest),
        );
    }

    let usage_params = webhook::UsageEventParams {
        plugin_id,
        user_id,
        session_id: &session_id,
        event_type: payload.event_name(),
        tool_name: payload.tool_name(),
        metadata: &sanitized_metadata,
        description: Some(&*description),
        prompt_preview: prompt_preview.as_deref(),
        cwd: payload.cwd(),
        dedup_key: &dedup_key,
        content_input_bytes: content_bytes.input,
        content_output_bytes: content_bytes.output,
        tool_use_id: payload.tool_use_id(),
    };

    webhook::insert_plugin_usage_event_with_device(pool, &usage_params, device).await
}
