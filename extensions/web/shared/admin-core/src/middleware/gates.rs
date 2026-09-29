//! Authentication and authorisation layers for the admin plane.
//!
//! These sit above `user_context_middleware`, which is what populates the
//! [`UserContext`] they read. They are separated from page context because
//! they answer a different question: context decides what a page renders,
//! these decide whether the request is allowed to reach one at all.
//!
//! The JSON API has three tiers: any signed-in user, the scoped console (an
//! admin or a marketplace participant, with each handler narrowing writes to
//! the caller's participation), and role-gated routes whose accepted roles the
//! router names at the mount point. A signed-in non-admin browsing the SSR
//! console reaches only their own account pages, plus — as a participant — the
//! per-person analysis tabs and the detail pages under a listing, each of
//! which 404s an owner outside scope.

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::request::ErrorBody;
use crate::types::{Role, UserContext, has_any};

// Why: axum `nest_service` strips its prefix from `request.uri()`, so a layer
// inside the admin SSR router sees `/profile` for `/admin/profile`; matching
// user-facing paths requires `OriginalUri`.
fn original_path(request: &Request) -> String {
    request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map_or_else(
            || request.uri().path().to_owned(),
            |o| o.0.path().to_owned(),
        )
}

// Why: the bridge CLI's `/bridge/device-link` carries its loopback callback in
// `?redirect=`; dropping the query strands the CLI on a callback that never
// arrives.
fn original_target(request: &Request) -> String {
    fn render(uri: &axum::http::Uri) -> String {
        uri.path_and_query()
            .map_or_else(|| uri.path().to_owned(), ToString::to_string)
    }
    request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map_or_else(|| render(request.uri()), |o| render(&o.0))
}

pub async fn require_user_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => {
            let target = urlencoding::encode(&original_target(&request)).into_owned();
            let redirect_url = format!("/admin/login?redirect={target}");
            axum::response::Redirect::temporary(&redirect_url).into_response()
        },
    }
}

pub async fn require_auth_middleware(request: Request, next: Next) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    match user_ctx {
        Some(ctx) if !ctx.user_id.as_str().is_empty() => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            axum::Json(ErrorBody {
                error: "Authentication required".to_owned(),
            }),
        )
            .into_response(),
    }
}

pub async fn require_scoped_console_middleware(request: Request, next: Next) -> Response {
    let allowed = request
        .extensions()
        .get::<UserContext>()
        .is_some_and(UserContext::has_scoped_console);
    if allowed {
        return next.run(request).await;
    }
    (
        StatusCode::FORBIDDEN,
        axum::Json(ErrorBody {
            error: "Console access or marketplace participation required".to_owned(),
        }),
    )
        .into_response()
}

pub async fn require_roles_middleware(
    State(accepted): State<&'static [Role]>,
    request: Request,
    next: Next,
) -> Response {
    let user_ctx = request.extensions().get::<UserContext>().cloned();
    let allowed = user_ctx.is_some_and(|ctx| has_any(&ctx.roles, accepted));
    if allowed {
        return next.run(request).await;
    }
    let names = accepted
        .iter()
        .map(|r| r.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    (
        StatusCode::FORBIDDEN,
        axum::Json(ErrorBody {
            error: format!("Role required: {names}"),
        }),
    )
        .into_response()
}

pub async fn non_admin_gate_middleware(request: Request, next: Next) -> Response {
    let path = original_path(&request);
    let path = path.as_str();
    let user_ctx = request.extensions().get::<UserContext>().cloned();

    let Some(ctx) = user_ctx else {
        return next.run(request).await;
    };
    if ctx.is_console || ctx.user_id.as_str().is_empty() {
        return next.run(request).await;
    }

    if is_non_admin_allowed_path(path, !ctx.participation.is_empty()) {
        next.run(request).await
    } else {
        axum::response::Redirect::to("/admin/profile").into_response()
    }
}

#[derive(Clone, Copy)]
enum PathMatch {
    Exact,
    Prefix,
    Child,
}

const ACCOUNT_PATHS: &[(PathMatch, &str)] = &[
    (PathMatch::Prefix, "/admin/profile"),
    (PathMatch::Prefix, "/admin/history"),
    (PathMatch::Prefix, "/admin/export/history"),
    (PathMatch::Prefix, "/admin/export/transcripts"),
    (PathMatch::Exact, "/admin/connect"),
    (PathMatch::Exact, "/admin/connectors"),
    (PathMatch::Prefix, "/admin/auth/"),
    (PathMatch::Prefix, "/admin/api/"),
    (PathMatch::Exact, "/admin/logout"),
    (PathMatch::Exact, "/admin/login"),
    (PathMatch::Exact, "/admin/setup"),
    (PathMatch::Exact, "/admin/"),
    (PathMatch::Exact, "/admin"),
];

const PARTICIPANT_PATHS: &[(PathMatch, &str)] = &[
    (PathMatch::Exact, "/admin/analysis/conversations"),
    (PathMatch::Prefix, "/admin/analysis/versions"),
    (PathMatch::Prefix, "/admin/export/analysis-conversation"),
    (PathMatch::Prefix, "/admin/export/analysis-marketplaces"),
    (PathMatch::Prefix, "/admin/export/analysis-versions"),
    (PathMatch::Exact, "/admin/sync"),
    (PathMatch::Child, "/admin/sessions/"),
    (PathMatch::Child, "/admin/contexts/"),
    (PathMatch::Child, "/admin/traces/"),
    (PathMatch::Child, "/admin/requests/"),
];

fn matches(entries: &[(PathMatch, &str)], path: &str) -> bool {
    entries.iter().any(|(how, pattern)| match how {
        PathMatch::Exact => path == *pattern,
        PathMatch::Prefix => path.starts_with(pattern),
        PathMatch::Child => path
            .strip_prefix(pattern)
            .is_some_and(|tail| !tail.is_empty() && !tail.contains('/')),
    })
}

fn is_non_admin_allowed_path(path: &str, participant: bool) -> bool {
    matches(ACCOUNT_PATHS, path) || (participant && matches(PARTICIPANT_PATHS, path))
}
