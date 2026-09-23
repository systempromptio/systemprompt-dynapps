//! Post-insert processing dispatched once a hook event is persisted.

use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SessionId, UserId};

use crate::event_hub::EventHub;
use crate::repositories::dashboard::{conversation_analytics, hooks_track, usage_aggregations};

use crate::types::webhook::{HookEvent, HookEventPayload};
use crate::types::{ENTITY_SKILL, EVENT_SESSION_END, EVENT_SESSION_START};

use super::{entity, helpers};

#[derive(Debug, Clone, Copy)]
pub(super) struct ProcessInsertedEventParams<'a> {
    pub pool: &'a PgPool,
    pub user_id: &'a UserId,
    pub plugin_id: &'a PluginId,
    pub session_id: &'a SessionId,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub payload: &'a HookEventPayload,
    pub event_hub: &'a EventHub,
}

pub(super) async fn process_inserted_event(params: &ProcessInsertedEventParams<'_>) {
    let pool = params.pool;
    let user_id = params.user_id;
    let session_id = params.session_id;
    let event_type = params.event_type;
    let payload = params.payload;
    let today = chrono::Utc::now().date_naive();

    usage_aggregations::upsert_daily_aggregation(&usage_aggregations::DailyAggregationParams {
        pool,
        user_id,
        plugin_id: params.plugin_id,
        date: &today,
        event_type,
        tool_name: params.tool_name,
        content_input_bytes: params.content_input_bytes,
        content_output_bytes: params.content_output_bytes,
        is_error: matches!(&payload.event, HookEvent::PostToolUseFailure(_)),
    })
    .await;

    let has_session = !session_id.as_str().is_empty();

    if has_session {
        update_session_tracking(params).await;
        track_session_entity(pool, user_id, session_id, payload).await;
    }

    handle_prompt_title(pool, event_type, session_id, payload).await;

    if event_type == EVENT_SESSION_END && has_session {
        handle_session_end(params).await;
    }

    params.event_hub.notify(user_id).await;
}

async fn update_session_tracking(params: &ProcessInsertedEventParams<'_>) {
    if let Some(path) = helpers::extract_file_path(params.payload).filter(|path| !path.is_empty()) {
        usage_aggregations::update_unique_files_touched(params.pool, params.session_id, &path)
            .await;
    }

    if params.event_type == EVENT_SESSION_START
        && let HookEvent::SessionStart(ref data) = params.payload.event
    {
        usage_aggregations::update_session_metadata(
            params.pool,
            params.session_id,
            &data.source,
            &data.model,
            &params.payload.common.permission_mode,
        )
        .await;
    }

    if !params.payload.common.permission_mode.is_empty() && params.event_type != EVENT_SESSION_START
    {
        usage_aggregations::update_session_permission_mode(
            params.pool,
            params.session_id,
            &params.payload.common.permission_mode,
        )
        .await;
    }
}

async fn track_session_entity(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
    payload: &HookEventPayload,
) {
    let Some((entity_type, entity_name)) = entity::detect_entity(payload) else {
        return;
    };
    let entity_id = if entity_type == ENTITY_SKILL {
        Some(
            entity_name
                .rsplit_once(':')
                .map_or(entity_name.as_str(), |(_, slug)| slug)
                .to_owned(),
        )
    } else {
        None
    };
    if let Err(e) = conversation_analytics::upsert_session_entity_link(
        pool,
        user_id,
        conversation_analytics::EntityLinkInput {
            session_id: session_id.as_str(),
            entity_type,
            entity_name: &entity_name,
            entity_id: entity_id.as_deref(),
        },
    )
    .await
    {
        tracing::warn!(error = %e, "Failed to upsert session entity link");
    }
}

async fn handle_prompt_title(
    pool: &PgPool,
    event_type: &str,
    session_id: &SessionId,
    payload: &HookEventPayload,
) {
    if event_type != "UserPromptSubmit" || session_id.as_str().is_empty() {
        return;
    }
    if let HookEvent::UserPromptSubmit(ref data) = payload.event
        && !data.prompt.is_empty()
    {
        let initial_title = helpers::derive_title(&data.prompt);
        usage_aggregations::update_session_title_if_empty(pool, session_id, &initial_title).await;
    }
}

async fn handle_session_end(params: &ProcessInsertedEventParams<'_>) {
    if let Err(e) = hooks_track::mark_session_ended(params.pool, params.session_id).await {
        tracing::warn!(error = %e, session_id = %params.session_id.as_str(), "mark_session_ended failed");
    }
}
