use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use chrono::{TimeZone, Utc};
use dynwork_chats::error::ChatsError;
use dynwork_chats::http::params::{ThreadInput, if_match, page_size};
use dynwork_chats::http::wire::{WireThread, message};
use dynwork_chats::ids::{IdError, MessageId, ThreadId};
use dynwork_chats::store::{MessageRecord, ModelRef, Role, ThreadRecord};
use dynwork_relay::auth::AuthError;
use serde_json::json;
use systemprompt::identifiers::UserId;

fn with_if_match(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::IF_MATCH,
        HeaderValue::from_str(value).expect("a header"),
    );
    headers
}

#[test]
fn if_match_takes_a_bare_quoted_or_weak_revision() {
    assert_eq!(if_match(&HeaderMap::new()).expect("absent"), None);
    for (value, revision) in [("3", 3), ("\"3\"", 3), ("W/\"12\"", 12), (" 7 ", 7)] {
        assert_eq!(
            if_match(&with_if_match(value)).expect(value),
            Some(revision),
            "{value}"
        );
    }
    for value in ["", "0", "-2", "x", "\"5", "*", "1.5"] {
        let refused = if_match(&with_if_match(value)).expect_err(value);
        assert_eq!(refused.key(), "bad_request", "{value}");
    }
}

#[test]
fn a_page_size_defaults_caps_and_refuses_zero() {
    assert_eq!(page_size(None, 50, 200).expect("default"), 50);
    assert_eq!(page_size(Some(10), 50, 200).expect("asked"), 10);
    assert_eq!(page_size(Some(9_999), 50, 200).expect("capped"), 200);
    assert_eq!(
        page_size(Some(0), 50, 200).expect_err("zero").status(),
        StatusCode::BAD_REQUEST
    );
}

#[test]
fn a_thread_body_is_parsed_and_checked() {
    let empty = ThreadInput::parse(b"  \n").expect("an empty body");
    assert!(empty.title.is_none() && empty.model.is_none() && empty.archived.is_none());

    let input = ThreadInput::parse(
        br#"{"title":"t","model":{"providerID":"p","id":"m"},"archived":false,"extra":1}"#,
    )
    .expect("a full body");
    let patch = input.into_patch();
    assert_eq!(patch.title.as_deref(), Some("t"));
    assert_eq!(
        patch.model,
        Some(ModelRef {
            provider_id: "p".to_owned(),
            model_id: "m".to_owned()
        })
    );
    assert_eq!(patch.archived, Some(false));

    let at_limit = format!("{{\"title\":\"{}\"}}", "é".repeat(512));
    assert!(ThreadInput::parse(at_limit.as_bytes()).is_ok());
    let over = format!("{{\"title\":\"{}\"}}", "é".repeat(513));
    let long_model = format!(
        r#"{{"model":{{"providerID":"p","id":"{}"}}}}"#,
        "m".repeat(129)
    );
    for body in [
        over.as_str(),
        long_model.as_str(),
        r#"{"model":{"providerID":" ","id":"m"}}"#,
        r#"{"title":null,"archived":"yes"}"#,
        "nope",
    ] {
        let refused = ThreadInput::parse(body.as_bytes()).expect_err(body);
        assert_eq!(refused.key(), "bad_request", "{body}");
    }
}

