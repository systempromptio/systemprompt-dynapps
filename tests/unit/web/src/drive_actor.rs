//! The `/drive` actor rules: only a token core signed `browser_session`,
//! sent as the session cookie with no `Authorization` header, is the person,
//! and it acts only from this site's pages or a top-level link.

use axum::http::header::{ACCEPT, AUTHORIZATION, COOKIE, ORIGIN};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method};
use systemprompt::identifiers::{ClientId, SessionId};
use systemprompt::models::auth::{
    CustomClaims, JwtAudience, JwtClaims, Permission, RegisteredClaims, TokenType, UserType,
};
use systemprompt_web_admin_platform::test_support::{
    REAUTH_KEY, cookie_token, is_browser_session, may_act_from, wants_html,
};

const SEC_FETCH_SITE: HeaderName = HeaderName::from_static("sec-fetch-site");
const SEC_FETCH_MODE: HeaderName = HeaderName::from_static("sec-fetch-mode");

fn web_session() -> JwtClaims {
    JwtClaims {
        registered: RegisteredClaims {
            sub: "user-1".to_owned(),
            iat: 0,
            exp: i64::MAX,
            nbf: None,
            iss: "test".to_owned(),
            aud: vec![JwtAudience::Api],
            jti: "jti-1".to_owned(),
        },
        custom: CustomClaims {
            scope: vec![Permission::User],
            username: "person".to_owned(),
            email: "person@example.com".to_owned(),
            user_type: UserType::User,
            roles: Vec::new(),
            attributes: std::collections::BTreeMap::new(),
            client_id: None,
            token_type: TokenType::Bearer,
            auth_time: 0,
            session_id: Some(SessionId::new("session-1")),
            rate_limit_tier: None,
            plugin_id: None,
            act: None,
            purpose: None,
            interaction: None,
            browser_session: true,
        },
    }
}

fn with(edit: impl FnOnce(&mut CustomClaims)) -> JwtClaims {
    let mut claims = web_session();
    edit(&mut claims.custom);
    claims
}

fn headers(pairs: &[(HeaderName, &'static str)]) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (name, value) in pairs {
        map.append(name.clone(), HeaderValue::from_static(value));
    }
    map
}

#[test]
fn only_a_signed_browser_session_is_the_person() {
    assert!(is_browser_session(&web_session()));
    assert!(!is_browser_session(&with(|c| c.browser_session = false)));
    assert!(!is_browser_session(&with(|c| {
        c.browser_session = false;
        c.client_id = Some(ClientId::bridge());
    })));
}

#[test]
fn the_token_is_read_only_from_the_session_cookie() {
    let cookie = headers(&[(COOKIE, "theme=dark; access_token=tok")]);
    assert_eq!(cookie_token(&cookie).as_deref(), Some("tok"));
}

#[test]
fn an_authorization_header_disqualifies_the_cookie() {
    let both = headers(&[
        (COOKIE, "access_token=tok"),
        (AUTHORIZATION, "Bearer other"),
    ]);
    assert_eq!(cookie_token(&both), None);
}

#[test]
fn no_or_empty_cookie_is_no_session() {
    assert_eq!(cookie_token(&HeaderMap::new()), None);
    assert_eq!(cookie_token(&headers(&[(COOKIE, "access_token=")])), None);
}

fn site() -> Vec<String> {
    vec!["https://astound.example".to_owned()]
}

#[test]
fn a_same_origin_request_may_act() {
    let fetch = headers(&[(SEC_FETCH_SITE, "same-origin")]);
    assert!(may_act_from(&Method::POST, &fetch, &site()));
    assert!(
        !may_act_from(&Method::PUT, &HeaderMap::new(), &site()),
        "core's same_origin fails closed on a request that names no origin"
    );
}

#[test]
fn a_cross_site_write_or_fetch_may_not_act() {
    let cross = headers(&[(SEC_FETCH_SITE, "cross-site"), (SEC_FETCH_MODE, "cors")]);
    assert!(!may_act_from(&Method::PUT, &cross, &site()));
    assert!(!may_act_from(&Method::GET, &cross, &site()));
    let foreign = headers(&[(ORIGIN, "https://evil.example")]);
    assert!(!may_act_from(&Method::PUT, &foreign, &site()));
    let own = headers(&[(ORIGIN, "https://astound.example")]);
    assert!(may_act_from(&Method::PUT, &own, &site()));
}

#[test]
fn a_cross_site_link_still_opens_a_page() {
    let link = headers(&[(SEC_FETCH_SITE, "cross-site"), (SEC_FETCH_MODE, "navigate")]);
    assert!(may_act_from(&Method::GET, &link, &site()));
    assert!(!may_act_from(&Method::POST, &link, &site()));
}

#[test]
fn a_page_load_is_sent_to_sign_in_and_a_script_call_gets_reauth() {
    let page = headers(&[(ACCEPT, "text/html,application/xhtml+xml,*/*;q=0.8")]);
    let script = headers(&[(ACCEPT, "application/json")]);
    assert!(wants_html(&page));
    assert!(!wants_html(&script));
    assert!(!wants_html(&HeaderMap::new()));
    assert_eq!(REAUTH_KEY, "reauth");
}
