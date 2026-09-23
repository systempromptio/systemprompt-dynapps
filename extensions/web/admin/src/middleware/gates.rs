//! Authentication and authorisation layers for the admin plane.
//!
//! These sit above `user_context_middleware`, which is what populates the
//! [`UserContext`] they read. They are separated from page context because
//! they answer a different question: context decides what a page renders,
//! these decide whether the request is allowed to reach one at all.

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::handlers::shared::ErrorBody;
use crate::types::{Role, UserContext, has_any};

// Why: `nest_service` strips its prefix from `request.uri()`, so a layer
// inside the admin SSR router sees `/profile` for a request to
// `/admin/profile`; matching user-facing paths requires `OriginalUri`.
fn original_path(request: &Request) -> String {
    request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map_or_else(
            || request.uri().path().to_owned(),
            |o| o.0.path().to_owned(),
        )
}

// Why: `/bridge/device-link` carries the bridge's loopback callback in
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

pub(crate) async fn require_user_middleware(request: Request, next: Next) -> Response {
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

pub(crate) async fn require_auth_middleware(request: Request, next: Next) -> Response {
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

// Why: the scoped-console tier of the JSON API — an admin, or a marketplace
// participant. The handlers behind it narrow every write to the caller's
// participation themselves; this layer only keeps out a caller with neither.
pub(crate) async fn require_scoped_console_middleware(request: Request, next: Next) -> Response {
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

// Why: one gate for all three tiers, taking the accepted roles as state, so
// the router names the tier at the mount point and this file holds no list.
// The message names the roles rather than saying "admin": a project manager
// refused a write route otherwise reads it as a bug in their session.
pub(crate) async fn require_roles_middleware(
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

// Why: anonymous users are deliberately not handled here —
// `require_user_middleware` runs after this layer and owns that case. The path
// must come from `OriginalUri`: this layer sits inside a
// `nest_service("/admin", …)`, which strips the prefix, and every arm of the
// allowlist matches on the full path.
pub(crate) async fn non_admin_gate_middleware(request: Request, next: Next) -> Response {
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
    // Why: the detail page under a listing, never the listing itself —
    // `/admin/sessions/<id>` opens for a scoped caller (the page decides
    // whether the owner is inside their view), `/admin/sessions` does not.
    Child,
}

// Why: the console a signed-in non-admin may reach. Account pages are theirs
// by identity; the rest is the scoped console, open only to a marketplace
// participant and narrowed by that participation inside each handler.
const ACCOUNT_PATHS: &[(PathMatch, &str)] = &[
    (PathMatch::Prefix, "/admin/profile"),
    (PathMatch::Prefix, "/admin/history"),
    // Why: the export of "My conversations" is identity-scoped like the page;
    // every other dataset refuses a caller itself (`export::model::DataSet::allows`).
    (PathMatch::Prefix, "/admin/export/history"),
    // Why: transcripts — one conversation, a set, and the set's count. The
    // one-conversation handler 404s any context the caller may not see; the
    // set handlers refuse a caller without a console seat unless the set is
    // their own history.
    (PathMatch::Prefix, "/admin/export/transcripts"),
    (PathMatch::Exact, "/admin/connect"),
    (PathMatch::Exact, "/admin/connectors"),
    (PathMatch::Prefix, "/admin/requirements/"),
    (PathMatch::Prefix, "/admin/auth/"),
    (PathMatch::Prefix, "/admin/api/"),
    (PathMatch::Exact, "/admin/logout"),
    (PathMatch::Exact, "/admin/login"),
    (PathMatch::Exact, "/admin/setup"),
    (PathMatch::Exact, "/admin/"),
    (PathMatch::Exact, "/admin"),
];

// Why: only the analysis tabs with a per-person dimension and the code sync of
// a participant's own marketplaces; Skills/Usage, Evaluation and Portfolio read
// instance-wide facts and stay admin-only. The `Child` entries are where the
// header id lookup lands, and each 404s an owner outside scope.
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
