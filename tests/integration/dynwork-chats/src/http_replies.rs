//! `POST /threads/{id}/prompt` and `POST /threads/{id}/interrupt`: a reply
//! starts, ends compacted into its message, is idempotent by `messageID`,
//! refuses a second prompt while it runs, keeps its partial text when
//! interrupted or failed, and a reply an earlier process left is settled.

use axum::http::StatusCode;
use dynwork_chats::ids::{MessageId, ThreadId};
use dynwork_chats::reply::document::{ReplyDoc, user_info};
use dynwork_chats::reply::events;
use dynwork_chats::store::{AppendOutcome, ModelRef, NewMessage, Role};
use serde_json::{Value, json};

use crate::fixture::at;
use crate::http_support::{App, desktop_token, http_suite, user};
use crate::reply_support::{WAIT, kinds};

const THREAD: &str = "ses_01REPLIES0000000000000001";
const ASK: &str = "msg_01PROMPT00000000000000001";
const ASK_AGAIN: &str = "msg_01PROMPT00000000000000002";

fn reply_id(answer: &crate::http_support::Answer) -> String {
    answer.body["assistantMessageID"]
        .as_str()
        .expect("an assistant message id")
        .to_owned()
}

fn assistant(messages: &[Value]) -> &Value {
    messages
        .iter()
        .find(|message| message["type"] == "assistant")
        .expect("an assistant message")
}

async fn prompt_answers_202_with_its_reply(app: &App) {
    app.create_thread("alice", THREAD).await;
    app.scripted(&["Hello ", "there."]);
    let answer = app.prompt("alice", THREAD, ASK, "Say hello").await;
    assert_eq!(answer.status, StatusCode::ACCEPTED, "{:?}", answer.body);
    let reply = reply_id(&answer);
    assert!(reply.parse::<MessageId>().is_ok(), "{reply}");
    app.settled(THREAD).await;
    let messages = app.messages("alice", THREAD).await;
    assert_eq!(messages.len(), 2);
    let (answered, asked) = (&messages[0], &messages[1]);
    assert_eq!(answered["id"], reply.as_str());
    assert_eq!(answered["type"], "assistant");
    assert_eq!(
        answered["content"],
        json!([{ "type": "text", "text": "Hello there." }])
    );
    assert_eq!(answered["finish"], "stop");
    assert_eq!(answered["rawFinish"], "end_turn");
    assert_eq!(answered["tokens"]["input"], 120);
    assert_eq!(
        answered["model"],
        json!({ "id": "claude-sonnet", "providerID": "anthropic" })
    );
    assert!(answered["time"]["completed"].is_i64());
    assert!(answered.get("error").is_none());
    assert_eq!(asked["id"], ASK);
    assert_eq!(asked["type"], "user");
    assert_eq!(asked["text"], "Say hello");
    assert_eq!(asked["metadata"]["assistantMessageID"], reply.as_str());
}

async fn the_same_message_id_twice_is_one_reply(app: &App) {
    app.create_thread("alice", THREAD).await;
    let script = app.script();
    let first = app.prompt("alice", THREAD, ASK, "once").await;
    assert_eq!(first.status, StatusCode::ACCEPTED);
    let retried = app.prompt("alice", THREAD, ASK, "once").await;
    assert_eq!(retried.status, StatusCode::ACCEPTED, "{:?}", retried.body);
    assert_eq!(reply_id(&retried), reply_id(&first));
    script.text("only once");
    drop(script);
    app.settled(THREAD).await;
    let after = app.prompt("alice", THREAD, ASK, "once").await;
    assert_eq!(after.status, StatusCode::ACCEPTED);
    assert_eq!(reply_id(&after), reply_id(&first));
    assert_eq!(app.model.requests().len(), 1);
    assert_eq!(app.messages("alice", THREAD).await.len(), 2);
}

async fn a_second_prompt_while_one_runs_is_409(app: &App) {
    app.create_thread("alice", THREAD).await;
    let script = app.script();
    let first = app.prompt("alice", THREAD, ASK, "first").await;
    assert_eq!(first.status, StatusCode::ACCEPTED);
    let second = app.prompt("alice", THREAD, ASK_AGAIN, "second").await;
    assert_eq!(second.status, StatusCode::CONFLICT);
    assert_eq!(second.key(), "busy");
    drop(script);
    app.settled(THREAD).await;
    app.scripted(&["now"]);
    let later = app.prompt("alice", THREAD, ASK_AGAIN, "second").await;
    assert_eq!(later.status, StatusCode::ACCEPTED, "{:?}", later.body);
    app.settled(THREAD).await;
}

