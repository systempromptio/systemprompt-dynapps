//! The typed analytics tools turn a flat input into one exact CLI argv. What
//! is pinned here: every field has the documented default, unknown fields are
//! refused rather than ignored, limits clamp to the page bound, and no value
//! can smuggle a second flag into the command line.

use serde_json::json;
use systemprompt_mcp_agent::typed::{
    ConversationAuditInput, MAX_PAGE, RequestLogInput, UsageByUserInput, UsersInput,
};

fn parse<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> T {
    serde_json::from_value(value).expect("valid input")
}

#[test]
fn usage_by_user_defaults_to_seven_days_and_fifty_rows() {
    let input: UsageByUserInput = parse(json!({}));
    assert_eq!(input.since, "7d");
    assert_eq!(input.until, "");
    assert_eq!(input.limit, 50);
}

#[test]
fn request_log_defaults_to_thirty_days_newest_first() {
    let input: RequestLogInput = parse(json!({}));
    assert_eq!(input.since, "30d");
    assert_eq!(input.limit, 50);
    assert!(input.user.is_empty() && input.cursor.is_empty());
}

#[test]
fn conversation_audit_requires_the_request_id_and_bounds_bodies() {
    assert!(serde_json::from_value::<ConversationAuditInput>(json!({})).is_err());
    let input: ConversationAuditInput = parse(json!({"request_id": "req_1"}));
    assert!(input.messages);
    assert!(!input.tools);
    assert_eq!(input.offset, 0);
    assert_eq!(input.limit, 20);
    assert_eq!(input.max_chars, 800);
}

#[test]
fn users_defaults_to_the_first_fifty() {
    let input: UsersInput = parse(json!({}));
    assert_eq!((input.limit, input.offset), (50, 0));
}

#[test]
fn unknown_fields_are_refused_not_ignored() {
    for value in [
        json!({"days": 7}),
        json!({"since": "7d", "export": "x.csv"}),
        json!({"request_id": "r", "format": "json"}),
    ] {
        assert!(
            serde_json::from_value::<UsageByUserInput>(value.clone()).is_err()
                || serde_json::from_value::<ConversationAuditInput>(value.clone()).is_err(),
            "{value} must not deserialize silently"
        );
    }
}

#[test]
fn request_log_builds_the_exact_argv_and_clamps_the_page() {
    let input: RequestLogInput = parse(json!({
        "since": "2026-09-08",
        "until": "2026-09-12",
        "user": "8a1ece9f-ff46-436e-99d3-21b589ac57f3",
        "model": "opus",
        "limit": 500,
        "cursor": "2026-09-11T09:15:02.113204Z@req_last"
    }));
    let command = systemprompt_mcp_agent::typed::request_log_command(&input).unwrap();
    assert_eq!(
        command,
        format!(
            "infra logs request list --since 2026-09-08 --limit {MAX_PAGE} --until 2026-09-12 \
             --user 8a1ece9f-ff46-436e-99d3-21b589ac57f3 --model opus \
             --before 2026-09-11T09:15:02.113204Z@req_last"
        )
    );
}

#[test]
fn usage_by_user_and_users_build_their_argv() {
    let usage: UsageByUserInput = parse(json!({"since": "14d", "limit": 10}));
    assert_eq!(
        systemprompt_mcp_agent::typed::usage_by_user_command(&usage).unwrap(),
        "analytics costs breakdown --by user --since 14d --limit 10"
    );
    let users: UsersInput = parse(json!({"offset": 50, "role": "admin"}));
    assert_eq!(
        systemprompt_mcp_agent::typed::users_command(&users).unwrap(),
        "admin users list --limit 50 --offset 50 --role admin"
    );
}

#[test]
fn conversation_audit_builds_its_argv_with_opt_in_sections() {
    let input: ConversationAuditInput = parse(json!({
        "request_id": "req_1", "tools": true, "offset": 20, "limit": 5, "max_chars": 300
    }));
    assert_eq!(
        systemprompt_mcp_agent::typed::conversation_audit_command(&input).unwrap(),
        "infra logs audit req_1 --offset 20 --limit 5 --max-content 300 --messages --tools"
    );
    let header_only: ConversationAuditInput =
        parse(json!({"request_id": "req_1", "messages": false}));
    assert_eq!(
        systemprompt_mcp_agent::typed::conversation_audit_command(&header_only).unwrap(),
        "infra logs audit req_1 --offset 0 --limit 20 --max-content 800"
    );
}

#[test]
fn a_value_cannot_smuggle_a_second_flag() {
    for bad in ["7d --export x.csv", "--json", "a b", "x\ny"] {
        let input: RequestLogInput = parse(json!({"user": bad}));
        assert!(
            systemprompt_mcp_agent::typed::request_log_command(&input).is_err(),
            "{bad:?} must be rejected"
        );
    }
    let blank: ConversationAuditInput = parse(json!({"request_id": "  "}));
    assert!(systemprompt_mcp_agent::typed::conversation_audit_command(&blank).is_err());
}
