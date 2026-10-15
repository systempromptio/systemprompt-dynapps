//! Standards edges of the Entra ID `OpenID Connect` login and the connector
//! OAuth client: OIDC Core §3.1.3.7 `azp`, `email_verified` semantics for
//! Entra's mutable `email`, boolean claims sent as strings, group overage via
//! `hasgroups`, the `{tenantid}` discovery issuer template, and the RFC 9207
//! `iss` authorization-response parameter.

use serde_json::{Value, json};
use systemprompt::sso::SsoError;
use systemprompt_web_admin_identity::sso::entra_oidc::{
    EntraOidcConfig, IdTokenClaims, authorization_response_issuer_matches, identity_from_claims,
    resolve_issuer_template,
};
use systemprompt_web_admin_platform::connector_oauth::generic::authorization_response_issuer_matches as connector_iss_matches;

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

// An Entra v2.0 id_token as issued, including members sign-in never reads.
fn entra_claims() -> Value {
    json!({
        "ver": "2.0",
        "iss": format!("https://login.microsoftonline.com/{TENANT}/v2.0"),
        "sub": "AAAAAAAAAAAAAAAAAAAAAIkzqFVrSaSaFHy782bbtaQ",
        "aud": "client-123",
        "exp": 1_900_000_000,
        "iat": 1_899_996_400,
        "nbf": 1_899_996_400,
        "nonce": NONCE,
        "tid": TENANT,
        "oid": "00000000-0000-0000-66f3-3332eca7ea81",
        "email": "ana@dynapps.es",
        "preferred_username": "ana@dynapps.es",
        "name": "Ana Lopez",
        "groups": ["g-admins"],
        "rh": "0.AXEA...",
        "uti": "Ckm1t0xHRUmHfMIkjrUtAQ",
        "aio": "AWQAm/8TAAAA",
    })
}

fn identity(raw: &Value) -> Result<systemprompt::sso::ExternalIdentity, SsoError> {
    let claims: IdTokenClaims = serde_json::from_value(raw.clone()).expect("claims parse");
    identity_from_claims(&claims, &config(), NONCE, raw.clone())
}

#[test]
fn unknown_entra_claims_are_ignored() {
    assert!(identity(&entra_claims()).is_ok());
}

#[test]
fn multiple_audiences_require_azp_naming_this_client() {
    let mut raw = entra_claims();
    raw["aud"] = json!(["client-123", "api://other"]);
    let err = identity(&raw).expect_err("multi-aud without azp");
    assert!(matches!(err, SsoError::InvalidAssertion(ref m) if m == "azp"));

    raw["azp"] = json!("someone-else");
    assert!(identity(&raw).is_err(), "azp naming another client");

    raw["azp"] = json!("client-123");
    assert!(identity(&raw).is_ok());
}

#[test]
fn single_audience_ignores_azp() {
    let mut raw = entra_claims();
    raw["azp"] = json!("another-client");
    assert!(identity(&raw).is_ok());
}

#[test]
fn a_tenant_member_email_is_verified() {
    assert!(identity(&entra_claims()).expect("identity").email_verified);
    let mut raw = entra_claims();
    raw["idp"] = json!(format!("https://sts.windows.net/{TENANT}/"));
    assert!(identity(&raw).expect("identity").email_verified);
}

#[test]
fn trust_guest_email_lets_a_guest_email_link() {
    let mut raw = entra_claims();
    raw["idp"] = json!("live.com");
    let claims: IdTokenClaims = serde_json::from_value(raw.clone()).expect("claims parse");
    let cfg = EntraOidcConfig {
        trust_guest_email: true,
        ..config()
    };
    let identity = identity_from_claims(&claims, &cfg, NONCE, raw).expect("identity");
    assert!(identity.email_verified);
}

#[test]
fn a_guest_email_is_unverified_unless_entra_vouches() {
    let mut raw = entra_claims();
    raw["idp"] = json!("https://sts.windows.net/72f988bf-86f1-41af-91ab-2d7cd011db47/");
    assert!(!identity(&raw).expect("identity").email_verified);

    raw["idp"] = json!("live.com");
    assert!(!identity(&raw).expect("identity").email_verified);

    raw["xms_edov"] = json!("true");
    assert!(identity(&raw).expect("identity").email_verified);

    raw["xms_edov"] = json!(false);
    raw["email_verified"] = json!(true);
    assert!(identity(&raw).expect("identity").email_verified);
}

#[test]
fn boolean_claims_tolerate_strings_and_garbage() {
    let mut raw = entra_claims();
    raw["xms_edov"] = json!(1);
    raw["email_verified"] = json!("yes");
    raw["hasgroups"] = json!(null);
    let claims: IdTokenClaims = serde_json::from_value(raw).expect("claims parse");
    assert!(!claims.xms_edov && !claims.email_verified && !claims.hasgroups);
}

#[test]
fn hasgroups_signals_overage() {
    let mut raw = entra_claims();
    raw.as_object_mut().expect("object").remove("groups");
    raw["hasgroups"] = json!(true);
    assert!(identity(&raw).expect("identity").groups_overage);
}

#[test]
fn issuer_template_resolves_to_the_tenant() {
    assert_eq!(
        resolve_issuer_template("https://login.microsoftonline.com/{tenantid}/v2.0", TENANT),
        format!("https://login.microsoftonline.com/{TENANT}/v2.0")
    );
    let concrete = format!("https://login.microsoftonline.com/{TENANT}/v2.0");
    assert_eq!(resolve_issuer_template(&concrete, TENANT), concrete);
}

#[test]
fn authorization_response_iss_must_name_the_tenant_issuer() {
    let configured = format!("https://login.microsoftonline.com/{TENANT}/v2.0");
    let template = "https://login.microsoftonline.com/{tenantid}/v2.0";
    assert!(authorization_response_issuer_matches(
        &configured,
        template,
        &configured,
        TENANT
    ));
    assert!(!authorization_response_issuer_matches(
        "https://login.microsoftonline.com/72f988bf-86f1-41af-91ab-2d7cd011db47/v2.0",
        template,
        &configured,
        TENANT
    ));
}

#[test]
fn connector_callback_iss_is_checked_only_when_both_sides_have_one() {
    assert!(connector_iss_matches(None, "https://auth.example.com"));
    assert!(connector_iss_matches(Some("https://evil.example"), ""));
    assert!(connector_iss_matches(
        Some("https://auth.example.com/"),
        "https://auth.example.com"
    ));
    assert!(!connector_iss_matches(
        Some("https://evil.example"),
        "https://auth.example.com"
    ));
}
