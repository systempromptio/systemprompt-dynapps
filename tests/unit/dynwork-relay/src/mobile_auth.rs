//! The mobile sign-in's pure parts: PKCE S256 against RFC 7636's vector, the
//! authorize parameters, the callback, the branding read from the theme, the
//! escaped consent page, the mobile JWT and the secrets' prefixes.

use axum::http::header;
use dynwork_relay::auth::verify_desktop;
use dynwork_relay::credential::{
    AuthorizationCode, ConsentRequest, CredentialHash, MobileCredential,
};
use dynwork_relay::ids::DeviceId;
use dynwork_relay::mobile_auth::branding::Branding;
use dynwork_relay::mobile_auth::jwt;
use dynwork_relay::mobile_auth::page::{self, Consent};
use dynwork_relay::mobile_auth::params::{
    AuthorizeQuery, ParamError, callback, challenge_of, valid_verifier, validate,
};
use dynwork_relay::store::Account;
use systemprompt::identifiers::UserId;

use crate::support::authority;

const RFC_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const RFC_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

fn query() -> AuthorizeQuery {
    AuthorizeQuery {
        redirect_uri: Some("dynwork://auth/callback".to_owned()),
        code_challenge: Some(RFC_CHALLENGE.to_owned()),
        code_challenge_method: Some("S256".to_owned()),
        state: Some("a".repeat(43)),
        device_name: Some("  Pixel 9  ".to_owned()),
    }
}

#[test]
fn s256_matches_the_rfc_7636_vector_and_verifiers_are_bounded() {
    assert_eq!(challenge_of(RFC_VERIFIER), RFC_CHALLENGE);
    assert!(valid_verifier(RFC_VERIFIER));
    assert!(valid_verifier(&"~".repeat(128)));
    assert!(!valid_verifier(&"a".repeat(42)));
    assert!(!valid_verifier(&"a".repeat(129)));
    assert!(!valid_verifier(&format!("{}+", "a".repeat(43))));
}

#[test]
fn a_well_formed_authorize_request_is_accepted_and_trimmed() {
    let request = validate(query()).expect("valid");
    assert_eq!(request.challenge, RFC_CHALLENGE);
    assert_eq!(request.device_name, "Pixel 9");
    let defaulted = validate(AuthorizeQuery {
        code_challenge_method: None,
        device_name: None,
        ..query()
    })
    .expect("valid");
    assert_eq!(defaulted.device_name, "Phone");
}