#[test]
fn every_http_error_has_its_status_and_key() {
    let record = thread(None);
    let cases: Vec<(ChatsError, StatusCode, &str)> = vec![
        (
            ChatsError::Id(IdError::Thread),
            StatusCode::BAD_REQUEST,
            "invalid_id",
        ),
        (
            ChatsError::Id(IdError::Cursor),
            StatusCode::BAD_REQUEST,
            "invalid_cursor",
        ),
        (
            ChatsError::Auth(AuthError::Missing),
            StatusCode::UNAUTHORIZED,
            "unauthorized",
        ),
        (
            ChatsError::Auth(AuthError::UnknownDevice),
            StatusCode::UNAUTHORIZED,
            "unauthorized",
        ),
        (
            ChatsError::BadRequest("x"),
            StatusCode::BAD_REQUEST,
            "bad_request",
        ),
        (
            ChatsError::TooLarge,
            StatusCode::PAYLOAD_TOO_LARGE,
            "too_large",
        ),
        (ChatsError::NotFound, StatusCode::NOT_FOUND, "not_found"),
        (
            ChatsError::PreconditionRequired,
            StatusCode::PRECONDITION_REQUIRED,
            "precondition_required",
        ),
        (
            ChatsError::Conflict(Box::new(record)),
            StatusCode::CONFLICT,
            "revision_conflict",
        ),
        (ChatsError::IdTaken, StatusCode::CONFLICT, "id_taken"),
        (ChatsError::Busy, StatusCode::CONFLICT, "busy"),
        (
            ChatsError::AiUnavailable,
            StatusCode::SERVICE_UNAVAILABLE,
            "ai_unavailable",
        ),
        (
            ChatsError::Devices(dynwork_relay::store::StoreError::Corrupt("x")),
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
        ),
    ];
    for (error, status, key) in cases {
        assert_eq!(error.status(), status, "{error:?}");
        assert_eq!(error.key(), key, "{error:?}");
    }
}

fn thread(deleted: Option<i64>) -> ThreadRecord {
    let at = |s: i64| {
        Utc.timestamp_opt(1_800_000_000 + s, 0)
            .single()
            .expect("a time")
    };
    ThreadRecord {
        id: "ses_01WIRE000000000000000000".parse().expect("a thread id"),
        user: UserId::new("alice".to_owned()),
        title: "t".to_owned(),
        model: Some(ModelRef {
            provider_id: "anthropic".to_owned(),
            model_id: "claude".to_owned(),
        }),
        revision: 3,
        archived_at: Some(at(5)),
        deleted_at: deleted.map(at),
        created_at: at(0),
        updated_at: at(9),
        change_seq: 4,
        last_event_seq: 0,
    }
}

#[test]
fn a_thread_on_the_wire_is_the_opencode_subset_plus_the_feed_fields() {
    let wire = serde_json::to_value(WireThread::from(&thread(None))).expect("serialize");
    assert_eq!(wire["id"], "ses_01WIRE000000000000000000");
    assert_eq!(
        wire["model"],
        json!({ "providerID": "anthropic", "id": "claude" })
    );
    assert_eq!(wire["time"]["created"], 1_800_000_000_000i64);
    assert_eq!(wire["time"]["updated"], 1_800_000_009_000i64);
    assert_eq!(wire["time"]["archived"], 1_800_000_005_000i64);
    assert_eq!(wire["deleted"], false);
    assert_eq!(wire["created_at"], "2027-01-15T08:00:00Z");
    assert!(wire["deleted_at"].is_null());
    assert!(wire.get("user").is_none() && wire.get("change_seq").is_none());

    let tombstone = serde_json::to_value(WireThread::from(&thread(Some(20)))).expect("serialize");
    assert_eq!(tombstone["deleted"], true);
    assert!(tombstone["deleted_at"].is_string());
}

#[test]
fn a_message_on_the_wire_takes_its_id_and_type_from_the_row() {
    let thread: ThreadId = "ses_01WIRE000000000000000000".parse().expect("a thread id");
    let id: MessageId = "msg_01WIRE000000000000000000"
        .parse()
        .expect("a message id");
    let record = |info| MessageRecord {
        id: id.clone(),
        thread: thread.clone(),
        role: Role::Assistant,
        info,
        parts: json!([]),
        seq: 1,
        created_at: Utc::now(),
        completed_at: None,
    };
    let stored = message(&record(
        json!({ "id": "other", "type": "user", "content": [] }),
    ));
    assert_eq!(
        stored,
        json!({ "id": "msg_01WIRE000000000000000000", "type": "assistant", "content": [] })
    );
    let bare = message(&record(json!("not an object")));
    assert_eq!(
        bare,
        json!({ "id": "msg_01WIRE000000000000000000", "type": "assistant" })
    );
}
