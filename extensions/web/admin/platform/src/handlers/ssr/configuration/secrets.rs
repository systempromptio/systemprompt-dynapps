//! The secrets card on `/admin/configuration` and its reload action.
//!
//! The card shows where secrets come from and how fresh they are — source,
//! generation, load and reload times, the last reload error, and the key
//! names. It never renders a value. **Reload now** re-reads the source
//! through core's secrets store and leaves an activity row
//! naming the keys that changed.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::response::Redirect;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::traits::SecretStore;

use super::BASE_URL;
use systemprompt_web_admin_core::activity::{NewActivity, record};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::request::require_write_origin;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::flash::{FlashTone, redirect_with_flash};
use systemprompt_web_admin_core::types::UserContext;

pub(crate) const RELOAD_URL: &str = "/admin/platform/secrets/reload";
const DETAIL_MAX: usize = 240;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SecretsCardView {
    pub source: String,
    pub generation: u64,
    pub initialised_at: DateTime<Utc>,
    pub last_reload_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub refresh_secs: Option<u64>,
    pub key_count: usize,
    pub key_names: Vec<String>,
    pub reload_url: &'static str,
}

pub(crate) fn secrets_card(secrets: &dyn SecretStore, faults: &mut Faults) -> SecretsCardView {
    let status = secrets.status();
    let mut key_names = faults.take("secret key names", secrets.key_names());
    key_names.sort_unstable();
    SecretsCardView {
        source: status.source,
        generation: status.generation,
        initialised_at: status.initialised_at,
        last_reload_at: status.last_reload_at,
        last_error: status.last_error,
        refresh_secs: secrets.refresh_interval().map(|d| d.as_secs()),
        key_count: key_names.len(),
        key_names,
        reload_url: RELOAD_URL,
    }
}

pub(crate) async fn secrets_reload(
    Extension(user_ctx): Extension<UserContext>,
    Extension(boot): Extension<ExtensionBoot>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
) -> AdminHtmlResult<Redirect> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    require_write_origin(&boot, &headers)?;
    Ok(match boot.secrets().reload().await {
        Ok(outcome) => {
            record(
                &pool,
                NewActivity::secrets_reloaded(
                    &user_ctx.user_id,
                    outcome.generation,
                    &outcome.changed_keys,
                ),
            )
            .await;
            tracing::info!(requested_by = %user_ctx.user_id, generation = outcome.generation, changed_keys = ?outcome.changed_keys, "Secrets reloaded from the console");
            let detail = if outcome.changed_keys.is_empty() {
                format!("generation {}, no keys changed", outcome.generation)
            } else {
                format!(
                    "generation {}, changed: {}",
                    outcome.generation,
                    outcome.changed_keys.join(", ")
                )
            };
            flash(FlashTone::Ok, "Secrets reloaded", &detail)
        },
        Err(error) => {
            tracing::warn!(requested_by = %user_ctx.user_id, error = %error, "Secrets reload failed");
            flash(FlashTone::Err, "Secrets reload failed", &error.to_string())
        },
    })
}

fn flash(tone: FlashTone, title: &str, detail: &str) -> Redirect {
    let detail: String = detail.chars().take(DETAIL_MAX).collect();
    redirect_with_flash(BASE_URL, tone, &format!("{title}: {detail}"))
}
