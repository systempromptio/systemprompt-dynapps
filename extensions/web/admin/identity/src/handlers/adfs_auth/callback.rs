//! `POST /admin/auth/adfs/acs` — the assertion consumer service. Verify
//! the posted `SAMLResponse`, gate its claims, resolve the identity to a local
//! user, and set the session.

use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Extension;
use axum::extract::Form;
use axum::extract::rejection::FormRejection;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use saml::sp::LoginTracker;
use saml::{
    ConsumeResponse, Identity, InMemoryReplayCache, ReplayMode, SsoResponseBinding,
    SsoResponseEndpoint,
};
use serde::Deserialize;

use super::identity::{AssertionClaims, resolve_identity};
use super::provider::{idp_descriptor, service_provider};
use super::session::{SessionSubject, mint_session, record_login, session_cookie};
use super::{
    AdfsDeps, AdfsError, FlowState, LoginRedirect, clear_state_cookie, read_state_cookie,
    sanitize_redirect, use_https,
};

static REPLAY_CACHE: LazyLock<InMemoryReplayCache> = LazyLock::new(InMemoryReplayCache::default);

#[derive(Deserialize)]
pub(crate) struct CallbackForm {
    #[serde(rename = "SAMLResponse")]
    saml_response: String,
    #[serde(rename = "RelayState")]
    relay_state: Option<String>,
}

struct SuccessfulLogin {
    redirect_to: String,
    jwt: String,
    max_age: i64,
}

pub(crate) async fn adfs_callback(
    Extension(deps): Extension<AdfsDeps>,
    headers: HeaderMap,
    form: Result<Form<CallbackForm>, FormRejection>,
) -> Result<Response, LoginRedirect> {
    if !deps.config.is_usable() {
        return Err(LoginRedirect("unavailable"));
    }
    let Ok(Form(form)) = form else {
        return Err(LoginRedirect("error"));
    };
    let login = run_callback(&deps, &headers, form).await?;
    Ok(success_response(&login))
}

async fn run_callback(
    deps: &AdfsDeps,
    headers: &HeaderMap,
    form: CallbackForm,
) -> Result<SuccessfulLogin, LoginRedirect> {
    let cfg = &deps.config;
    let flow = correlate(
        cfg.allow_idp_initiated,
        headers,
        form.relay_state.as_deref(),
    )?;
    let (idp, identity) = verify_response(deps, &form.saml_response, flow.as_ref())
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "SSO assertion rejected");
            match e {
                AdfsError::Assertion(saml::Error::XmlParse(m)) if m.contains("NameID") => {
                    LoginRedirect("no_subject")
                },
                AdfsError::Assertion(saml::Error::UnsolicitedNotAllowed) => {
                    LoginRedirect("correlation_lost")
                },
                AdfsError::Metadata(_)
                | AdfsError::MetadataFetch(_)
                | AdfsError::MetadataParse(_)
                | AdfsError::ServiceProvider(_) => LoginRedirect("unavailable"),
                _ => LoginRedirect("invalid_assertion"),
            }
        })?;

    let claims = AssertionClaims::from_identity(&identity);
    let resolved = resolve_identity(deps, &idp, &claims).await?;

    let subject = SessionSubject::from(&resolved);
    let (jwt, max_age) = mint_session(&deps.session_service, &subject, headers)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to mint session for SSO user");
            LoginRedirect("error")
        })?;
    record_login(&deps.write_pool, &subject).await;

    tracing::info!(
        user_id = %resolved.user_id,
        email = %resolved.email,
        roles = ?resolved.roles,
        assertion_id = %identity.assertion_id,
        "SSO login succeeded"
    );

    Ok(SuccessfulLogin {
        redirect_to: flow.map_or_else(|| sanitize_redirect(None), |f| f.redirect_to),
        jwt,
        max_age,
    })
}

fn correlate(
    allow_idp_initiated: bool,
    headers: &HeaderMap,
    relay_state: Option<&str>,
) -> Result<Option<FlowState>, LoginRedirect> {
    match read_state_cookie(headers) {
        Some(flow) => {
            if relay_state != Some(flow.state.as_str()) {
                tracing::warn!("SSO RelayState does not match the state cookie");
                return Err(LoginRedirect("error"));
            }
            Ok(Some(flow))
        },
        None if allow_idp_initiated => Ok(None),
        None => {
            tracing::warn!(
                "SSO response arrived with no state cookie and IdP-initiated sign-in is off"
            );
            Err(LoginRedirect("error"))
        },
    }
}

async fn verify_response(
    deps: &AdfsDeps,
    saml_response_b64: &str,
    flow: Option<&FlowState>,
) -> Result<(String, Identity), AdfsError> {
    let cfg = &deps.config;
    let idp = idp_descriptor(cfg).await?;
    let sp = service_provider(cfg)?;
    let xml = STANDARD
        .decode(saml_response_b64.trim())
        .map_err(|e| AdfsError::Base64(e.to_string()))?;

    let tracker = flow.map(|f| LoginTracker {
        request_id: f.request_id.clone(),
        issued_at: UNIX_EPOCH + Duration::from_secs(f.issued_at_unix),
        idp_entity_id: idp.entity_id.clone(),
        acs_endpoint: SsoResponseEndpoint::post(cfg.acs_url.clone(), 0, true),
        requested_authn_context: None,
        requested_name_id_format: None,
    });

    let identity = sp
        .consume_response(ConsumeResponse {
            idp: &idp,
            peer_crypto_policy: None,
            saml_response: &xml,
            binding: SsoResponseBinding::HttpPost,
            relay_state: flow.map(|f| f.state.as_str()),
            tracker: tracker.as_ref(),
            expected_destination: &cfg.acs_url,
            now: SystemTime::now(),
            clock_skew: Duration::from_secs(cfg.clock_skew_seconds),
            replay_cache: Some(&*REPLAY_CACHE),
            replay_mode: ReplayMode::All,
            holder_of_key_cert: None,
        })
        .map_err(AdfsError::Assertion)?;
    Ok((idp.entity_id.clone(), identity))
}

fn success_response(login: &SuccessfulLogin) -> Response {
    let mut out = HeaderMap::new();
    if let Ok(val) = clear_state_cookie(use_https()).parse() {
        out.append(SET_COOKIE, val);
    }
    if let Ok(val) = session_cookie(&login.jwt, login.max_age).parse() {
        out.append(SET_COOKIE, val);
    }
    (out, Redirect::to(&login.redirect_to)).into_response()
}