async fn another_users_thread_is_404(app: &App) {
    app.create_thread("alice", THREAD).await;
    let prompt = app.prompt("bob", THREAD, ASK, "mine?").await;
    assert_eq!(prompt.status, StatusCode::NOT_FOUND);
    assert_eq!(prompt.key(), "not_found");
    let interrupt = app.interrupt("bob", THREAD).await;
    assert_eq!(interrupt.status, StatusCode::NOT_FOUND);
    let log = app.log("bob", THREAD, "?follow=1").await;
    assert_eq!(log.status, StatusCode::NOT_FOUND);
    let missing = app
        .prompt("alice", "ses_01NOSUCHTHREAD0000000001", ASK, "x")
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert!(app.model.requests().is_empty());
}

async fn without_a_model_service_it_is_503(app: &App) {
    app.create_thread("alice", THREAD).await;
    app.source.switch_off();
    let answer = app.prompt("alice", THREAD, ASK, "anyone?").await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(answer.key(), "ai_unavailable");
    assert!(app.messages("alice", THREAD).await.is_empty());
}

async fn malformed_prompts_are_400(app: &App) {
    app.create_thread("alice", THREAD).await;
    let path = format!("/threads/{THREAD}/prompt");
    for (body, key) in [
        (json!({ "messageID": ASK, "parts": [] }), "bad_request"),
        (
            json!({ "messageID": ASK, "parts": [{ "type": "text", "text": "  " }] }),
            "bad_request",
        ),
        (
            json!({ "messageID": ASK, "parts": [{ "type": "file", "url": "x" }] }),
            "bad_request",
        ),
        (
            json!({ "messageID": "nope", "parts": [{ "type": "text", "text": "hi" }] }),
            "bad_request",
        ),
        (
            json!({ "parts": [{ "type": "text", "text": "hi" }] }),
            "bad_request",
        ),
    ] {
        let answer = app
            .call(axum::http::Method::POST, &path)
            .bearer(&desktop_token("alice"))
            .json(&body)
            .send()
            .await;
        assert_eq!(answer.status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(answer.key(), key, "{body}");
    }
    assert!(app.model.requests().is_empty());
}

async fn a_model_error_ends_in_execution_failed(app: &App) {
    app.create_thread("alice", THREAD).await;
    let script = app.script();
    script.text("Partial ");
    script.fail("provider.error", "the provider is overloaded");
    drop(script);
    let answer = app.prompt("alice", THREAD, ASK, "fail please").await;
    assert_eq!(answer.status, StatusCode::ACCEPTED);
    app.settled(THREAD).await;
    let events = app
        .log("alice", THREAD, "?follow=0")
        .await
        .until_closed()
        .await;
    let last = events.last().expect("events");
    assert_eq!(last.kind(), "session.execution.failed");
    assert_eq!(
        last.data["data"]["error"],
        json!({ "type": "provider.error", "message": "the provider is overloaded" })
    );
    assert!(!kinds(&events).contains(&"session.step.ended"));
    let ended = events
        .iter()
        .find(|event| event.kind() == "session.text.ended");
    assert_eq!(ended.expect("text ended").data["data"]["text"], "Partial ");
    let messages = app.messages("alice", THREAD).await;
    let answered = assistant(&messages);
    assert_eq!(answered["finish"], "error");
    assert_eq!(answered["error"]["type"], "provider.error");
    assert_eq!(
        answered["content"],
        json!([{ "type": "text", "text": "Partial " }])
    );
}

async fn interrupt_keeps_the_partial_text(app: &App) {
    app.create_thread("alice", THREAD).await;
    let script = app.script();
    let answer = app.prompt("alice", THREAD, ASK, "go on forever").await;
    assert_eq!(answer.status, StatusCode::ACCEPTED);
    let mut log = app.log("alice", THREAD, "?follow=1").await;
    script.text("Working on it, ");
    loop {
        let event = log.next(WAIT).await.expect("an event before the interrupt");
        if event.kind() == "session.text.delta" {
            break;
        }
    }
    let stopped = app.interrupt("alice", THREAD).await;
    assert_eq!(stopped.status, StatusCode::NO_CONTENT);
    let rest = log.until_terminal().await;
    assert_eq!(
        kinds(&rest),
        ["session.text.ended", "session.execution.interrupted"]
    );
    assert_eq!(rest[0].data["data"]["text"], "Working on it, ");
    assert_eq!(rest[1].data["data"]["reason"], "user");
    drop(script);
    let messages = app.messages("alice", THREAD).await;
    let answered = assistant(&messages);
    assert_eq!(answered["finish"], "error");
    assert_eq!(
        answered["error"],
        json!({ "type": "aborted", "message": "Step interrupted" })
    );
    assert_eq!(
        answered["content"],
        json!([{ "type": "text", "text": "Working on it, " }])
    );
    app.scripted(&["again"]);
    let next = app.prompt("alice", THREAD, ASK_AGAIN, "next").await;
    assert_eq!(next.status, StatusCode::ACCEPTED, "{:?}", next.body);
    app.settled(THREAD).await;
}

async fn interrupt_with_nothing_running_is_204(app: &App) {
    app.create_thread("alice", THREAD).await;
    let answer = app.interrupt("alice", THREAD).await;
    assert_eq!(answer.status, StatusCode::NO_CONTENT);
}

async fn the_history_goes_to_the_model(app: &App) {
    app.create_thread("alice", THREAD).await;
    app.scripted(&["One"]);
    assert_eq!(
        app.prompt("alice", THREAD, ASK, "first").await.status,
        StatusCode::ACCEPTED
    );
    app.settled(THREAD).await;
    app.scripted(&["Two"]);
    assert_eq!(
        app.prompt("alice", THREAD, ASK_AGAIN, "second")
            .await
            .status,
        StatusCode::ACCEPTED
    );
    app.settled(THREAD).await;
    let requests = app.model.requests();
    assert_eq!(requests.len(), 2);
    let second = &requests[1];
    assert_eq!(second.user, user("alice"));
    assert_eq!(second.thread.as_str(), THREAD);
    let history: Vec<(Role, &str)> = second
        .history
        .iter()
        .map(|turn| (turn.role, turn.text.as_str()))
        .collect();
    assert_eq!(
        history,
        [
            (Role::User, "first"),
            (Role::Assistant, "One"),
            (Role::User, "second")
        ]
    );
    let events = app
        .log("alice", THREAD, "?follow=0")
        .await
        .until_closed()
        .await;
    let totals: Vec<&Value> = events
        .iter()
        .filter(|event| event.kind() == "session.usage.updated")
        .map(|event| &event.data["data"]["tokens"]["input"])
        .collect();
    assert_eq!(totals, [&json!(120), &json!(240)]);
    let created = kinds(&events)
        .iter()
        .filter(|kind| **kind == "session.created")
        .count();
    assert_eq!(created, 1);
}

async fn a_reply_left_by_an_earlier_process_is_settled(app: &App) {
    app.create_thread("alice", THREAD).await;
    let alice = user("alice");
    let thread: ThreadId = THREAD.parse().expect("a thread id");
    let asked: MessageId = ASK.parse().expect("a message id");
    let reply = MessageId::generate_at(at(5));
    let doc = ReplyDoc {
        id: reply.clone(),
        model: ModelRef {
            provider_id: "anthropic".to_owned(),
            model_id: "claude-sonnet".to_owned(),
        },
        created: at(5).timestamp_millis(),
    };
    for message in [
        NewMessage {
            id: asked.clone(),
            role: Role::User,
            info: user_info(&asked, "before the restart", doc.created, &reply),
            parts: json!([]),
            completed_at: Some(at(5)),
        },
        NewMessage {
            id: reply.clone(),
            role: Role::Assistant,
            info: doc.started(),
            parts: json!([]),
            completed_at: None,
        },
    ] {
        let outcome = app
            .store
            .append_message(&alice, &thread, &message, at(5))
            .await;
        assert!(
            matches!(outcome, Ok(AppendOutcome::Appended(_))),
            "{outcome:?}"
        );
    }
    let written = app
        .store
        .append_events(
            &alice,
            &thread,
            &[
                events::execution_started(&thread, &reply),
                events::text_started(&thread, &reply),
                events::text_delta(&thread, &reply, "Half"),
            ],
            at(6),
        )
        .await
        .expect("write the events");
    assert_eq!(written.map(|records| records.len()), Some(3));
    let events = app
        .log("alice", THREAD, "?follow=0")
        .await
        .until_closed()
        .await;
    assert_eq!(
        kinds(&events),
        [
            "session.execution.started",
            "session.text.started",
            "session.text.delta",
            "session.text.ended",
            "session.execution.failed",
        ]
    );
    assert_eq!(events[3].data["data"]["text"], "Half");
    assert_eq!(events[4].data["data"]["error"]["type"], "restarted");
    let messages = app.messages("alice", THREAD).await;
    let answered = assistant(&messages);
    assert_eq!(answered["error"]["type"], "restarted");
    assert_eq!(
        answered["content"],
        json!([{ "type": "text", "text": "Half" }])
    );
    assert!(answered["time"]["completed"].is_i64());
    app.scripted(&["after"]);
    let next = app
        .prompt("alice", THREAD, ASK_AGAIN, "after the restart")
        .await;
    assert_eq!(next.status, StatusCode::ACCEPTED, "{:?}", next.body);
    app.settled(THREAD).await;
}

http_suite!(
    prompt_answers_202_with_its_reply,
    the_same_message_id_twice_is_one_reply,
    a_second_prompt_while_one_runs_is_409,
    another_users_thread_is_404,
    without_a_model_service_it_is_503,
    malformed_prompts_are_400,
    a_model_error_ends_in_execution_failed,
    interrupt_keeps_the_partial_text,
    interrupt_with_nothing_running_is_204,
    the_history_goes_to_the_model,
    a_reply_left_by_an_earlier_process_is_settled,
);
