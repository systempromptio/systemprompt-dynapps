//! The pure half of the Entra ID OpenID Connect login: id_token claim shapes,
//! the claims-to-identity rules, the config defaults and the authorize URL —
//! everything that decides a login without a live tenant or a signing key.

use serde_json::{Value, json};
use systemprompt::sso::SsoError;
use systemprompt_web_admin_identity::sso::entra_oidc::{
    AuthorizeRequest, EntraOidcConfig, IdTokenClaims, build_authorize_url, identity_from_claims,
    pkce_challenge,
};

const TENANT: &str = "9188040d-6c67-4c5b-b112-36a304b66dad";
const NONCE: &str = "n-0S6_WzA2Mj";

fn config() -> EntraOidcConfig {
    EntraOidcConfig {
        enabled: true,
        entra_tenant_id: TENANT.to_owned(),
        entra_client_id: "client-123".to_owned(),
        redirect_uri: Some("https://example.test/admin/auth/entra/callback".to_owned()),
        ..EntraOidcConfig::default()
    }
}

fn raw_claims() -> Value {
    json!({
        "iss": format!("https://login.microsoftonline.com/{TENANT}/v2.0"),
        "sub": "AAAAAAAAAAAAAAAAAAAAAIkzqFVrSaSaFHy782bbtaQ",
        "aud": "client-123",
        "exp": 1_900_000_000,
        "iat": 1_899_996_400,
        "nonce": NONCE,
        "tid": TENANT,
        "email": "ana@dynapps.es",
        "name": "Ana Lopez",
        "groups": ["g-admins", "g-users"],
        "jti": "token-id-1",
    })
}

fn parse(raw: &Value) -> IdTokenClaims {
    serde_json::from_value(raw.clone()).expect("claims parse")
}

#[test]
fn aud_parses_as_string_or_array() {
    let mut raw = raw_claims();
    assert_eq!(parse(&raw).aud, vec!["client-123".to_owned()]);
    raw["aud"] = json!(["client-123", "other"]);
    assert_eq!(parse(&raw).aud.len(), 2);
}

#[test]
fn full_claims_map_to_identity() {
    let raw = raw_claims();
    let identity = identity_from_claims(&parse(&raw), &config(), NONCE, raw).expect("identity");
    assert_eq!(identity.email.as_deref(), Some("ana@dynapps.es"));
    assert!(identity.email_verified);
    assert_eq!(identity.display_name.as_deref(), Some("Ana Lopez"));
    assert_eq!(identity.groups, vec!["g-admins", "g-users"]);
    assert!(!identity.groups_overage);
    assert_eq!(identity.tenant_id.as_deref(), Some(TENANT));
    assert_eq!(identity.assertion_id.as_deref(), Some("token-id-1"));
    assert_eq!(
        identity.assertion_expires_at.map(|t| t.timestamp()),
        Some(1_900_000_000)
    );
}

#[test]
fn claim_names_signal_group_overage() {
    let mut raw = raw_claims();
    raw.as_object_mut().expect("object").remove("groups");
    raw["_claim_names"] = json!({"groups": "src1"});
    raw["_claim_sources"] =
        json!({"src1": {"endpoint": "https://graph.microsoft.com/v1.0/me/getMemberObjects"}});
    let identity = identity_from_claims(&parse(&raw), &config(), NONCE, raw).expect("identity");
    assert!(identity.groups_overage);
    assert!(identity.groups.is_empty());
}

#[test]
fn nonce_mismatch_is_refused() {
    let raw = raw_claims();
    let err = identity_from_claims(&parse(&raw), &config(), "other", raw).unwrap_err();
    assert!(matches!(err, SsoError::InvalidAssertion(ref m) if m == "nonce"));
}

#[test]
fn foreign_tenant_is_refused() {
    let mut raw = raw_claims();
    raw["tid"] = json!("72f988bf-86f1-41af-91ab-2d7cd011db47");
    let err = identity_from_claims(&parse(&raw), &config(), NONCE, raw).unwrap_err();
    assert!(matches!(err, SsoError::InvalidAssertion(ref m) if m == "tenant"));
}

#[test]
fn empty_subject_is_refused() {
    let mut raw = raw_claims();
    raw["sub"] = json!("");
    let err = identity_from_claims(&parse(&raw), &config(), NONCE, raw).unwrap_err();
    assert!(matches!(err, SsoError::NoSubject));
}

