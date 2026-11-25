//! The person's `ui` session as a bearer: the credential `DynWork` desktop
//! holds for its person, checked the one way the extension checks it
//! wherever it takes one (the session handoff, the artifacts broker).
//!
//! `Authorization: Bearer <token>`, signed by core for the `ui` audience and
//! not expired, carrying the `ui_session` claim in the shape core gives it
//! (`ui_session_well_formed`), a session id and a sign-in time (`auth_time`
//! above zero). Anything else, an agent's bearer, the bridge's, an API key,
//! a Device key or the web session sent as a bearer, is no person session.
//! The sign-in time is never later than now.

use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt::models::auth::{AuthenticatedUser, JwtAudience, JwtClaims};
use systemprompt::oauth::validate_jwt_token;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

const BEARER: &str = "Bearer ";

#[derive(Debug, Clone)]
pub(crate) struct PersonSession {
    pub(crate) user: AuthenticatedUser,
    pub(crate) session: SessionId,
    pub(crate) signed_in: DateTime<Utc>,
}

pub(crate) fn person_session(boot: &ExtensionBoot, headers: &HeaderMap) -> Option<PersonSession> {
    let bearer = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix(BEARER))?;
    let claims = validate_jwt_token(
        boot.token_authority().as_ref(),
        bearer.trim(),
        &[JwtAudience::Ui],
    )
    .inspect_err(|error| tracing::debug!(%error, "a person session bearer did not validate"))
    .ok()?;
    person_of(claims)
}

fn person_of(claims: JwtClaims) -> Option<PersonSession> {
    if !claims.custom.ui_session || !claims.ui_session_well_formed() {
        return None;
    }
    let signed_in = DateTime::<Utc>::from_timestamp(claims.custom.auth_time, 0)
        .filter(|_| claims.custom.auth_time > 0)?
        .min(Utc::now());
    let session = claims.custom.session_id?;
    let user = AuthenticatedUser::new_with_roles(
        UserId::new(claims.registered.sub),
        claims.custom.username,
        claims.custom.email,
        claims.custom.scope,
        claims.custom.roles,
    );
    Some(PersonSession {
        user,
        session,
        signed_in,
    })
}
