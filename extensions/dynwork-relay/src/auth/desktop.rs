//! The desktop's `DynWork` JWT.
//!
//! Signature, issuer and audience are not enough: core's anonymous site
//! session is a first-party JWT too (audiences `web`, `api`, `a2a`, `mcp`).
//! A desktop is a signed-in person, so the claims must name a user or an
//! admin and carry no delegation (`act`), no plugin scope and no caller
//! purpose. The bridge device link mints exactly that (audiences `bridge` and
//! `mcp`, the user's own permissions, no purpose).

use chrono::{DateTime, Utc};
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt::models::auth::{JwtAudience, JwtClaims, TokenAuthority, UserType};

use super::AuthError;

pub const DESKTOP_AUDIENCES: &[JwtAudience] = &[JwtAudience::Bridge, JwtAudience::Api];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopIdentity {
    pub user: UserId,
    pub username: String,
    pub email: String,
    pub session: Option<SessionId>,
    pub expires_at: DateTime<Utc>,
}

pub(crate) const fn person(claims: &JwtClaims) -> Result<(), AuthError> {
    let custom = &claims.custom;
    if !matches!(custom.user_type, UserType::User | UserType::Admin) {
        return Err(AuthError::NotAPerson);
    }
    if custom.act.is_some() || custom.plugin_id.is_some() || custom.purpose.is_some() {
        return Err(AuthError::NotAPerson);
    }
    Ok(())
}

pub fn verify_desktop(
    authority: &dyn TokenAuthority,
    token: &str,
) -> Result<DesktopIdentity, AuthError> {
    let claims = authority
        .verify_bearer(token, DESKTOP_AUDIENCES)
        .map_err(AuthError::Rejected)?;
    person(&claims)?;
    let user = UserId::try_new(claims.registered.sub).map_err(|source| {
        tracing::debug!(error = %source, "desktop token subject is not a user id");
        AuthError::Malformed
    })?;
    let expires_at =
        DateTime::<Utc>::from_timestamp(claims.registered.exp, 0).ok_or(AuthError::Malformed)?;
    Ok(DesktopIdentity {
        user,
        username: claims.custom.username,
        email: claims.custom.email,
        session: claims.custom.session_id,
        expires_at,
    })
}