#[test]
fn every_malformed_parameter_is_refused() {
    let cases = [
        (
            AuthorizeQuery {
                redirect_uri: None,
                ..query()
            },
            ParamError::RedirectUri,
        ),
        (
            AuthorizeQuery {
                redirect_uri: Some("dynwork://auth/callback/".to_owned()),
                ..query()
            },
            ParamError::RedirectUri,
        ),
        (
            AuthorizeQuery {
                redirect_uri: Some("http://127.0.0.1:1/cb".to_owned()),
                ..query()
            },
            ParamError::RedirectUri,
        ),
        (
            AuthorizeQuery {
                code_challenge_method: Some("plain".to_owned()),
                ..query()
            },
            ParamError::Method,
        ),
        (
            AuthorizeQuery {
                code_challenge: None,
                ..query()
            },
            ParamError::Challenge,
        ),
        (
            AuthorizeQuery {
                code_challenge: Some(format!("{}=", &RFC_CHALLENGE[..42])),
                ..query()
            },
            ParamError::Challenge,
        ),
        (
            AuthorizeQuery {
                state: None,
                ..query()
            },
            ParamError::State,
        ),
        (
            AuthorizeQuery {
                state: Some("a".repeat(31)),
                ..query()
            },
            ParamError::State,
        ),
        (
            AuthorizeQuery {
                state: Some("a".repeat(257)),
                ..query()
            },
            ParamError::State,
        ),
        (
            AuthorizeQuery {
                state: Some(format!("{}&x", "a".repeat(40))),
                ..query()
            },
            ParamError::State,
        ),
        (
            AuthorizeQuery {
                device_name: Some("a".repeat(65)),
                ..query()
            },
            ParamError::DeviceName,
        ),
        (
            AuthorizeQuery {
                device_name: Some("bad\u{7}".to_owned()),
                ..query()
            },
            ParamError::DeviceName,
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(validate(input), Err(expected));
    }
}

#[test]
fn the_callback_is_the_constant_uri_with_encoded_values() {
    assert_eq!(
        callback(&[("code", "dwa1_x-y"), ("state", "a b&c")]),
        "dynwork://auth/callback?code=dwa1_x-y&state=a%20b%26c"
    );
}

#[test]
fn branding_is_read_from_the_theme_and_unsafe_values_fall_back() {
    let theme = "branding:\n  display_name: \"Dynapps\"\n  theme_color: \"#01007C\"\n  logo_light: \"/files/images/logo.svg\"\ncolors: {}\n";
    let branding = Branding::from_theme_yaml(theme);
    assert_eq!(branding.display_name, "Dynapps");
    assert_eq!(branding.theme_color, "#01007C");
    assert_eq!(branding.logo.as_deref(), Some("/files/images/logo.svg"));
    let hostile = "branding:\n  display_name: \"\"\n  theme_color: \"red;}</style>\"\n  logo_light: \"//evil.example/x.svg\"\n";
    let safe = Branding::from_theme_yaml(hostile);
    assert_eq!(safe, Branding::default());
    assert_eq!(
        Branding::from_theme_yaml("::: not yaml"),
        Branding::default()
    );
    assert_eq!(
        Branding::load(std::path::Path::new("/nonexistent")),
        Branding::default()
    );
}

#[tokio::test]
async fn the_consent_page_escapes_every_value_and_carries_its_security_headers() {
    let branding = Branding {
        display_name: "Acme <Corp>".to_owned(),
        ..Branding::default()
    };
    let response = page::consent(&Consent {
        branding: &branding,
        device_name: "\"><script>alert(1)</script>",
        account: "a&b@example.test",
        request: "dwq1_token",
    });
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .expect("a body");
    let html = String::from_utf8_lossy(&body);
    assert!(!html.contains("<script>"));
    assert!(html.contains("&quot;&gt;&lt;script&gt;"));
    assert!(html.contains("Acme &lt;Corp&gt;") && html.contains("a&amp;b@example.test"));
    assert!(html.contains(r#"action="/api/dynwork/mobile/v1/authorize/decision""#));
    assert!(html.contains(r#"value="dwq1_token""#));
    assert_eq!(headers[header::X_FRAME_OPTIONS], "DENY");
    assert_eq!(headers[header::REFERRER_POLICY], "no-referrer");
    assert!(
        headers[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap_or_default()
            .contains("default-src 'none'")
    );
}

#[test]
fn a_mobile_jwt_names_its_device_and_is_no_desktop_token() {
    let authority = authority();
    let account = Account {
        user: UserId::try_new("alice").expect("a user id"),
        name: "alice".to_owned(),
        email: "alice@example.test".to_owned(),
        roles: vec!["user".to_owned()],
    };
    let device = DeviceId::generate();
    let token = jwt::mint(authority.as_ref(), &account, device).expect("mint");
    assert_eq!(token.expires_in, 3600);
    let identity = jwt::verify(authority.as_ref(), &token.token).expect("verify");
    assert_eq!(identity.user, account.user);
    assert_eq!(identity.device, device);
    assert_eq!(identity.email, "alice@example.test");
    assert!(verify_desktop(authority.as_ref(), &token.token).is_err());
    assert!(authority.verify_session_cookie(&token.token).is_err());
    assert!(jwt::verify(authority.as_ref(), "eyJ.not.a-token").is_err());
}

#[test]
fn mobile_secrets_are_recognised_only_by_their_own_prefix() {
    let (credential, hash) = MobileCredential::mint();
    assert_eq!(
        CredentialHash::of_mobile_credential(credential.expose()),
        Some(hash)
    );
    assert_eq!(CredentialHash::of_presented(credential.expose()), None);
    let (code, _) = AuthorizationCode::mint();
    let (request, _) = ConsentRequest::mint();
    assert!(CredentialHash::of_code(code.expose()).is_some());
    assert_eq!(CredentialHash::of_code(request.expose()), None);
    assert_eq!(CredentialHash::of_consent_request(code.expose()), None);
    assert_eq!(CredentialHash::of_mobile_credential(code.expose()), None);
    assert_eq!(format!("{credential:?}"), "MobileCredential(<redacted>)");
    assert!(!format!("{code:?}{request:?}").contains("dw"));
}
