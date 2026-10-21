//! The caller's own personal-subscription keys, relayed to core.
//!
//! Core owns the per-person provider-credential store and serves it at
//! `/v1/me/provider-credentials`, which authenticates by bearer token. The
//! console's session lives in an `HttpOnly` cookie the page script cannot
//! read, so these handlers forward the caller's own session token as the
//! bearer and pass core's status and body back unchanged. Nothing here names
//! a user: core reads the account from the token, so there is no parameter
//! that could point at somebody else's key.

use std::time::Duration;

use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use systemprompt::manifest::Config;
use systemprompt_web_admin_core::auth::session::extract_token_from_headers;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

const CORE_PATH: &str = "/v1/me/provider-credentials";
const RELAY_TIMEOUT: Duration = Duration::from_secs(30);

fn provider_segment(provider: &str) -> AdminResult<&str> {
    let shaped = !provider.is_empty()
        && provider.len() <= 64
        && provider
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'));
    if shaped {
        Ok(provider)
    } else {
        Err(AdminError::BadRequest(format!(
            "{provider:?} is not a provider name."
        )))
    }
}

async fn relay(
    headers: &HeaderMap,
    method: Method,
    suffix: &str,
    body: Option<Bytes>,
) -> AdminResult<Response> {
    let token = extract_token_from_headers(headers)?;
    let base = Config::get()?
        .api_internal_url
        .trim_end_matches('/')
        .to_owned();
    let client = reqwest::Client::builder()
        .timeout(RELAY_TIMEOUT)
        .build()
        .map_err(|e| AdminError::UpstreamFailure {
            context: "provider-credential relay client",
            source: Box::new(e),
        })?;
    let mut request = client
        .request(method, format!("{base}{CORE_PATH}{suffix}"))
        .bearer_auth(token);
    if let Some(body) = body {
        request = request
            .header(header::CONTENT_TYPE, "application/json")
            .body(body);
    }
    let upstream = request
        .send()
        .await
        .map_err(|e| AdminError::UpstreamFailure {
            context: "provider-credential store",
            source: Box::new(e),
        })?;
    let status =
        StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let content_type = upstream
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json")
        .to_owned();
    let bytes = upstream
        .bytes()
        .await
        .map_err(|e| AdminError::UpstreamFailure {
            context: "provider-credential store",
            source: Box::new(e),
        })?;
    if status == StatusCode::NO_CONTENT {
        return Ok(status.into_response());
    }
    Ok((status, [(header::CONTENT_TYPE, content_type)], bytes).into_response())
}

pub(crate) async fn list_own_provider_credentials(headers: HeaderMap) -> AdminResult<Response> {
    relay(&headers, Method::GET, "", None).await
}

pub(crate) async fn put_own_provider_credential(
    headers: HeaderMap,
    Path(provider): Path<String>,
    body: Bytes,
) -> AdminResult<Response> {
    let suffix = format!("/{}", provider_segment(&provider)?);
    relay(&headers, Method::PUT, &suffix, Some(body)).await
}

pub(crate) async fn delete_own_provider_credential(
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> AdminResult<Response> {
    let suffix = format!("/{}", provider_segment(&provider)?);
    relay(&headers, Method::DELETE, &suffix, None).await
}
