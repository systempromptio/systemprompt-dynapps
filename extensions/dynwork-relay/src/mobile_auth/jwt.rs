//! The phone's short-lived JWT.
//!
//! Minted by the instance's token authority for one device of an active
//! person, for one hour. Its only audience is `dynwork-mobile` and it carries
//! the `dynwork-mobile` purpose, so core's session middleware, the bridge and
//! MCP routes, and the relay's desktop routes all refuse it: it proves to the
//! instance's `DynWork` routes which account and which device a phone is, and
//! nothing more. The device id travels as the session id, so a revoked
//! device's JWT is refused before it expires.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::{AccessTokenId, SessionId, UserId};
use systemprompt::models::auth::{
    AuthenticatedUser, JwtAudience, Permission, TokenAuthority, TokenError, UserJwtRequest,
    UserType,
};

use crate::auth::AuthError;
use crate::ids::DeviceId;
use crate::store::Account;

pub const MOBILE_AUDIENCE: &str = "dynwork-mobile";
pub const MOBILE_PURPOSE: &str = "dynwork-mobile";
pub const JWT_TTL: chrono::Duration = chrono::Duration::hours(1);

#[derive(Debug, Clone)]
pub struct MobileToken {
    pub token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileIdentity {
    pub user: UserId,
    pub device: DeviceId,
    pub email: String,
    pub name: String,
    pub expires_at: DateTime<Utc>,
}

fn audience() -> Vec<JwtAudience> {
    vec![JwtAudience::Resource(MOBILE_AUDIENCE.to_owned())]
}

pub fn mint(
    authority: &dyn TokenAuthority,
    account: &Account,
    device: DeviceId,
) -> Result<MobileToken, TokenError> {
    let permissions = vec![Permission::User];
    let token = authority.mint_user_jwt(&UserJwtRequest {
        user: AuthenticatedUser::new_with_roles(
            account.user.clone(),
            account.name.clone(),
            account.email.clone(),
            permissions.clone(),
            account.roles.clone(),
        ),
        session_id: SessionId::new(device.to_string()),
        jti: AccessTokenId::generate(),
        permissions,
        audience: audience(),
        ttl: JWT_TTL,
        client_id: None,
        purpose: Some(MOBILE_PURPOSE.to_owned()),
        plugin_id: None,
        interaction: None,
        browser_session: false,
    })?;
    Ok(MobileToken {
        token,
        expires_in: JWT_TTL.num_seconds(),
    })
}

pub fn verify(authority: &dyn TokenAuthority, token: &str) -> Result<MobileIdentity, AuthError> {
    let claims = authority
        .verify_bearer(token, &audience())
        .map_err(AuthError::Rejected)?;
    let custom = &claims.custom;
    if custom.purpose.as_deref() != Some(MOBILE_PURPOSE)
        || custom.user_type != UserType::User
        || custom.act.is_some()
        || custom.plugin_id.is_some()
    {
        return Err(AuthError::NotAPerson);
    }
    let device: DeviceId = custom
        .session_id
        .as_ref()
        .ok_or(AuthError::Malformed)?
        .as_str()
        .parse()
        .map_err(|_not_a_device: crate::ids::IdError| AuthError::Malformed)?;
    let user = UserId::try_new(claims.registered.sub).map_err(|_bad| AuthError::Malformed)?;
    let expires_at =
        DateTime::<Utc>::from_timestamp(claims.registered.exp, 0).ok_or(AuthError::Malformed)?;
    Ok(MobileIdentity {
        user,
        device,
        email: claims.custom.email,
        name: claims.custom.username,
        expires_at,
    })
}
