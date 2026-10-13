//! `POST /admin/farm/runs/{run}/cancel` — ask a live run to stop, then return
//! to the run's page, where its status settles once the runner lets go.

use std::sync::Arc;

use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::flash::{FlashTone, redirect_with_flash};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::error::FarmError;
use crate::handlers::ssr::farm_agent::require_console;
use crate::services::runs::cancel;
use crate::state::FarmState;
use crate::types::FarmRunId;

pub(crate) async fn cancel_run_action(
    shell: Page,
    Extension(state): Extension<Arc<FarmState>>,
    Path(run_id): Path<String>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let run_id = FarmRunId::new(run_id);
    let back = format!("/admin/farm/runs/{}", run_id.as_str());
    let (tone, message) = match cancel(&state, &run_id, &shell.user.user_id).await {
        Ok(true) => (FlashTone::Ok, "Cancel requested".to_owned()),
        Ok(false) => (
            FlashTone::Info,
            "Nothing to cancel: the run has finished or no runner holds it.".to_owned(),
        ),
        Err(FarmError::NotFound(_)) => {
            return Ok(
                redirect_with_flash("/admin/farm/runs", FlashTone::Err, "Run not found")
                    .into_response(),
            );
        },
        Err(FarmError::Validation(reason) | FarmError::Forbidden(reason)) => {
            (FlashTone::Warn, format!("Not cancelled: {reason}"))
        },
        Err(other) => return Err(AdminError::from(other).into()),
    };
    Ok(redirect_with_flash(&back, tone, &message).into_response())
}
