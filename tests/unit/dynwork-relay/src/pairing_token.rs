use chrono::Utc;
use dynwork_relay::auth::{AuthError, verify_desktop};
use dynwork_relay::credential::{CredentialHash, DeviceCredential, PAIRING_PREFIX, PairingToken};
use systemprompt::identifiers::{AccessTokenId, PluginId, SessionId, UserId};
use systemprompt::models::auth::{
    ActClaim, AuthenticatedUser, JwtAudience, Permission, UserJwtRequest,
};
use systemprompt_security::jwt::user_claims;

use crate::support::{ISSUER, authority};

fn request(permissions: Vec<Permission>, audience: Vec<JwtAudience>) -> UserJwtRequest {
    UserJwtRequest {
        user: AuthenticatedUser::new(
            UserId::try_new("alice").expect("a user id"),
            "alice".to_owned(),
            "alice@example.test".to_owned(),
            permissions.clone(),
        ),
        session_id: SessionId::generate(),
        jti: AccessTokenId::generate(),
        permissions,
        audience,
        ttl: chrono::Duration::minutes(5),
        client_id: None,
        purpose: None,
        plugin_id: None,
        interaction: None,
        browser_session: false,
    }
}

fn bridge(permissions: Vec<Permission>) -> UserJwtRequest {
    request(permissions, vec![JwtAudience::Bridge, JwtAudience::Mcp])
}

fn mint(request: &UserJwtRequest) -> String {
    authority().mint_user_jwt(request).expect("mint a token")
}

#[test]
fn a_pairing_token_is_opaque_and_hashes_to_its_stored_hash() {
    let (token, hash) = PairingToken::mint();
    assert!(token.expose().starts_with(PAIRING_PREFIX));
    assert_eq!(token.expose().len(), PAIRING_PREFIX.len() + 43);
    assert_eq!(CredentialHash::of_pairing_token(token.expose()), Some(hash));
    assert!(!format!("{token:?}").contains(&token.expose()[PAIRING_PREFIX.len()..]));
}

#[test]
fn pairing_tokens_and_device_credentials_are_not_interchangeable() {
    let (pairing, _) = PairingToken::mint();
    let (credential, _) = DeviceCredential::mint();
    assert_eq!(CredentialHash::of_presented(pairing.expose()), None);
    assert_eq!(CredentialHash::of_pairing_token(credential.expose()), None);
}

#[test]
fn a_users_and_an_admins_bridge_token_are_desktops() {
    for permissions in [vec![Permission::User], vec![Permission::Admin]] {
        let identity =
            verify_desktop(authority().as_ref(), &mint(&bridge(permissions))).expect("a desktop");
        assert_eq!(identity.user.as_str(), "alice");
        assert!(identity.expires_at > Utc::now());
    }
}

#[test]
fn an_anonymous_or_service_token_is_not_a_desktop() {
    let anonymous = request(vec![Permission::Anonymous], JwtAudience::standard());
    let service = bridge(vec![Permission::Service]);
    for token in [mint(&anonymous), mint(&service), mint(&bridge(Vec::new()))] {
        assert!(matches!(
            verify_desktop(authority().as_ref(), &token),
            Err(AuthError::NotAPerson)
        ));
    }
}

#[test]
fn a_plugin_scoped_or_purpose_bound_token_is_not_a_desktop() {
    let plugin = UserJwtRequest {
        plugin_id: Some(PluginId::new("dynapps-commons")),
        ..bridge(vec![Permission::User])
    };
    let purpose = UserJwtRequest {
        purpose: Some("ci-export".to_owned()),
        ..bridge(vec![Permission::User])
    };
    for token in [mint(&plugin), mint(&purpose)] {
        assert!(matches!(
            verify_desktop(authority().as_ref(), &token),
            Err(AuthError::NotAPerson)
        ));
    }
}

#[test]
fn a_delegated_token_is_not_a_desktop() {
    let mut claims = user_claims(ISSUER, &bridge(vec![Permission::User])).expect("claims");
    claims.custom.act = Some(ActClaim {
        iss: ISSUER.to_owned(),
        sub: "agent".to_owned(),
        act: Box::new(None),
    });
    let token = authority()
        .sign_claims(&serde_json::to_value(&claims).expect("json"))
        .expect("sign");
    assert!(matches!(
        verify_desktop(authority().as_ref(), &token),
        Err(AuthError::NotAPerson)
    ));
}

#[test]
fn a_token_of_another_audience_is_rejected_before_the_policy() {
    let mcp_only = request(vec![Permission::User], vec![JwtAudience::Mcp]);
    assert!(matches!(
        verify_desktop(authority().as_ref(), &mint(&mcp_only)),
        Err(AuthError::Rejected(_))
    ));
}
