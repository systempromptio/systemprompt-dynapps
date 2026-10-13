//! The session mint every browser sign-in ends in: create the authenticated
//! session row, sign the JWT, and spell the `access_token` cookie that
//! carries it. Entra ID SSO and the development login link both land here, so
//! the two can never drift on what a session is — and both record the sign-in
//! as a `login` activity, the row every "Logins" figure counts.
//!
//! A JWT is checked on `permissions`, not roles, so a subject whose roles
//! grant manage is minted `Admin` in addition to (never instead of) `User`.

use axum::http::HeaderMap;
use sqlx::PgPool;

use systemprompt::identifiers::UserId;

use systemprompt::analytics::SessionAnalyticsBuilder;
use systemprompt::identifiers::SessionSource;
use systemprompt::manifest::Config;
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::oauth::SessionCreationService;
use systemprompt::oauth::services::{JwtConfig, JwtSigningParams, generate_jwt};

use super::secure_flag;
use crate::repositories::dev_login::DevLoginUser;
use crate::repositories::users::federated;
use systemprompt_web_admin_core::activity::NewActivity;
use systemprompt_web_admin_core::repositories::user_activity::record;

#[derive(Debug, Clone)]
pub(crate) struct SessionSubject {
    pub user_id: UserId,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

impl From<&federated::ResolvedFederatedUser> for SessionSubject {
    fn from(resolved: &federated::ResolvedFederatedUser) -> Self {
        Self {
            user_id: resolved.user_id.clone(),
            email: resolved.email.clone(),
            display_name: resolved.display_name.clone(),
            roles: resolved.roles.clone(),
        }
    }
}

impl From<DevLoginUser> for SessionSubject {
    fn from(user: DevLoginUser) -> Self {
        Self {
            user_id: user.user_id,
            email: user.email,
            display_name: user.display_name,
            roles: user.roles,
        }
    }
}

pub(crate) async fn record_login(pool: &PgPool, subject: &SessionSubject) {
    record(
        pool,
        NewActivity::login(&subject.user_id, &subject.display_name),
    )
    .await;
}

pub(crate) fn session_cookie(jwt: &str, max_age: i64) -> String {
    format!(
        "access_token={jwt}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        secure_flag()
    )
}

pub fn permissions_for_roles(roles: &[String]) -> Vec<Permission> {
    if systemprompt_web_admin_core::types::roles_grant_manage(roles) {
        vec![Permission::Admin, Permission::User]
    } else {
        vec![Permission::User]
    }
}

pub(crate) async fn mint_session(
    session_service: &SessionCreationService,
    subject: &SessionSubject,
    headers: &HeaderMap,
) -> Result<(String, i64), String> {
    let session_id = session_service
        .create_authenticated_session(
            &subject.user_id,
            &SessionAnalyticsBuilder::new(headers).build(),
            SessionSource::Oauth,
        )
        .await
        .map_err(|e| e.to_string())?;

    let cfg = Config::get().map_err(|e| e.to_string())?;
    let permissions = permissions_for_roles(&subject.roles);
    let user = AuthenticatedUser::new_with_roles(
        subject.user_id.clone(),
        subject.display_name.clone(),
        subject.email.clone(),
        permissions.clone(),
        subject.roles.clone(),
    );

    let jwt_config = JwtConfig {
        permissions,
        audience: cfg.jwt_audiences.clone(),
        expires_in: chrono::Duration::seconds(cfg.jwt_access_token_expiration),
        resource: None,
        plugin_id: None,
        client_id: None,
        purpose: None,
    };
    let signing = JwtSigningParams {
        issuer: &cfg.jwt_issuer,
    };
    let jti = systemprompt::identifiers::AccessTokenId::generate();
    let token =
        generate_jwt(&user, jwt_config, jti, &session_id, &signing).map_err(|e| e.to_string())?;

    Ok((token, cfg.jwt_access_token_expiration))
}
