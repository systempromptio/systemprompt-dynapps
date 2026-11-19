//! `GET /threads/{id}/messages`: history pages newest first with `before`
//! and `more`, each message an `OpenCode` document with its row's `id` and
//! `type`, and what it refuses.

use axum::http::{Method, StatusCode};
use dynwork_chats::ids::{MessageId, ThreadId};
use dynwork_chats::store::{AppendOutcome, NewMessage, Role};
use serde_json::json;

use crate::fixture::at;
use crate::http_support::{Answer, App, desktop_token, http_suite, user};

const ID: &str = "ses_01HISTORY000000000000000";

async fn thread_with_messages(app: &App, count: i64) -> Vec<MessageId> {
    let created = app
        .call(Method::PUT, &format!("/threads/{ID}"))
        .bearer(&desktop_token("alice"))
        .json(&json!({ "title": "history" }))
        .send()
        .await;
    assert_eq!(created.status, StatusCode::OK);
    let thread: ThreadId = ID.parse().expect("a thread id");
    let mut ids = Vec::new();
    for n in 0..count {
        let role = if n % 2 == 0 {
            Role::User
        } else {
            Role::Assistant
        };
        let id = MessageId::generate_at(at(n));
        let message = NewMessage {
            id: id.clone(),
            role,
            info: json!({ "id": "ignored", "type": "ignored", "time": { "created": n }, "text": format!("m{n}") }),
            parts: json!([]),
            completed_at: None,
        };
        let outcome = app
            .store
            .append_message(&user("alice"), &thread, &message, at(100 + n))
            .await
            .expect("append a message");
        assert!(matches!(outcome, AppendOutcome::Appended(_)), "{outcome:?}");
        ids.push(id);
    }
    ids
}

async fn page(app: &App, query: &str) -> Answer {
    let answer = app
        .call(Method::GET, &format!("/threads/{ID}/messages{query}"))
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{query}: {:?}", answer.body);
    answer
}

fn ids(answer: &Answer) -> Vec<String> {
    answer.body["messages"]
        .as_array()
        .expect("a message list")
        .iter()
        .map(|message| message["id"].as_str().unwrap_or("").to_owned())
        .collect()
}

async fn history_pages_newest_first(app: &App) {
    let sent = thread_with_messages(app, 5).await;
    let name = |n: usize| sent[n].as_str().to_owned();

    let first = page(app, "?limit=2").await;
    assert_eq!(ids(&first), vec![name(4), name(3)]);
    assert_eq!(first.body["more"], true);
    assert_eq!(first.body["before"], name(3));
    assert_eq!(first.body["messages"][0]["type"], "user");
    assert_eq!(first.body["messages"][1]["type"], "assistant");
    assert_eq!(first.body["messages"][0]["text"], "m4");

    let second = page(app, &format!("?limit=2&before={}", name(3))).await;
    assert_eq!(ids(&second), vec![name(2), name(1)]);
    assert_eq!(second.body["more"], true);

    let last = page(app, &format!("?limit=2&before={}", name(1))).await;
    assert_eq!(ids(&last), vec![name(0)]);
    assert_eq!(last.body["more"], false);
    assert!(last.body["before"].is_null());

    let all = page(app, "").await;
    assert_eq!(ids(&all).len(), 5);
    assert_eq!(all.body["more"], false);
}

async fn history_refuses_what_is_not_there(app: &App) {
    thread_with_messages(app, 1).await;
    let alice = desktop_token("alice");
    let cases = [
        (
            "/threads/ses_01NOSUCHTHREAD00000000000/messages".to_owned(),
            StatusCode::NOT_FOUND,
            "not_found",
        ),
        (
            "/threads/nope/messages".to_owned(),
            StatusCode::BAD_REQUEST,
            "invalid_id",
        ),
        (
            format!("/threads/{ID}/messages?before=bad"),
            StatusCode::BAD_REQUEST,
            "invalid_id",
        ),
        (
            format!("/threads/{ID}/messages?limit=0"),
            StatusCode::BAD_REQUEST,
            "bad_request",
        ),
    ];
    for (path, status, key) in cases {
        let answer = app.call(Method::GET, &path).bearer(&alice).send().await;
        assert_eq!(answer.status, status, "{path}");
        assert_eq!(answer.key(), key, "{path}");
    }
    let unknown_before = page(app, "?before=msg_01UNKNOWN0000000000000000").await;
    assert_eq!(unknown_before.body["messages"], json!([]));
}

http_suite!(
    history_pages_newest_first,
    history_refuses_what_is_not_there
);
