//! Admin plane request middleware: session resolution and page context.
//!
//! `user_context_middleware` runs first and puts a [`UserContext`] on the
//! request — from the session cookie, or from a personal access token on the
//! export routes (`pat`); `systemprompt_web_admin_core::middleware::gates`
//! then decides whether the request may proceed, and
//! `marketplace_context_middleware` supplies what a page needs to render.
//!
//! The marketplace counts injected into every render are cached because they
//! are derived from a remote catalog and are identical for every user holding
//! the same role set.

use std::sync::Arc;

use axum::Extension;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use systemprompt::database::PgPool;
use systemprompt::identifiers::{Email, UserId};

use systemprompt_web_admin_identity::repositories::users::user_settings::find_user_settings;

use systemprompt_web_admin_core::auth::session::extract_user_from_cookie;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::scope::Participation;
use systemprompt_web_admin_core::ssr::viewer_tz::{Tz, parse_tz, with_viewer_tz};
use systemprompt_web_admin_core::types::{
    UserContext, roles_grant_console, roles_grant_developer, roles_grant_manage,
    roles_grant_platform,
};

use super::pat;

#[derive(Debug, Serialize)]
struct AuthMeResponse {
    user_id: UserId,
    username: String,
    email: Email,
    roles: Vec<String>,
    group_ids: Vec<String>,
    project_ids: Vec<String>,
    is_admin: bool,
    is_console: bool,
    is_platform_admin: bool,
}

pub(super) async fn user_context_middleware(
    State(pool): State<Arc<PgPool>>,
    mut request: Request,
    next: Next,
) -> Result<Response, pat::PatRejection> {
    let cookie = extract_user_from_cookie(request.headers());
    let pat = cookie
        .is_err()
        .then(|| pat::pat_request(&request))
        .flatten();
    let session = match cookie {
        Ok(s) => s,
        Err(reason) => match pat::pat_session(&pool, pat).await {
            pat::PatSession::Accepted(s) => s,
            pat::PatSession::Rejected(rejection) => return Err(rejection),
            pat::PatSession::NotApplicable => {
                tracing::warn!(reason = %reason, "UserContext middleware: no valid session");
                return Ok(next.run(request).await);
            },
        },
    };

    let profile = find_access_profile(&pool, &session.user_id).await;
    let (roles, group_ids, project_ids) = profile.map_or_else(
        || (vec!["user".to_owned()], Vec::new(), Vec::new()),
        |p| (p.roles, p.group_ids, p.project_ids),
    );

    let is_console = roles_grant_console(&roles);
    let participation = if is_console {
        Participation::default()
    } else {
        find_participation(&pool, &session.user_id, &group_ids, &project_ids).await
    };

    let ctx = UserContext {
        user_id: session.user_id,
        username: session.username,
        email: session.email,
        is_admin: roles_grant_manage(&roles),
        is_console,
        is_platform_admin: roles_grant_platform(&roles),
        is_developer: roles_grant_developer(&roles),
        roles,
        group_ids,
        project_ids,
        participation,
        email_verified: false,
        session_id: session.session_id,
    };

    let tz = find_viewer_tz(&pool, &ctx.user_id).await;
    request.extensions_mut().insert(ctx);
    Ok(with_viewer_tz(tz, next.run(request)).await)
}

async fn find_viewer_tz(pool: &PgPool, user_id: &UserId) -> Tz {
    match find_user_settings(pool, user_id).await {
        Ok(row) => row.map_or(Tz::UTC, |r| parse_tz(&r.timezone)),
        Err(e) => {
            tracing::warn!(error = %e, "user settings read failed; rendering times in UTC");
            Tz::UTC
        },
    }
}

async fn find_access_profile(
    pool: &PgPool,
    user_id: &UserId,
) -> Option<systemprompt_web_admin_core::repositories::principal::role::UserAccessProfile> {
    systemprompt_web_admin_core::repositories::principal::role::find_user_access_profile(
        pool, user_id,
    )
    .await
    // Why: fail-soft-ok: an unreadable profile degrades to the plain `user`
    // role, so the gates fail closed rather than failing every request.
    .inspect_err(|e| tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user roles"))
    .ok()
    .flatten()
}

async fn find_participation(
    pool: &PgPool,
    user_id: &UserId,
    group_ids: &[String],
    project_ids: &[String],
) -> Participation {
    systemprompt_web_admin_core::repositories::scope::participation::get_participation(
        pool, user_id, group_ids, project_ids,
    )
    .await
    // Why: fail-soft-ok: unresolved participation grants no scoped access, so
    // the console fails closed rather than refusing the request outright.
    .inspect_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to resolve marketplace participation");
    })
    .unwrap_or_default()
}

pub(super) async fn auth_me_handler(
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    if user_ctx.user_id.as_str().is_empty() {
        return Err(AdminError::Unauthorized("Not authenticated".into()));
    }
    Ok(axum::Json(AuthMeResponse {
        user_id: user_ctx.user_id,
        username: user_ctx.username,
        email: user_ctx.email,
        roles: user_ctx.roles,
        group_ids: user_ctx.group_ids,
        project_ids: user_ctx.project_ids,
        is_admin: user_ctx.is_admin,
        is_console: user_ctx.is_console,
        is_platform_admin: user_ctx.is_platform_admin,
    })
    .into_response())
}
