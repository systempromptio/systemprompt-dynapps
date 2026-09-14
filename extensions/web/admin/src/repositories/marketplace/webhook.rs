//! Records plugin usage events emitted over the webhook path.

use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SessionId, UserId};

use crate::error::AdminError;

#[derive(Debug, Clone, Copy)]
pub struct UsageEventParams<'a> {
    pub plugin_id: &'a PluginId,
    pub user_id: &'a UserId,
    pub session_id: &'a SessionId,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    // JSON: arbitrary per-event metadata posted by the plugin hook.
    pub metadata: &'a serde_json::Value,
    pub description: Option<&'a str>,
    pub prompt_preview: Option<&'a str>,
    pub cwd: Option<&'a str>,
    pub dedup_key: &'a str,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub loc_added: i64,
    pub loc_removed: i64,
}

pub async fn insert_plugin_usage_event(
    pool: &PgPool,
    params: &UsageEventParams<'_>,
) -> Result<bool, AdminError> {
    insert_plugin_usage_event_with_device(pool, params, None).await
}

/// Device identity validated independently of the hook JWT and retained with
/// its event.
#[derive(Debug, Clone)]
pub struct AuthenticatedHookDevice {
    pub consumer_id: UserId,
    pub device_id: systemprompt::identifiers::DeviceId,
    pub host: systemprompt::models::feedback::EvaluatorClient,
}

pub async fn insert_plugin_usage_event_with_device(
    pool: &PgPool,
    params: &UsageEventParams<'_>,
    device: Option<&AuthenticatedHookDevice>,
) -> Result<bool, AdminError> {
    if device.is_some_and(|identity| &identity.consumer_id != params.user_id) {
        return Err(AdminError::Forbidden(
            "Hook consumer does not match enrolled device".to_owned(),
        ));
    }
    let mut tx = pool.begin().await.map_err(AdminError::internal)?;
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query!(
        "INSERT INTO plugin_usage_events
            (id, user_id, session_id, event_type, tool_name, metadata, plugin_id,
             description, prompt_preview, cwd, dedup_key,
             content_input_bytes, content_output_bytes, loc_added, loc_removed)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
         ON CONFLICT (dedup_key) WHERE dedup_key IS NOT NULL DO NOTHING",
        &id,
        params.user_id.as_str(),
        params.session_id.as_str(),
        params.event_type,
        params.tool_name,
        params.metadata,
        params.plugin_id.as_str(),
        params.description,
        params.prompt_preview,
        params.cwd,
        params.dedup_key,
        params.content_input_bytes,
        params.content_output_bytes,
        params.loc_added,
        params.loc_removed,
    )
    .execute(&mut *tx)
    .await
    .map_err(|error| {
        if let Some(db) = error.as_database_error() {
            match db.code().as_deref() {
                Some("23505") => {
                    return AdminError::Conflict(
                        "Event ID was already used with different content".to_owned(),
                    );
                },
                Some("23514" | "23503") => {
                    return AdminError::Forbidden("Invalid session ownership".to_owned());
                },
                _ => {},
            }
        }
        tracing::error!(%error, "Event persistence unavailable");
        AdminError::Unavailable("Event was not accepted; retry with the same event ID".to_owned())
    })?;

    if let Some(device) = device {
        let event_id = sqlx::query_scalar!("SELECT id FROM plugin_usage_events WHERE dedup_key=$1 AND user_id=$2 AND session_id=$3", params.dedup_key, params.user_id.as_str(), params.session_id.as_str()).fetch_one(&mut *tx).await.map_err(AdminError::internal)?;
        let host = systemprompt::marketplace::managed::consumer::host_key(device.host);
        sqlx::query!("INSERT INTO feedback_authenticated_events(event_id,consumer_id,device_id,host,native_session_id) VALUES($1,$2,$3,$4,$5) ON CONFLICT(event_id) DO NOTHING",event_id,device.consumer_id.as_str(),device.device_id.as_str(),host,params.session_id.as_str()).execute(&mut *tx).await.map_err(AdminError::internal)?;
        let retained = sqlx::query!("SELECT consumer_id,device_id,host,native_session_id FROM feedback_authenticated_events WHERE event_id=$1",event_id).fetch_one(&mut *tx).await.map_err(AdminError::internal)?;
        if retained.consumer_id != device.consumer_id.as_str()
            || retained.device_id != device.device_id.as_str()
            || retained.host != host
            || retained.native_session_id != params.session_id.as_str()
        {
            return Err(AdminError::Conflict(
                "Hook retry conflicts with authenticated device evidence".to_owned(),
            ));
        }
    }
    tx.commit().await.map_err(AdminError::internal)?;
    Ok(result.rows_affected() > 0)
}
