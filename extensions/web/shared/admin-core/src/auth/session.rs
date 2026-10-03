//! Who is calling: the session cookie or bearer token on a request, decoded
//! into the caller's identity.
//!
//! Every admin surface authenticates the same way, so the extractors live
//! here rather than beside any one handler. The cookie path accepts only the
//! `Api` audience; [`extract_mcp_accessor_user`] widens that for the tokens
//! core's accessors forward from the desktop bridge.

use axum::http::HeaderMap;
use systemprompt::identifiers::{Email, UserId};
use systemprompt::manifest::Config;
use systemprompt::models::auth::JwtAudience;
use systemprompt::oauth::validate_jwt_token;

use crate::error::AdminError;
use crate::types::CookieSession;

pub fn extract_user_from_cookie(headers: &HeaderMap) -> Result<CookieSession, AdminError> {
    extract_user_with_audiences(headers, &[JwtAudience::Api])
}

// Why: core's token accessors are called with a bridge-forwarded token that
// carries `Mcp`, not `Api`; the widening lives here so the cookie path keeps
// its single audience.
pub fn extract_mcp_accessor_user(headers: &HeaderMap) -> Result<CookieSession, AdminError> {
    extract_user_with_audiences(
        headers,
        &[JwtAudience::Api, JwtAudience::Mcp, JwtAudience::Bridge],
    )
}

fn extract_user_with_audiences(
    headers: &HeaderMap,
    audiences: &[JwtAudience],
) -> Result<CookieSession, AdminError> {
    let token = extract_token_from_headers(headers)?;

    let jwt_issuer = Config::get()?.jwt_issuer.clone();

    let claims = validate_jwt_token(&token, &jwt_issuer, audiences)?;

    let email = Email::try_new(claims.email.clone()).map_err(AdminError::unauthenticated)?;

    Ok(CookieSession {
        user_id: UserId::new(claims.sub),
        username: claims.username,
        email,
        session_id: claims.session_id,
    })
}

pub fn extract_token_from_headers(headers: &HeaderMap) -> Result<String, AdminError> {
    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok())
        && let Some(token) = auth
            .strip_prefix("Bearer ")
            .or_else(|| auth.strip_prefix("bearer "))
    {
        let t = token.trim();
        if !t.is_empty() {
            return Ok(t.to_owned());
        }
    }

    let cookie_header = headers
        .get("cookie")
        .ok_or_else(|| AdminError::Unauthorized("No cookie or Authorization header".to_owned()))?
        .to_str()
        .map_err(AdminError::unauthenticated)?;

    let token = cookie_header
        .split(';')
        .find_map(|c| c.trim().strip_prefix("access_token="))
        .ok_or_else(|| {
            AdminError::Unauthorized("No access_token cookie or Authorization: Bearer".to_owned())
        })?;

    if token.is_empty() {
        return Err(AdminError::Unauthorized(
            "Empty access_token cookie".to_owned(),
        ));
    }
    Ok(token.to_owned())
}

pub fn resolve_principal(
    cookie_user_id: Option<&str>,
    query_user_id: Option<&str>,
) -> Option<UserId> {
    cookie_user_id.or(query_user_id).map(UserId::new)
}
