use axum::http::{HeaderMap, HeaderValue};
use chrono::{TimeZone, Utc};
use dynwork_chats::http::log::{LogQuery, start_after, wants_follow};
use dynwork_chats::http::prompt::PromptInput;
use dynwork_chats::ids::{MessageId, ThreadId};
use dynwork_chats::reply::document::{
    Ending, Fold, ReplyDoc, history, message_text, prior_tokens, reply_of, user_info,
};
use dynwork_chats::reply::events::{self, Tokens, envelope, event_id};
use dynwork_chats::reply::model::{Usage, finish_of, raw_finish_of};
use dynwork_chats::store::{EventRecord, MessageRecord, ModelRef, Role};
use serde_json::{Value, json};

fn at(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 17, 9, 0, seconds as u32)
        .single()
        .expect("a valid instant")
}

fn thread() -> ThreadId {
    "ses_01UNITREPLY000000000000001"
        .parse()
        .expect("a thread id")
}

fn reply() -> MessageId {
    "msg_01UNITREPLY000000000000001"
        .parse()
        .expect("a message id")
}

fn model() -> ModelRef {
    ModelRef {
        provider_id: "anthropic".to_owned(),
        model_id: "claude-sonnet".to_owned(),
    }
}

fn record(kind: &str, data: Value, seq: i64) -> EventRecord {
    EventRecord {
        thread: thread(),
        seq,
        message: Some(reply()),
        kind: kind.to_owned(),
        data,
        created_at: at(1),
    }
}

#[test]
fn the_envelope_carries_location_and_durable_where_the_desktop_does() {
    let (thread, reply) = (thread(), reply());
    let built = [
        events::session_created(&thread, "title", &model()),
        events::execution_started(&thread, &reply),
        events::step_started(&thread, &reply, &model(), 5),
        events::text_started(&thread, &reply),
        events::text_delta(&thread, &reply, "x"),
        events::text_ended(&thread, &reply, "x"),
        events::step_ended(&thread, &reply, "stop", "end_turn", Tokens::default()),
        events::usage_updated(&thread, &reply, Tokens::default()),
        events::execution_succeeded(&thread, &reply),
    ];
    let expected = [
        (true, true),
        (false, true),
        (true, true),
        (true, true),
        (true, false),
        (true, true),
        (true, true),
        (false, false),
        (false, true),
    ];
    for (seq, (event, (located, durable))) in built.iter().zip(expected).enumerate() {
        let seq = i64::try_from(seq).expect("a seq") + 1;
        let sent = envelope(&record(&event.kind, event.data.clone(), seq));
        assert_eq!(sent["type"], event.kind.as_str());
        assert_eq!(sent["created"], at(1).timestamp_millis());
        assert_eq!(sent.get("location").is_some(), located, "{sent}");
        assert_eq!(sent.get("durable").is_some(), durable, "{sent}");
        if durable {
            assert_eq!(
                sent["durable"],
                json!({ "aggregateID": thread.as_str(), "seq": seq, "version": 1 })
            );
        }
        assert_eq!(sent["data"]["sessionID"], thread.as_str());
    }
    assert!(events::is_terminal(events::EXECUTION_INTERRUPTED));
    assert!(!events::is_terminal(events::STEP_ENDED));
}

#[test]
fn an_event_id_is_derived_from_its_row() {
    let first = record(events::TEXT_DELTA, json!({}), 7);
    let again = record(events::TEXT_DELTA, json!({ "other": true }), 7);
    let next = record(events::TEXT_DELTA, json!({}), 8);
    assert_eq!(event_id(&first), event_id(&again));
    assert_ne!(event_id(&first), event_id(&next));
    assert!(event_id(&first) < event_id(&next));
    assert!(event_id(&first).starts_with("evt_"));
}

#[test]
fn a_derived_message_id_is_stable_and_distinct_per_seed() {
    let one = MessageId::derived_at(at(3), "ses_a/msg_a");
    assert_eq!(one, MessageId::derived_at(at(3), "ses_a/msg_a"));
    assert_ne!(one, MessageId::derived_at(at(3), "ses_a/msg_b"));
    assert_ne!(one, MessageId::derived_at(at(4), "ses_a/msg_a"));
    assert!(one.as_str().parse::<MessageId>().is_ok());
    assert!(MessageId::derived_at(at(3), "x") < MessageId::derived_at(at(4), "x"));
}

#[test]
fn finish_reasons_map_to_opencode_and_unknown_ones_are_hidden() {
    for (raw, finish) in [
        ("end_turn", "stop"),
        ("stop", "stop"),
        ("max_tokens", "length"),
        ("tool_use", "tool-calls"),
        ("refusal", "content-filter"),
        ("something else", "unknown"),
    ] {
        assert_eq!(finish_of(raw), finish, "{raw}");
    }
    assert_eq!(raw_finish_of("end_turn"), "end_turn");
    assert_eq!(raw_finish_of("a provider's free text"), "other");
}

#[test]
fn usage_keeps_the_largest_counts_and_the_last_finish() {
    let mut fold = Fold::default();
    assert_eq!((fold.finish(), fold.raw_finish()), ("stop", "stop"));
    fold.add_usage(&Usage {
        input: 100,
        ..Usage::default()
    });
    fold.add_usage(&Usage {
        output: 9,
        raw_finish: Some("max_tokens"),
        ..Usage::default()
    });
    assert_eq!(fold.tokens().input, 100);
    assert_eq!(fold.tokens().output, 9);
    assert_eq!(fold.finish(), "length");
    let total = fold
        .tokens()
        .plus(Tokens::from_json(&fold.tokens().to_json()));
    assert_eq!(total.input, 200);
}

