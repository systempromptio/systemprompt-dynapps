//! `GET /drive/handoff?code=…&path=…`: spend a handoff code for a web
//! session.
//!
//! The code is deleted as it is read, live or not. A live one names its
//! person, path and sign-in time; core's `mint_browser_session` mints the web
//! session with that sign-in time, the cookie is set by core's own
//! `session_cookie_redirect` (the SSO callback's cookie, `SameSite=Lax`, with
//! the same analytics), the login is recorded as a `DynWork` handoff, and the
//! answer is a `303` to the code's own path. Anything else, an unknown,
//! spent or expired code, a person no longer active, a failed mint, is the
//! same `303` to the web login with the query's `path` as `redirect`, as the
//! developer login answers every failure the same way.

use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use systemprompt::analytics::SessionAnalyticsBuilder;
use systemprompt::api::routes::oauth::cookie::session_cookie_redirect;
use systemprompt::api::services::session_cookie::SessionCookie;
use systemprompt::sso::{BrowserSessionSubject, mint_browser_session};
use systemprompt_web_admin_core::activity::NewActivity;
use systemprompt_web_admin_core::repositories::user_activity::record;

use super::{Handoff, drive_path, hash_code};
use crate::repositories::drive_handoffs::{self, DriveHandoff};

const SIGN_IN: &str = "/admin/login";
const DRIVE_HOME: &str = "/drive";

#[derive(Debug, Default, Deserialize)]
pub(super) struct RedeemParams {
    code: Option<String>,
    path: Option<String>,
}

pub(super) async fn redeem(
    State(handoff): State<Handoff>,
    headers: HeaderMap,
    params: Result<Query<RedeemParams>, QueryRejection>,
) -> Response {
    let params = params.map(|Query(params)| params).unwrap_or_default();
    let config = handoff.0.boot.config();
    let back = params
        .path
        .as_deref()
        .and_then(|raw| drive_path(config.api_external_url.trim_end_matches('/'), raw))
        .unwrap_or_else(|| DRIVE_HOME.to_owned());
    let Some(code) = params.code.filter(|code| !code.is_empty()) else {
        return sign_in(&back);
    };
    let spent = drive_handoffs::delete_drive_handoff(&handoff.0.pool, &hash_code(&code)).await;
    let opened = match spent {
        Ok(Some(opened)) => opened,
        Ok(None) => return sign_in(&back),
        Err(error) => {
            tracing::error!(%error, "a session handoff code could not be read");
            return sign_in(&back);
        },
    };
    open(&handoff, &headers, &opened)
        .await
        .unwrap_or_else(|| sign_in(&back))
}

async fn open(handoff: &Handoff, headers: &HeaderMap, opened: &DriveHandoff) -> Option<Response> {
    let parts = &handoff.0;
    let config = parts.boot.config();
    let subject = BrowserSessionSubject {
        user_id: &opened.user_id,
        display_name: &opened.display_name,
        email: &opened.email,
        roles: &opened.roles,
    };
    let analytics = SessionAnalyticsBuilder::new(headers).build();
    let minted = mint_browser_session(
        &parts.session_service,
        config,
        &subject,
        opened.authenticated_at,
        &analytics,
    )
    .await
    .inspect_err(|error| {
        tracing::error!(%error, user_id = %opened.user_id, "a session handoff could not mint the web session");
    })
    .ok()?;
    record(
        &parts.pool,
        NewActivity::dynwork_handoff_login(&opened.user_id, &opened.display_name),
    )
    .await;
    tracing::info!(user_id = %opened.user_id, "a DynWork session handoff opened a web session");
    Some(session_cookie_redirect(
        SessionCookie::from_config(config),
        &minted.access_token,
        &opened.path,
        minted.max_age_secs,
    ))
}

fn sign_in(back: &str) -> Response {
    Redirect::to(&format!("{SIGN_IN}?redirect={}", urlencoding::encode(back))).into_response()
}
