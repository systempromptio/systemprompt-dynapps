//! Request dispatch for [`super::FakePortal`]: OAuth discovery, registration,
//! authorization and token, then the MCP endpoints behind the bearer check.

use axum::http::{HeaderMap, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{Inner, MCP_PATHS, PendingCode};
use crate::fake_http::{RecordedRequest, record};

const RESOURCE_METADATA: &str = "/.well-known/oauth-protected-resource";

pub(super) fn handle(
    inner: &Inner,
    method: &Method,
    uri: &Uri,
    headers: &HeaderMap,
    body: &[u8],
) -> Response {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let body = String::from_utf8_lossy(body).into_owned();
    record(
        &inner.requests,
        RecordedRequest {
            method: method.to_string(),
            path: uri.path().to_owned(),
            query: uri.query().map(str::to_owned),
            authorization: authorization.clone(),
            body: body.clone(),
        },
    );
    let path = uri.path();
    let base = &inner.base_url;
    match (method.as_str(), path) {
        ("GET", p) if p.starts_with(RESOURCE_METADATA) => {
            let suffix = &p[RESOURCE_METADATA.len()..];
            let resource = if suffix.is_empty() {
                MCP_PATHS[0]
            } else {
                suffix
            };
            if !MCP_PATHS.contains(&resource) {
                return error(StatusCode::NOT_FOUND, "unknown resource");
            }
            json_ok(&json!({
                "resource": format!("{base}{resource}"),
                "authorization_servers": [base],
                "bearer_methods_supported": ["header"],
                "scopes_supported": ["odoo"],
            }))
        },
        ("GET", "/.well-known/oauth-authorization-server") => json_ok(&json!({
            "issuer": base,
            "authorization_endpoint": format!("{base}/authorize"),
            "token_endpoint": format!("{base}/token"),
            "registration_endpoint": format!("{base}/register"),
            "userinfo_endpoint": format!("{base}/userinfo"),
            "response_types_supported": ["code"],
            "grant_types_supported": ["authorization_code", "refresh_token"],
            "code_challenge_methods_supported": ["S256"],
            "token_endpoint_auth_methods_supported": ["none"],
            "scopes_supported": ["odoo"],
        })),
        ("POST", "/register") => register(inner, &body),
        ("GET", "/authorize") => authorize(inner, uri.query().unwrap_or_default()),
        ("POST", "/token") => token(inner, &body),
        ("GET", "/userinfo") => inner
            .subject_of_header(authorization.as_deref())
            .map_or_else(
                || unauthorized(inner, MCP_PATHS[0]),
                |sub| {
                    json_ok(
                        &json!({ "sub": sub, "email": format!("{sub}@portal.test"), "name": sub }),
                    )
                },
            ),
        (_, p) if MCP_PATHS.contains(&p) => {
            super::mcp::handle(inner, (method, p), headers, authorization.as_deref(), &body)
        },
        _ => error(StatusCode::NOT_FOUND, "no such route"),
    }
}

fn register(inner: &Inner, body: &str) -> Response {
    let Ok(request) = serde_json::from_str::<Value>(body) else {
        return error(StatusCode::BAD_REQUEST, "invalid_client_metadata");
    };
    if request["token_endpoint_auth_method"] != "none" {
        return error(StatusCode::BAD_REQUEST, "invalid_client_metadata");
    }
    let redirects: Vec<String> = request["redirect_uris"]
        .as_array()
        .map(|uris| {
            uris.iter()
                .filter_map(|u| u.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    if redirects.is_empty() {
        return error(StatusCode::BAD_REQUEST, "invalid_redirect_uri");
    }
    let client_id = inner.next("portal-client");
    inner
        .clients
        .lock()
        .expect("clients")
        .insert(client_id.clone(), redirects.clone());
    (
        StatusCode::CREATED,
        axum::Json(json!({
            "client_id": client_id,
            "redirect_uris": redirects,
            "token_endpoint_auth_method": "none",
            "grant_types": ["authorization_code", "refresh_token"],
        })),
    )
        .into_response()
}

fn params(raw: &str) -> std::collections::BTreeMap<String, String> {
    url::form_urlencoded::parse(raw.as_bytes())
        .into_owned()
        .collect()
}

fn authorize(inner: &Inner, query: &str) -> Response {
    let q = params(query);
    let get = |k: &str| q.get(k).cloned().unwrap_or_default();
    let (client_id, redirect_uri) = (get("client_id"), get("redirect_uri"));
    let registered = inner
        .clients
        .lock()
        .expect("clients")
        .get(&client_id)
        .is_some_and(|uris| uris.contains(&redirect_uri));
    if !registered {
        return error(StatusCode::BAD_REQUEST, "unknown client or redirect_uri");
    }
    if get("response_type") != "code"
        || get("code_challenge_method") != "S256"
        || get("code_challenge").is_empty()
    {
        return error(StatusCode::BAD_REQUEST, "PKCE S256 required");
    }
    let code = inner.next("portal-code");
    let subject = inner.login.lock().expect("login").clone();
    inner.codes.lock().expect("codes").insert(
        code.clone(),
        PendingCode {
            client_id,
            redirect_uri: redirect_uri.clone(),
            challenge: get("code_challenge"),
            subject,
        },
    );
    let mut location = url::Url::parse(&redirect_uri).expect("registered redirect is a URL");
    location
        .query_pairs_mut()
        .append_pair("code", &code)
        .append_pair("state", &get("state"))
        .append_pair("iss", &inner.base_url);
    (
        StatusCode::FOUND,
        [(header::LOCATION, location.to_string())],
    )
        .into_response()
}

fn token(inner: &Inner, body: &str) -> Response {
    let form = params(body);
    let get = |k: &str| form.get(k).cloned().unwrap_or_default();
    let subject = match get("grant_type").as_str() {
        "authorization_code" => {
            let Some(pending) = inner.codes.lock().expect("codes").remove(&get("code")) else {
                return error(StatusCode::BAD_REQUEST, "invalid_grant");
            };
            let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(get("code_verifier").as_bytes()));
            if pending.challenge != challenge
                || pending.client_id != get("client_id")
                || pending.redirect_uri != get("redirect_uri")
            {
                return error(StatusCode::BAD_REQUEST, "invalid_grant");
            }
            pending.subject
        },
        "refresh_token" => {
            let subject = inner
                .refresh
                .lock()
                .expect("refresh")
                .remove(&get("refresh_token"));
            let Some(subject) = subject else {
                return error(StatusCode::BAD_REQUEST, "invalid_grant");
            };
            subject
        },
        _ => return error(StatusCode::BAD_REQUEST, "unsupported_grant_type"),
    };
    let (access, refresh) = inner.issue(&subject);
    let expires_in = *inner.expires_in.lock().expect("expires_in");
    json_ok(&json!({
        "access_token": access,
        "refresh_token": refresh,
        "token_type": "Bearer",
        "expires_in": expires_in,
        "scope": "odoo",
    }))
}

pub(super) fn json_ok(value: &Value) -> Response {
    axum::Json(value.clone()).into_response()
}

pub(super) fn error(status: StatusCode, message: &str) -> Response {
    (status, axum::Json(json!({ "error": message }))).into_response()
}

pub(super) fn unauthorized(inner: &Inner, path: &str) -> Response {
    let challenge = format!(
        "Bearer resource_metadata=\"{}{RESOURCE_METADATA}{path}\"",
        inner.base_url
    );
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, challenge)],
        axum::Json(json!({ "error": "invalid_token" })),
    )
        .into_response()
}
