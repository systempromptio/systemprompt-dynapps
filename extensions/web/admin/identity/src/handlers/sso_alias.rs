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
    SsoCallback, SsoDriver, SsoReturn, SsoReturnBinding, binder_from_headers, into_response,
    start_response,
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

    #[must_use]
    pub fn provisions(&self) -> bool {
        self.driver
            .provider_ids()
            .iter()
            .filter_map(|id| self.driver.provider(id))
            .any(|p| p.policy().auto_provision)
    }
}

// Why: axum answers `Result<Response, E>` through `E: IntoResponse`, so each
// way a sign-in path fails picks its own response here rather than at the
// call site.
#[derive(Debug)]
pub(crate) enum SsoFlowError {
    Refused(&'static str),
    MetadataNotFound,
    MetadataUnavailable,
}

impl IntoResponse for SsoFlowError {
    fn into_response(self) -> Response {
        match self {
            Self::Refused(reason) => login_redirect(reason),
            Self::MetadataNotFound => StatusCode::NOT_FOUND.into_response(),
            Self::MetadataUnavailable => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        }
    }
}

type SsoFlowResult = Result<Response, SsoFlowError>;

#[derive(Debug, Deserialize)]
pub(crate) struct StartParams {
    redirect: Option<String>,
}

async fn start(id: &'static str, deps: SsoDeps, params: Option<StartParams>) -> SsoFlowResult {
    let redirect = params.and_then(|p| p.redirect);
    match deps.driver.start(id, redirect.as_deref()).await {
        Ok(outcome) => Ok(start_response(&outcome)),
        Err(err) => {
            tracing::warn!(provider_id = id, error = %err, "SSO start failed");
            Err(SsoFlowError::Refused(err.reason_code()))
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
) -> SsoFlowResult {
    // Why: core's driver sends a return for a provider it does not know to
    // `/`; ours belongs on the login page with the same reason.
    if !deps.provider_enabled(id) {
        return Err(SsoFlowError::Refused("unavailable"));
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
            SsoCallback {
                ret: SsoReturn { binding, params },
                binder_cookie: binder.as_deref(),
                analytics: &analytics,
                ip,
            },
        )
        .await;
    Ok(into_response(outcome))
}

pub(crate) async fn saml_start(
    Extension(deps): Extension<SsoDeps>,
    params: Result<Query<StartParams>, QueryRejection>,
) -> SsoFlowResult {
    start(entra_saml::ID, deps, params.ok().map(|Query(p)| p)).await
}

pub(crate) async fn saml_acs(
    Extension(deps): Extension<SsoDeps>,
    headers: HeaderMap,
    form: Result<Form<Vec<(String, String)>>, FormRejection>,
) -> SsoFlowResult {
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
) -> SsoFlowResult {
    start(entra_oidc::ID, deps, params.ok().map(|Query(p)| p)).await
}

pub(crate) async fn oidc_callback(
    Extension(deps): Extension<SsoDeps>,
    headers: HeaderMap,
    params: Result<Query<Vec<(String, String)>>, QueryRejection>,
) -> SsoFlowResult {
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

pub(crate) async fn saml_metadata(Extension(deps): Extension<SsoDeps>) -> SsoFlowResult {
    match deps.driver.metadata(entra_saml::ID).await {
        None => Err(SsoFlowError::MetadataNotFound),
        Some(Ok(doc)) => Ok(([(header::CONTENT_TYPE, doc.content_type)], doc.body).into_response()),
        Some(Err(err)) => {
            tracing::warn!(error = %err, "SSO metadata unavailable");
            Err(SsoFlowError::MetadataUnavailable)
        },
    }
}
