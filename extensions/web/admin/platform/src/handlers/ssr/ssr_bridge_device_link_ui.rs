//! The person's UI session a device link may carry.
//!
//! A client asks for one by adding a `person_ui` client's PKCE request to the
//! device link: `client_id`, `redirect_uri`, `scope`, `code_challenge` and
//! `code_challenge_method`. Without those five the link stays a plain device
//! link; with only some of them it is refused, never quietly downgraded. The
//! approved request rides on the exchange code, and core's
//! `POST /v1/auth/bridge/session-pat` answers it with a `ui_code`.
//!
//! The checks are core's `check_ui_request`, made with core's own client
//! lookup and redirect matching: core's `issue_device_link_code` needs an
//! `OAuthRepository`, which an extension cannot build without the deployment's
//! at-rest pepper. Only a browser sign-in approves one, and only a recent one:
//! past a day the page first sends the person through Entra with `max_age`, so
//! the UI session starts from a fresh sign-in and its step-up does not fire
//! while it lasts. A refused request is an error page that says why; nothing
//! is issued.

use std::sync::Arc;

use axum::response::{IntoResponse, Redirect, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::ClientId;
use systemprompt::oauth::PkceMethod;
use systemprompt::oauth::repository::{ClientRepository, OAuthRepository};
use systemprompt::oauth::services::validation::validate_redirect_uri;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlError, AdminHtmlResult};
use systemprompt_web_admin_core::types::UserContext;
use url::form_urlencoded;

const PAGE_PATH: &str = "/bridge-auth/device-link";
const ENTRA_START: &str = "/admin/auth/entra/start";
const FRESH_SIGN_IN_SECS: i64 = 86_400;
const ENTRA_MAX_AGE_SECS: &str = "86399";

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct UiSessionParams {
    pub client_id: Option<ClientId>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct UiSessionRequest<'a> {
    pub client_id: &'a ClientId,
    pub scope: &'a str,
    pub code_challenge: &'a str,
    redirect_uri: &'a str,
    code_challenge_method: &'a str,
}

impl UiSessionParams {
    // Why: `None` is a plain device link; a partial request is refused so a
    // client that asked for a UI session never gets a code without one.
    pub(super) fn request(&self) -> Result<Option<UiSessionRequest<'_>>, Refusal> {
        match (
            self.client_id.as_ref(),
            self.redirect_uri.as_deref(),
            self.scope.as_deref(),
            self.code_challenge.as_deref(),
            self.code_challenge_method.as_deref(),
        ) {
            (None, None, None, None, None) => Ok(None),
            (
                Some(client_id),
                Some(redirect_uri),
                Some(scope),
                Some(code_challenge),
                Some(code_challenge_method),
            ) => Ok(Some(UiSessionRequest {
                client_id,
                scope,
                code_challenge,
                redirect_uri,
                code_challenge_method,
            })),
            _ => Err(Refusal::Incomplete),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Refusal {
    Incomplete,
    Client,
    Redirect,
    Challenge,
    Scope,
    NotBrowser,
}

impl Refusal {
    const fn message(self) -> &'static str {
        match self {
            Self::Incomplete => {
                "A UI session needs client_id, redirect_uri, scope, code_challenge and \
                 code_challenge_method together."
            },
            Self::Client => "client is not a declared person_ui client",
            Self::Redirect => "redirect_uri is not registered for this client",
            Self::Challenge => "an S256 PKCE code_challenge is required",
            Self::Scope => "scope is not one the client declares",
            Self::NotBrowser => "Only a browser sign-in can approve a UI session.",
        }
    }
}

impl From<Refusal> for AdminError {
    fn from(refusal: Refusal) -> Self {
        let message = refusal.message().to_owned();
        match refusal {
            Refusal::NotBrowser => Self::Forbidden(message),
            Refusal::Incomplete
            | Refusal::Client
            | Refusal::Redirect
            | Refusal::Challenge
            | Refusal::Scope => Self::BadRequest(message),
        }
    }
}

#[derive(Debug)]
pub(super) enum Gate {
    Allowed {
        redirect_uri: String,
        authenticated_at: DateTime<Utc>,
    },
    StaleSignIn,
}

pub(super) async fn gate(
    pool: &Arc<PgPool>,
    user: &UserContext,
    request: &UiSessionRequest<'_>,
) -> AdminHtmlResult<Gate> {
    let redirect_uri = check_request(pool, request).await?;
    if !user.browser_sign_in {
        return Err(Refusal::NotBrowser.into());
    }
    let now = Utc::now();
    let signed_in = DateTime::<Utc>::from_timestamp(user.auth_time, 0)
        .filter(|_| user.auth_time > 0)
        .map(|at| at.min(now));
    Ok(match signed_in {
        Some(at) if (now - at).num_seconds() <= FRESH_SIGN_IN_SECS => Gate::Allowed {
            redirect_uri,
            authenticated_at: at,
        },
        _ => Gate::StaleSignIn,
    })
}

async fn check_request(
    pool: &Arc<PgPool>,
    request: &UiSessionRequest<'_>,
) -> AdminHtmlResult<String> {
    let db = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let clients = ClientRepository::new(&db);
    if !clients
        .is_person_ui(request.client_id)
        .await
        .map_err(AdminHtmlError::internal)?
    {
        return Err(Refusal::Client.into());
    }
    let client = clients
        .find_by_client_id(request.client_id)
        .await
        .map_err(AdminHtmlError::internal)?
        .ok_or(Refusal::Client)?;
    let Ok(redirect_uri) = validate_redirect_uri(&client.redirect_uris, Some(request.redirect_uri))
    else {
        return Err(Refusal::Redirect.into());
    };
    if request.code_challenge.is_empty()
        || request.code_challenge_method != PkceMethod::S256.as_str()
    {
        return Err(Refusal::Challenge.into());
    }
    let scopes = OAuthRepository::parse_scopes(request.scope);
    if scopes.is_empty() || scopes.iter().any(|scope| !client.scopes.contains(scope)) {
        return Err(Refusal::Scope.into());
    }
    Ok(redirect_uri)
}

// Why: core's SSO start keeps `redirect` only as a same-origin path
// (`sanitize_return_to`), so the page and its request are re-encoded here
// rather than echoed from the raw query.
pub(super) fn fresh_sign_in(
    redirect: Option<&str>,
    ui: &UiSessionParams,
    replaces: Option<&str>,
) -> Response {
    let mut page = form_urlencoded::Serializer::new(String::new());
    for (name, value) in [
        ("redirect", redirect),
        ("client_id", ui.client_id.as_ref().map(ClientId::as_str)),
        ("redirect_uri", ui.redirect_uri.as_deref()),
        ("scope", ui.scope.as_deref()),
        ("code_challenge", ui.code_challenge.as_deref()),
        ("code_challenge_method", ui.code_challenge_method.as_deref()),
        ("replaces", replaces),
    ] {
        if let Some(value) = value {
            page.append_pair(name, value);
        }
    }
    let return_to = format!("{PAGE_PATH}?{}", page.finish());
    let start = form_urlencoded::Serializer::new(String::new())
        .append_pair("max_age", ENTRA_MAX_AGE_SECS)
        .append_pair("redirect", &return_to)
        .finish();
    Redirect::to(&format!("{ENTRA_START}?{start}")).into_response()
}
