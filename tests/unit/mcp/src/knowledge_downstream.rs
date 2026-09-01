//! The per-user Atlassian credential contract: `sync_knowledge` reads the
//! caller's e-mail and API token from request headers, names the missing
//! header when one is absent or blank, accepts the lower-cased names the
//! proxy forwards, and lets a header override the configured site.

use axum::http::{HeaderMap, HeaderValue};
use systemprompt::traits::ExtensionError;
use systemprompt_mcp_knowledge_bank::config::AtlassianSettings;
use systemprompt_mcp_knowledge_bank::downstream::{
    AtlassianCredentials, HEADER_API_TOKEN, HEADER_BASE_URL, HEADER_EMAIL,
};
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;

fn settings() -> AtlassianSettings {
    AtlassianSettings {
        base_url: "https://astounddigital.atlassian.net/".to_owned(),
        include_comments: true,
    }
}

fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (name, value) in pairs {
        map.insert(
            axum::http::HeaderName::from_bytes(name.as_bytes()).expect("header name"),
            HeaderValue::from_str(value).expect("header value"),
        );
    }
    map
}

#[test]
fn present_headers_become_the_callers_credentials_on_the_configured_site() {
    let creds = AtlassianCredentials::from_headers(
        &headers(&[
            (HEADER_EMAIL, " alice@astound.com "),
            (HEADER_API_TOKEN, "tok-123"),
        ]),
        &settings(),
    )
    .expect("credentials parse");
    assert_eq!(creds.email, "alice@astound.com");
    assert_eq!(creds.api_token, "tok-123");
    assert_eq!(
        creds.base_url, "https://astounddigital.atlassian.net",
        "trailing slash trimmed"
    );
}

#[test]
fn a_missing_or_blank_header_is_named_in_the_error() {
    let err = AtlassianCredentials::from_headers(
        &headers(&[(HEADER_EMAIL, "alice@astound.com")]),
        &settings(),
    )
    .expect_err("token missing");
    assert!(matches!(err, KnowledgeBankError::MissingHeader(name) if name == HEADER_API_TOKEN));
    assert_eq!(err.code(), "MISSING_HEADER");
    assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    let text = err.to_string();
    assert!(text.contains("x-atlassian-api-token"), "{text}");
    assert!(text.contains("project_context skill"), "{text}");

    let err = AtlassianCredentials::from_headers(
        &headers(&[(HEADER_EMAIL, "   "), (HEADER_API_TOKEN, "tok")]),
        &settings(),
    )
    .expect_err("blank e-mail is missing");
    assert!(matches!(err, KnowledgeBankError::MissingHeader(name) if name == HEADER_EMAIL));
}

#[test]
fn header_names_match_case_insensitively_as_the_proxy_forwards_them() {
    let creds = AtlassianCredentials::from_headers(
        &headers(&[
            ("X-Atlassian-Email", "bob@astound.com"),
            ("X-ATLASSIAN-API-TOKEN", "tok"),
        ]),
        &settings(),
    )
    .expect("HeaderMap lower-cases names");
    assert_eq!(creds.email, "bob@astound.com");
}

#[test]
fn a_base_url_header_overrides_the_configured_site_and_is_required_without_one() {
    let creds = AtlassianCredentials::from_headers(
        &headers(&[
            (HEADER_EMAIL, "a@b.c"),
            (HEADER_API_TOKEN, "tok"),
            (HEADER_BASE_URL, "https://other.atlassian.net/"),
        ]),
        &settings(),
    )
    .expect("parses");
    assert_eq!(creds.base_url, "https://other.atlassian.net");

    let err = AtlassianCredentials::from_headers(
        &headers(&[(HEADER_EMAIL, "a@b.c"), (HEADER_API_TOKEN, "tok")]),
        &AtlassianSettings::default(),
    )
    .expect_err("no site anywhere");
    assert!(matches!(err, KnowledgeBankError::Config(_)));
}

#[test]
fn debug_output_never_shows_the_token() {
    let creds = AtlassianCredentials::from_headers(
        &headers(&[(HEADER_EMAIL, "a@b.c"), (HEADER_API_TOKEN, "s3cret")]),
        &settings(),
    )
    .expect("parses");
    assert!(!format!("{creds:?}").contains("s3cret"));
}
