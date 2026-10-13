//! This installation's sign-in paths, served by core's SSO driver: the same
//! flow core mounts at `/api/v1/core/sso/{id}/*`, reached at the URLs the
//! Entra ID applications were registered with.
//!
//! A start the driver refuses redirects to `/admin/login?sso=<reason>`; a
//! finish always answers a redirect, never an error status, so a failed
//! sign-in lands on a usable login page.

use std::net::IpAddr;
use std::sync::Arc;

use axum::Extension;
use axum::extract::rejection::{FormRejection, QueryRejection};
use axum::extract::{Form, Query};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::analytics::SessionAnalyticsBuilder;
use systemprompt::oauth::SessionCreationService;
use systemprompt::sso::{
    SsoDriver, SsoReturn, SsoReturnBinding, binder_from_headers, into_response, start_response,
};

use crate::sso::{entra_oidc, entra_saml};

const LOGIN_PATH: &str = "/admin/login";

/// Per-request dependencies for every browser sign-in, shared via an axum
/// `Extension`.
#[derive(Clone)]
pub struct SsoDeps {
    pub driver: SsoDriver,
    pub write_pool: Arc<PgPool>,
    pub session_service: Arc<SessionCreationService>,
}

impl std::fmt::Debug for SsoDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SsoDeps")
            .field("driver", &self.driver)
            .finish_non_exhaustive()
    }
}

impl SsoDeps {
    #[must_use]
    pub fn provider_enabled(&self, id: &str) -> bool {
        self.driver.provider(id).is_some()
    }

    /// Whether any enabled provider creates accounts on first sign-in.
    #[must_use]
    pub fn provisions(&self) -> bool {
        self.driver
            .provider_ids()
            .iter()
            .filter_map(|id| self.driver.provider(id))
            .any(|p| p.policy().auto_provision)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct StartParams {
    redirect: Option<String>,
}

async fn start(id: &'static str, deps: SsoDeps, params: Option<StartParams>) -> Response {
    let redirect = params.and_then(|p| p.redirect);
    match deps.driver.start(id, redirect.as_deref()).await {
        Ok(outcome) => start_response(outcome),
        Err(err) => {
            tracing::warn!(provider_id = id, error = %err, "SSO start failed");
            login_redirect(err.reason_code())
        },
    }
}

fn login_redirect(reason: &str) -> Response {
    Redirect::to(&format!("{LOGIN_PATH}?sso={reason}")).into_response()
}

async fn finish(
    id: &'static str,
    deps: SsoDeps,
    headers: &HeaderMap,
    binding: SsoReturnBinding,
    params: Vec<(String, String)>,
) -> Response {
    // Why: core's driver sends a return for a provider it does not know to
    // `/`; ours belongs on the login page with the same reason.
    if !deps.provider_enabled(id) {
        return login_redirect("unavailable");
    }
    let analytics = SessionAnalyticsBuilder::new(headers).build();
    let ip = analytics
        .ip_address
        .as_deref()
        .and_then(|raw| raw.parse::<IpAddr>().ok());
    let binder = binder_from_headers(headers);
    let outcome = deps
        .driver
        .finish(
            id,
            SsoReturn { binding, params },
            binder.as_deref(),
            &analytics,
            ip,
        )
        .await;
    into_response(outcome)
}

pub(crate) async fn saml_start(
    Extension(deps): Extension<SsoDeps>,
    params: Result<Query<StartParams>, QueryRejection>,
) -> Response {
    start(entra_saml::ID, deps, params.ok().map(|Query(p)| p)).await
}

pub(crate) async fn saml_acs(
    Extension(deps): Extension<SsoDeps>,
    headers: HeaderMap,
    form: Result<Form<Vec<(String, String)>>, FormRejection>,
) -> Response {
    let params = form.map(|Form(p)| p).unwrap_or_default();
    finish(
        entra_saml::ID,
        deps,
        &headers,
        SsoReturnBinding::Form,
        params,
    )
    .await
}

pub(crate) async fn oidc_start(
    Extension(deps): Extension<SsoDeps>,
    params: Result<Query<StartParams>, QueryRejection>,
) -> Response {
    start(entra_oidc::ID, deps, params.ok().map(|Query(p)| p)).await
}

pub(crate) async fn oidc_callback(
    Extension(deps): Extension<SsoDeps>,
    headers: HeaderMap,
    params: Result<Query<Vec<(String, String)>>, QueryRejection>,
) -> Response {
    let params = params.map(|Query(p)| p).unwrap_or_default();
    finish(
        entra_oidc::ID,
        deps,
        &headers,
        SsoReturnBinding::Query,
        params,
    )
    .await
}

/// `GET /saml/metadata` — our service-provider descriptor, which the Entra
/// ID application's Identifier names.
pub(crate) async fn saml_metadata(Extension(deps): Extension<SsoDeps>) -> Response {
    match deps.driver.metadata(entra_saml::ID).await {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(Ok(doc)) => ([(header::CONTENT_TYPE, doc.content_type)], doc.body).into_response(),
        Some(Err(err)) => {
            tracing::warn!(error = %err, "SSO metadata unavailable");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        },
    }
}
