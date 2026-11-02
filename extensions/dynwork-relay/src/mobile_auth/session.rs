//! Who is signed in to the instance in this browser.
//!
//! The authorize page sits behind the instance's own browser session: the
//! `access_token` cookie core's sign-in sets (Entra on the Dynapps brand). It
//! is verified as a session cookie (first-party audience, delegation depth,
//! user type re-derived from the scope), must be a browser session, and must
//! name a person: no anonymous visitor, delegation, plugin scope or purpose.
//! A bearer header is never read here, so no API token opens a consent page.

use axum::http::{HeaderMap, header};
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::TokenAuthority;

use crate::auth::desktop::person;

const COOKIE: &str = "access_token";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserUser {
    pub user: UserId,
    pub email: String,
}

fn session_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| {
            pair.trim()
                .strip_prefix(COOKIE)
                .and_then(|rest| rest.strip_prefix('='))
        })
        .filter(|token| !token.is_empty())
}

pub fn browser_user(authority: &dyn TokenAuthority, headers: &HeaderMap) -> Option<BrowserUser> {
    let token = session_cookie(headers)?;
    let claims = authority
        .verify_session_cookie(token)
        .inspect_err(|error| tracing::debug!(%error, "dynwork mobile: session cookie refused"))
        .ok()?;
    if !claims.custom.browser_session || person(&claims).is_err() {
        return None;
    }
    let user = UserId::try_new(claims.registered.sub).ok()?;
    Some(BrowserUser {
        user,
        email: claims.custom.email,
    })
}
