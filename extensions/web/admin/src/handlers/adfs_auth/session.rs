//! The session mint every browser SSO ends in: create the authenticated
//! session row and sign the JWT the `access_token` cookie carries.

use axum::http::HeaderMap;

use systemprompt::analytics::SessionAnalyticsBuilder;
use systemprompt::identifiers::SessionSource;
use systemprompt::models::Config;
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::oauth::SessionCreationService;
use systemprompt::oauth::services::{
    JwtConfig, JwtSigningParams, generate_access_token_jti, generate_jwt,
};

use crate::repositories::users::federated;

pub(super) async fn mint_session(
    session_service: &SessionCreationService,
    resolved: &federated::ResolvedFederatedUser,
    headers: &HeaderMap,
) -> Result<(String, i64), String> {
    let session_id = session_service
        .create_authenticated_session(
            &resolved.user_id,
            &SessionAnalyticsBuilder::new(headers).build(),
            SessionSource::Oauth,
        )
        .await
        .map_err(|e| e.to_string())?;

    let cfg = Config::get().map_err(|e| e.to_string())?;
    let uuid = resolved
        .user_id
        .as_str()
        .parse()
        .unwrap_or_else(|_| uuid::Uuid::nil());
    let user = AuthenticatedUser::new_with_roles(
        uuid,
        resolved.display_name.clone(),
        resolved.email.clone(),
        vec![Permission::User],
        resolved.roles.clone(),
    );

    let jwt_config = JwtConfig {
        permissions: vec![Permission::User],
        audience: cfg.jwt_audiences.clone(),
        expires_in_hours: Some(cfg.jwt_access_token_expiration / 3600),
        resource: None,
        plugin_id: None,
        // Why: ADFS establishes a browser session for a person, not a token
        // issued to a registered OAuth client, so there is no client to name.
        client_id: None,
    };
    let signing = JwtSigningParams {
        issuer: &cfg.jwt_issuer,
    };
    let jti = generate_access_token_jti();
    let token =
        generate_jwt(&user, jwt_config, jti, &session_id, &signing).map_err(|e| e.to_string())?;

    Ok((token, cfg.jwt_access_token_expiration))
}
