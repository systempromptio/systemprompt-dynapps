//! The two writes a person makes to their own account.
//!
//! Everything else under `/api/public/admin` is role-gated, because it mutates
//! somebody else's identity or the instance's configuration. These two are not:
//! they are scoped to the caller, and the target is read from the validated
//! session rather than from the request, so there is no parameter to point at
//! another account. That is why they sit in their own module and their own
//! router tier instead of being widened into the admin one.
//!
//! Closing an account requires echoing the session email, so a bodyless DELETE
//! cannot close one by accident; the settings row (no foreign key) is deleted
//! first, and `ai_requests` is kept as the audit record.

use std::sync::Arc;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;

use crate::repositories;
use crate::repositories::users::user_settings::UserSettingsInput;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

const TIMEZONE_MAX: usize = 64;

fn validated(input: &UserSettingsInput) -> Result<(), AdminError> {
    let tz = input.timezone.trim();
    let shaped = !tz.is_empty()
        && tz.len() <= TIMEZONE_MAX
        && tz
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '+'));
    if !shaped {
        return Err(AdminError::BadRequest(format!(
            "{tz:?} is not shaped like an IANA timezone name."
        )));
    }
    for (field, value) in [
        ("display_name", input.display_name.as_deref()),
        ("avatar_url", input.avatar_url.as_deref()),
    ] {
        if value.is_some_and(|v| v.chars().count() > 512) {
            return Err(AdminError::BadRequest(format!("{field} is too long.")));
        }
    }
    Ok(())
}

pub(crate) async fn update_own_settings_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    body: Result<Json<UserSettingsInput>, JsonRejection>,
) -> AdminResult<Response> {
    let Json(input) = body.map_err(|e| AdminError::BadRequest(e.body_text()))?;
    validated(&input)?;
    let saved =
        repositories::users::user_settings::update_user_settings(&pool, &user_ctx.user_id, &input)
            .await?;
    Ok(Json(saved).into_response())
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeleteAccountRequest {
    confirm_email: String,
}

pub(crate) async fn delete_own_account_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    body: Result<Json<DeleteAccountRequest>, JsonRejection>,
) -> AdminResult<Response> {
    let Json(request) = body.map_err(|e| {
        AdminError::BadRequest(format!(
            "Deleting an account needs {{\"confirm_email\": \"…\"}} naming it: {}",
            e.body_text()
        ))
    })?;
    if !request
        .confirm_email
        .eq_ignore_ascii_case(user_ctx.email.as_str())
    {
        return Err(AdminError::BadRequest(
            "confirm_email does not match the signed-in account.".to_owned(),
        ));
    }
    repositories::users::user_settings::delete_user_settings(&pool, &user_ctx.user_id).await?;
    if !repositories::users::mutations::archive_user(
        &pool,
        &user_ctx.user_id,
        &user_ctx.user_id,
        "deleted by the account holder",
    )
    .await?
    {
        return Err(AdminError::NotFound("No such user.".to_owned()));
    }
    repositories::users::revocation::revoke_user_access(&pool, &user_ctx.user_id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