#[test]
fn preferred_username_is_used_only_when_it_is_an_address() {
    let mut raw = raw_claims();
    raw.as_object_mut().expect("object").remove("email");
    raw["preferred_username"] = json!("ana@wearedynapps.com");
    let identity =
        identity_from_claims(&parse(&raw), &config(), NONCE, raw.clone()).expect("identity");
    assert_eq!(identity.email.as_deref(), Some("ana@wearedynapps.com"));
    assert!(!identity.email_verified);

    raw["preferred_username"] = json!("ana");
    let identity = identity_from_claims(&parse(&raw), &config(), NONCE, raw).expect("identity");
    assert_eq!(identity.email, None);
}

#[test]
fn missing_jti_leaves_assertion_id_to_the_token_digest() {
    let mut raw = raw_claims();
    raw.as_object_mut().expect("object").remove("jti");
    let identity = identity_from_claims(&parse(&raw), &config(), NONCE, raw).expect("identity");
    assert_eq!(identity.assertion_id, None);
}

#[test]
fn custom_groups_claim_is_read() {
    let mut raw = raw_claims();
    raw["roles"] = json!(["App.Admin"]);
    let cfg = EntraOidcConfig {
        groups_claim: "roles".to_owned(),
        ..config()
    };
    let identity = identity_from_claims(&parse(&raw), &cfg, NONCE, raw).expect("identity");
    assert_eq!(identity.groups, vec!["App.Admin"]);
}

#[test]
fn config_defaults_and_usability() {
    let cfg: EntraOidcConfig = serde_yaml::from_str("enabled: true").expect("parse");
    assert_eq!(cfg.client_secret_secret, "entra_oidc_client_secret");
    assert_eq!(cfg.redirect_path, "/admin/auth/entra/callback");
    assert_eq!(cfg.scopes, vec!["openid", "profile", "email"]);
    assert_eq!(cfg.groups_claim, "groups");
    assert_eq!(cfg.clock_skew_seconds, 120);
    assert_eq!(cfg.jwks_allowed_hosts, vec!["login.microsoftonline.com"]);
    assert_eq!(cfg.policy.default_roles, vec!["user"]);
    assert!(!cfg.is_usable(), "no tenant, client or redirect_uri");

    assert!(config().is_usable());
    assert!(
        !EntraOidcConfig {
            enabled: false,
            ..config()
        }
        .is_usable()
    );
    assert!(
        !EntraOidcConfig {
            redirect_uri: None,
            ..config()
        }
        .is_usable()
    );
    assert_eq!(
        config().issuer(),
        format!("https://login.microsoftonline.com/{TENANT}/v2.0")
    );
    assert!(serde_yaml::from_str::<EntraOidcConfig>("bogus_key: 1").is_err());
}

#[test]
fn committed_yaml_parses_disabled() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../services/web/config/entra-oidc.yaml"
    );
    let text = std::fs::read_to_string(path).expect("entra-oidc.yaml");
    let cfg: EntraOidcConfig = serde_yaml::from_str(&text).expect("parse");
    assert!(!cfg.enabled);
    assert!(
        cfg.policy
            .group_roles
            .values()
            .any(|r| r.iter().any(|r| r == "admin"))
    );
}

#[test]
fn authorize_url_carries_every_parameter() {
    let scopes = vec!["openid".to_owned(), "email".to_owned()];
    let url = build_authorize_url(&AuthorizeRequest {
        authorization_endpoint: "https://login.microsoftonline.com/t/oauth2/v2.0/authorize",
        client_id: "client-123",
        redirect_uri: "https://example.test/admin/auth/entra/callback",
        scopes: &scopes,
        state: "st",
        nonce: "nn",
        code_challenge: &pkce_challenge("verifier"),
    });
    assert!(url.starts_with("https://login.microsoftonline.com/t/oauth2/v2.0/authorize?"));
    for expected in [
        "client_id=client-123",
        "response_type=code",
        "redirect_uri=https%3A%2F%2Fexample.test%2Fadmin%2Fauth%2Fentra%2Fcallback",
        "response_mode=query",
        "scope=openid%20email",
        "state=st",
        "nonce=nn",
        "code_challenge_method=S256",
        "prompt=select_account",
    ] {
        assert!(url.contains(expected), "{expected} missing from {url}");
    }
}

#[test]
fn pkce_challenge_matches_rfc7636_vector() {
    assert_eq!(
        pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGrBEU6WtsU"
    );
}
