//! Pure halves of the Farm console's `client` module.

use systemprompt_web_admin_agent_farm::client::{FarmApi, is_agent_slug, is_task_id, status_error};

#[test]
fn debug_never_prints_the_token() {
    let api = FarmApi::new("http://farm.invalid/farm/api/v1/", "s3cr3t-value");
    let shown = format!("{api:?}");
    assert!(!shown.contains("s3cr3t-value"), "{shown}");
    assert!(shown.contains("http://farm.invalid/farm/api/v1"), "{shown}");
}

#[test]
fn blank_url_or_token_is_unconfigured() {
    assert!(!FarmApi::new("", "t").is_configured());
    assert!(!FarmApi::new("http://x", "  ").is_configured());
    assert!(FarmApi::new("http://x", "t").is_configured());
}

#[test]
fn task_ids_are_uuids_only() {
    assert!(is_task_id("01a17751-467a-7792-b87c-8b67e9d8fa36"));
    assert!(!is_task_id("../credentials"));
    assert!(!is_task_id(""));
    assert!(!is_task_id("abc/def"));
}

#[test]
fn agent_slugs_are_lowercase_words_only() {
    assert!(is_agent_slug("code-reviewer"));
    assert!(!is_agent_slug("../credentials"));
    assert!(!is_agent_slug("Code"));
    assert!(!is_agent_slug("a/b"));
    assert!(!is_agent_slug(""));
}

#[test]
fn status_errors_carry_the_farm_code() {
    let err = status_error(
        409,
        br#"{"error":{"code":"task_has_no_result","message":"m"}}"#,
    );
    assert_eq!(err.code(), Some("task_has_no_result"));
    assert_eq!(
        err.to_string(),
        "farm-api answered 409 task_has_no_result: m"
    );
    let err = status_error(500, b"Internal Server Error");
    assert_eq!(err.code(), None);
    assert_eq!(err.to_string(), "farm-api answered 500: no error body");
}