fn message(id: &str, role: Role, info: Value, done: bool) -> MessageRecord {
    MessageRecord {
        id: id.parse().expect("a message id"),
        thread: thread(),
        role,
        info,
        parts: json!([]),
        seq: 0,
        created_at: at(0),
        completed_at: done.then(|| at(1)),
    }
}

#[test]
fn the_history_is_oldest_first_finished_and_alternating() {
    let text = |words: &str| json!({ "content": [{ "type": "text", "text": words }, { "type": "reasoning", "text": "no" }], "tokens": { "input": 5, "output": 1 } });
    let newest_first = [
        message(
            "msg_0000000006",
            Role::User,
            json!({ "text": "now?" }),
            true,
        ),
        message(
            "msg_0000000005",
            Role::Assistant,
            json!({ "content": [] }),
            false,
        ),
        message(
            "msg_0000000004",
            Role::User,
            json!({ "text": "again" }),
            true,
        ),
        message(
            "msg_0000000003",
            Role::Assistant,
            json!({ "content": [] }),
            true,
        ),
        message("msg_0000000002", Role::Assistant, text("hello"), true),
        message("msg_0000000001", Role::User, json!({ "text": "hi" }), true),
    ];
    let turns: Vec<(Role, String)> = history(&newest_first)
        .into_iter()
        .map(|turn| (turn.role, turn.text))
        .collect();
    assert_eq!(
        turns,
        [
            (Role::User, "hi".to_owned()),
            (Role::Assistant, "hello".to_owned()),
            (Role::User, "again\n\nnow?".to_owned()),
        ]
    );
    assert_eq!(message_text(&newest_first[4]), "hello");
    let prior = prior_tokens(&newest_first);
    assert_eq!((prior.input, prior.output), (5, 1));
}

#[test]
fn a_stored_prompt_names_its_reply_and_a_finished_reply_its_ending() {
    let asked: MessageId = "msg_01UNITPROMPT00000000000001".parse().expect("an id");
    let stored = message(
        asked.as_str(),
        Role::User,
        user_info(&asked, "hi", 1, &reply()),
        true,
    );
    assert_eq!(reply_of(&stored), Some(reply()));
    assert_eq!(stored.info["text"], "hi");
    let doc = ReplyDoc {
        id: reply(),
        model: model(),
        created: 1,
    };
    assert_eq!(doc.started()["content"], json!([]));
    let fold = Fold {
        text: "partial".to_owned(),
        ..Fold::default()
    };
    let interrupted = doc.finished(&fold, &Ending::Interrupted, 9);
    assert_eq!(interrupted["finish"], "error");
    assert_eq!(interrupted["error"]["type"], "aborted");
    assert_eq!(interrupted["time"], json!({ "created": 1, "completed": 9 }));
    assert!(interrupted.get("rawFinish").is_none());
    let succeeded = doc.finished(&fold, &Ending::Succeeded, 9);
    assert_eq!(succeeded["finish"], "stop");
    assert_eq!(succeeded["rawFinish"], "stop");
    assert!(succeeded.get("error").is_none());
    assert_eq!(
        succeeded["content"],
        json!([{ "type": "text", "text": "partial" }])
    );
}

#[test]
fn a_prompt_body_is_text_with_an_optional_model() {
    let parsed = PromptInput::parse(
        br#"{"messageID":"msg_01UNITPROMPT00000000000001","parts":[{"type":"text","text":"a"},{"type":"text","text":"b"}],"model":{"providerID":"openai","id":"gpt"}}"#,
    )
    .expect("a prompt");
    assert_eq!(parsed.text, "a\nb");
    assert_eq!(
        parsed.model.map(|model| model.model_id),
        Some("gpt".to_owned())
    );
    let refused = PromptInput::parse(
        br#"{"messageID":"msg_01UNITPROMPT00000000000001","parts":[{"type":"text","text":"a"}],"model":{"providerID":"","id":"gpt"}}"#,
    )
    .expect_err("an empty provider");
    assert_eq!(refused.key(), "bad_request");
}

#[test]
fn a_log_query_resumes_after_the_later_seq() {
    let query = |after: Option<i64>, follow: Option<&str>| LogQuery {
        after,
        follow: follow.map(str::to_owned),
    };
    let mut headers = HeaderMap::new();
    assert_eq!(start_after(&query(None, None), &headers).expect("none"), 0);
    assert_eq!(
        start_after(&query(Some(4), None), &headers).expect("after"),
        4
    );
    headers.insert("last-event-id", HeaderValue::from_static("9"));
    assert_eq!(
        start_after(&query(Some(4), None), &headers).expect("both"),
        9
    );
    assert_eq!(
        start_after(&query(Some(12), None), &headers).expect("both"),
        12
    );
    headers.insert("last-event-id", HeaderValue::from_static("x"));
    assert!(start_after(&query(None, None), &headers).is_err());
    assert!(!wants_follow(&query(None, None)).expect("default"));
    assert!(wants_follow(&query(None, Some("1"))).expect("one"));
    assert!(!wants_follow(&query(None, Some("0"))).expect("zero"));
    assert!(wants_follow(&query(None, Some("yes"))).is_err());
}
