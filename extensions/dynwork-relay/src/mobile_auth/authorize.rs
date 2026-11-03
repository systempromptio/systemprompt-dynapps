//! `GET /authorize` (the consent page) and `POST /authorize/decision`.
//!
//! The page is shown only to a person signed in to the instance in this
//! browser; anyone else is sent through the instance login and back. Showing
//! it opens a grant bound to that person, the phone's PKCE challenge and its
//! `state`, keyed by a one-time consent request token that only the page's
//! form carries, so a decision cannot be forged from another site or for
//! another account. Allow answers `303` to `dynwork://auth/callback` with a
//! one-time code and the `state`; Deny with `error=access_denied` and the
//! `state`.

use axum::Form;
use axum::extract::rejection::{FormRejection, QueryRejection};
use axum::extract::{OriginalUri, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::params::{self, AuthorizeQuery, callback, percent_encode};
use super::session::browser_user;
use super::{CODE_TTL, LOGIN_PATH, OPEN_GRANTS_PER_USER, copy, page};
use crate::audit::AuditEvent;
use crate::credential::{AuthorizationCode, ConsentRequest, CredentialHash};
use crate::ids::GrantId;
use crate::state::RelayState;
use crate::store::{DecisionOutcome, GrantDecision, GrantOutcome, NewGrant};

fn error_page(state: &RelayState, status: StatusCode, message: &str) -> Response {
    page::error(&state.branding, status, message)
}

fn see_other(location: &str) -> Response {
    (
        StatusCode::SEE_OTHER,
        [
            (header::LOCATION, location),
            (header::CACHE_CONTROL, "no-store"),
        ],
    )
        .into_response()
}

pub async fn show(
    State(state): State<RelayState>,
    OriginalUri(original): OriginalUri,
    headers: HeaderMap,
    query: Result<Query<AuthorizeQuery>, QueryRejection>,
) -> Response {
    let request = match query.map(|Query(query)| params::validate(query)) {
        Ok(Ok(request)) => request,
        Ok(Err(error)) => {
            tracing::debug!(%error, "dynwork mobile: authorize request refused");
            return error_page(&state, StatusCode::BAD_REQUEST, copy::INVALID_REQUEST);
        },
        Err(error) => {
            tracing::debug!(%error, "dynwork mobile: authorize query unreadable");
            return error_page(&state, StatusCode::BAD_REQUEST, copy::INVALID_REQUEST);
        },
    };
    let Some(user) = browser_user(state.authority.as_ref(), &headers) else {
        let target = original
            .path_and_query()
            .map_or_else(|| original.path().to_owned(), ToString::to_string);
        let login = format!("{LOGIN_PATH}?redirect={}", percent_encode(&target));
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [
                (header::LOCATION, login.as_str()),
                (header::CACHE_CONTROL, "no-store"),
            ],
        )
            .into_response();
    };
    let now = state.clock.now();
    let (token, hash) = ConsentRequest::mint();
    let grant = NewGrant {
        id: GrantId::generate(),
        user: user.user,
        request: hash,
        challenge: request.challenge,
        state: request.state,
        device_name: request.device_name,
        now,
        expires_at: now + CODE_TTL,
        max_open: OPEN_GRANTS_PER_USER,
    };
    match state.store.insert_grant(&grant).await {
        Ok(GrantOutcome::Inserted) => page::consent(&page::Consent {
            branding: &state.branding,
            device_name: &grant.device_name,
            account: &user.email,
            request: token.expose(),
        }),
        Ok(GrantOutcome::TooMany) => error_page(
            &state,
            StatusCode::TOO_MANY_REQUESTS,
            copy::TOO_MANY_REQUESTS,
        ),
        Ok(GrantOutcome::UnknownUser) => {
            error_page(&state, StatusCode::UNAUTHORIZED, copy::SIGNED_OUT)
        },
        Err(error) => {
            tracing::error!(error = ?error, "dynwork mobile: grant insert failed");
            error_page(&state, StatusCode::INTERNAL_SERVER_ERROR, copy::UNAVAILABLE)
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct DecisionForm {
    pub request: String,
    pub decision: String,
}

pub async fn decide(
    State(state): State<RelayState>,
    headers: HeaderMap,
    form: Result<Form<DecisionForm>, FormRejection>,
) -> Response {
    let Ok(Form(form)) = form else {
        return error_page(&state, StatusCode::BAD_REQUEST, copy::INVALID_REQUEST);
    };
    let approve = match form.decision.as_str() {
        "allow" => true,
        "deny" => false,
        _ => return error_page(&state, StatusCode::BAD_REQUEST, copy::INVALID_REQUEST),
    };
    let Some(user) = browser_user(state.authority.as_ref(), &headers) else {
        return error_page(&state, StatusCode::UNAUTHORIZED, copy::SIGNED_OUT);
    };
    let Some(request) = CredentialHash::of_consent_request(&form.request) else {
        return error_page(&state, StatusCode::BAD_REQUEST, copy::INVALID_REQUEST);
    };
    let now = state.clock.now();
    let code = approve.then(AuthorizationCode::mint);
    let decision = GrantDecision {
        request,
        user: user.user.clone(),
        code: code
            .as_ref()
            .map(|(_, hash)| (hash.clone(), now + CODE_TTL)),
        now,
    };
    let outcome = match state.store.decide_grant(&decision).await {
        Ok(outcome) => outcome,
        Err(error) => {
            tracing::error!(error = ?error, "dynwork mobile: grant decision failed");
            return error_page(&state, StatusCode::INTERNAL_SERVER_ERROR, copy::UNAVAILABLE);
        },
    };
    let location = match (outcome, code) {
        (DecisionOutcome::Approved { state: echoed }, Some((code, _))) => {
            callback(&[("code", code.expose()), ("state", &echoed)])
        },
        (DecisionOutcome::Denied { state: echoed }, _) => {
            callback(&[("error", "access_denied"), ("state", &echoed)])
        },
        (DecisionOutcome::Unknown, _) => {
            return error_page(&state, StatusCode::BAD_REQUEST, copy::INVALID_REQUEST);
        },
        _ => return error_page(&state, StatusCode::BAD_REQUEST, copy::EXPIRED_REQUEST),
    };
    state.audit.record(&AuditEvent::MobileConsent {
        user: user.user,
        approved: approve,
    });
    see_other(&location)
}
