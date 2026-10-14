//! The two doors into the Odoo demo: the JSON route for scripts and the
//! console form. Both are admin-only — the JSON route by the manage-tier
//! layer it is merged under, the form by its own check, because console
//! pages admit roles that may not run it.

use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult, AdminResult};
use systemprompt_web_admin_core::ssr::flash::{FlashTone, redirect_with_flash};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::types::UserContext;
use systemprompt_web_admin_farm::state::FarmState;
use systemprompt_web_admin_platform::service_accounts::ServiceAccounts;

use super::start_odoo_demo;

const EVALS_URL: &str = "/admin/farm/evals";

pub(in crate::router::admin) async fn odoo_demo_api(
    Extension(user_ctx): Extension<UserContext>,
    Extension(accounts): Extension<ServiceAccounts>,
    Extension(farm): Extension<Arc<FarmState>>,
) -> AdminResult<Response> {
    let started = start_odoo_demo(&accounts, &farm, &user_ctx.user_id).await?;
    Ok(Json(started).into_response())
}

pub(in crate::router::admin) async fn odoo_demo_action(
    shell: Page,
    Extension(accounts): Extension<ServiceAccounts>,
    Extension(farm): Extension<Arc<FarmState>>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let response = match start_odoo_demo(&accounts, &farm, &shell.user.user_id).await {
        Ok(started) => redirect_with_flash(
            EVALS_URL,
            FlashTone::Ok,
            &format!(
                "Odoo demo started: {} suites running, odoo_destructive held until {}",
                started.eval_runs.len(),
                started.valid_until.format("%H:%M UTC")
            ),
        ),
        Err(
            AdminError::NotFound(reason)
            | AdminError::Unprocessable(reason)
            | AdminError::Upstream(reason),
        ) => redirect_with_flash(
            EVALS_URL,
            FlashTone::Err,
            &format!("Odoo demo not started: {reason}"),
        ),
        Err(other) => return Err(other.into()),
    };
    Ok(response.into_response())
}
