//! The change hint: what each kind of change sends and to whom, what a
//! refused write sends (nothing), and the relay's `NOTIFY` it rides on.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use dynapps_test_common::db_or_skip;
use dynwork_chats::hint::{Hints, KIND, RelayHints};
use dynwork_relay::hint::CHANNEL;
use dynwork_relay::ids::PeerId;
use dynwork_relay::ws::PEER_HEADER;
use serde_json::{Value, json};
use sqlx::postgres::{PgListener, PgPoolOptions};
use systemprompt::identifiers::UserId;

use crate::http_support::{App, desktop_token, http_suite, user};

const A: &str = "ses_01HINTA00000000000000000";
const ASK: &str = "msg_01HINTPROMPT0000000000001";
const ASK_AGAIN: &str = "msg_01HINTPROMPT0000000000002";
const PEER: &str = "0123456789abcdef0123456789abcdef";

pub(crate) type Sent = (String, Option<PeerId>);

#[derive(Debug, Default)]
pub(crate) struct RecordingHints(Mutex<Vec<Sent>>);

impl RecordingHints {
    pub(crate) fn sent(&self) -> Vec<Sent> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn clear(&self) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }

    pub(crate) fn users(&self) -> Vec<String> {
        self.sent().into_iter().map(|(user, _)| user).collect()
    }
}

#[async_trait]
impl Hints for RecordingHints {
    async fn changed(&self, user: &UserId, except: Option<PeerId>) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((user.as_str().to_owned(), except));
    }
}

fn peer() -> PeerId {
    PeerId::parse_hex(PEER).expect("a peer id")
}

fn path(id: &str) -> String {
    format!("/threads/{id}")
}

async fn put(app: &App, who: &str, id: &str, revision: Option<&str>, body: &Value) -> StatusCode {
    let mut call = app
        .call(Method::PUT, &path(id))
        .bearer(&desktop_token(who))
        .json(body);
    if let Some(revision) = revision {
        call = call.if_match(revision);
    }
    call.send().await.status
}

async fn creating_a_thread_hints_its_owner_except_the_caller_desktop(app: &App) {
    let created = app
        .call(Method::PUT, &path(A))
        .bearer(&desktop_token("alice"))
        .header(PEER_HEADER, PEER)
        .json(&json!({ "title": "Plan" }))
        .send()
        .await;
    assert_eq!(created.status, StatusCode::OK, "{:?}", created.body);
    assert_eq!(app.hints.sent(), vec![("alice".to_owned(), Some(peer()))]);

    let other = put(app, "bob", "ses_01HINTB00000000000000000", None, &json!({})).await;
    assert_eq!(other, StatusCode::OK);
    assert_eq!(
        app.hints.sent().last(),
        Some(&("bob".to_owned(), None)),
        "no peer header means every desktop gets it"
    );
    assert_eq!(app.hints.sent().len(), 2);
}

async fn a_malformed_peer_header_hints_every_desktop(app: &App) {
    let created = app
        .call(Method::PUT, &path(A))
        .bearer(&desktop_token("alice"))
        .header(PEER_HEADER, "not-a-peer")
        .json(&json!({}))
        .send()
        .await;
    assert_eq!(created.status, StatusCode::OK, "{:?}", created.body);
    assert_eq!(app.hints.sent(), vec![("alice".to_owned(), None)]);
}

async fn every_kind_of_edit_hints_exactly_once(app: &App) {
    assert_eq!(put(app, "alice", A, None, &json!({})).await, StatusCode::OK);
    let steps = [
        ("1", json!({ "title": "Renamed" })),
        ("2", json!({ "archived": true })),
        ("3", json!({ "archived": false })),
        (
            "4",
            json!({ "model": { "providerID": "anthropic", "id": "claude-sonnet" } }),
        ),
    ];
    for (count, (revision, body)) in steps.iter().enumerate() {
        assert_eq!(
            put(app, "alice", A, Some(revision), body).await,
            StatusCode::OK
        );
        assert_eq!(app.hints.sent().len(), count + 2, "after {body}");
    }
    let deleted = app
        .call(Method::DELETE, &path(A))
        .bearer(&desktop_token("alice"))
        .if_match("5")
        .send()
        .await;
    assert_eq!(deleted.status, StatusCode::OK, "{:?}", deleted.body);
    assert_eq!(app.hints.sent().len(), 6);
    assert_eq!(app.hints.users(), vec!["alice"; 6]);
}

async fn a_write_that_changes_nothing_hints_nobody(app: &App) {
    assert_eq!(put(app, "alice", A, None, &json!({})).await, StatusCode::OK);
    app.hints.clear();

    let stale = put(app, "alice", A, Some("9"), &json!({ "title": "x" })).await;
    assert_eq!(stale, StatusCode::CONFLICT);
    let again = put(app, "alice", A, None, &json!({ "title": "x" })).await;
    assert_eq!(again, StatusCode::CONFLICT);
    let taken = put(app, "bob", A, None, &json!({})).await;
    assert_eq!(taken, StatusCode::CONFLICT);
    let foreign = put(app, "bob", A, Some("1"), &json!({ "title": "mine" })).await;
    assert_eq!(foreign, StatusCode::NOT_FOUND);
    let empty = put(app, "alice", A, Some("1"), &json!({})).await;
    assert_eq!(empty, StatusCode::OK);
    let archived = put(
        app,
        "alice",
        "ses_01HINTC00000000000000000",
        None,
        &json!({ "archived": true }),
    )
    .await;
    assert_eq!(archived, StatusCode::BAD_REQUEST);
    let bare = app
        .call(Method::DELETE, &path(A))
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    assert_eq!(bare.status, StatusCode::PRECONDITION_REQUIRED);
    let foreign_delete = app
        .call(Method::DELETE, &path(A))
        .bearer(&desktop_token("bob"))
        .if_match("1")
        .send()
        .await;
    assert_eq!(foreign_delete.status, StatusCode::NOT_FOUND);
    let anonymous = app
        .call(Method::PUT, &path(A))
        .json(&json!({}))
        .send()
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);

    assert!(app.hints.sent().is_empty(), "{:?}", app.hints.sent());
}

async fn a_reply_hints_when_it_starts_and_when_it_ends(app: &App) {
    app.create_thread("alice", A).await;
    assert_eq!(app.hints.sent().len(), 1, "the create");
    let script = app.script();
    let started = app
        .call(Method::POST, &format!("{}/prompt", path(A)))
        .bearer(&desktop_token("alice"))
        .header(PEER_HEADER, PEER)
        .json(&json!({ "messageID": ASK, "parts": [{ "type": "text", "text": "hi" }] }))
        .send()
        .await;
    assert_eq!(started.status, StatusCode::ACCEPTED, "{:?}", started.body);
    assert_eq!(
        app.hints.sent().last(),
        Some(&("alice".to_owned(), Some(peer())))
    );
    assert_eq!(app.hints.sent().len(), 2, "the start");

    let retried = app.prompt("alice", A, ASK, "hi").await;
    assert_eq!(retried.status, StatusCode::ACCEPTED);
    let busy = app.prompt("alice", A, ASK_AGAIN, "more").await;
    assert_eq!(busy.status, StatusCode::CONFLICT);
    assert_eq!(
        app.hints.sent().len(),
        2,
        "a retry and a busy prompt send none"
    );

    script.text("hello");
    drop(script);
    app.settled(A).await;
    assert_eq!(
        app.hints.sent().last(),
        Some(&("alice".to_owned(), None)),
        "the end goes to every desktop"
    );
    assert_eq!(app.hints.sent().len(), 3, "the end");
}

async fn an_interrupted_reply_hints_once_at_its_end(app: &App) {
    app.create_thread("alice", A).await;
    let _held = app.script();
    let started = app.prompt("alice", A, ASK, "hi").await;
    assert_eq!(started.status, StatusCode::ACCEPTED, "{:?}", started.body);
    let stopped = app.interrupt("alice", A).await;
    assert_eq!(stopped.status, StatusCode::NO_CONTENT);
    assert_eq!(app.hints.sent().len(), 3);
}

async fn a_refused_prompt_hints_nobody(app: &App) {
    app.create_thread("alice", A).await;
    app.hints.clear();
    let missing = app
        .prompt("alice", "ses_01HINTNONE000000000000000", ASK, "hi")
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let foreign = app.prompt("bob", A, ASK, "hi").await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    app.source.switch_off();
    let off = app.prompt("alice", A, ASK, "hi").await;
    assert_eq!(off.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(app.hints.sent().is_empty(), "{:?}", app.hints.sent());
}

http_suite!(
    creating_a_thread_hints_its_owner_except_the_caller_desktop,
    a_malformed_peer_header_hints_every_desktop,
    every_kind_of_edit_hints_exactly_once,
    a_write_that_changes_nothing_hints_nobody,
    a_reply_hints_when_it_starts_and_when_it_ends,
    an_interrupted_reply_hints_once_at_its_end,
    a_refused_prompt_hints_nobody,
);

#[tokio::test]
async fn the_relay_hint_reaches_the_relays_channel() {
    let db = db_or_skip!();
    let mut listener = PgListener::connect_with(&db.pool)
        .await
        .expect("listen for hints");
    listener.listen(CHANNEL).await.expect("listen");
    let hints = RelayHints::new(Arc::clone(&db.pool));
    hints.changed(&user("alice"), Some(peer())).await;
    hints.changed(&user("bob"), None).await;
    let mut seen = Vec::new();
    for _ in 0..2 {
        let note = tokio::time::timeout(Duration::from_secs(5), listener.recv())
            .await
            .expect("a hint arrives")
            .expect("the listener is open");
        seen.push(serde_json::from_str::<Value>(note.payload()).expect("a JSON payload"));
    }
    drop(listener);
    db.cleanup().await;
    assert_eq!(seen[0]["user"], "alice");
    assert_eq!(seen[0]["except"], PEER);
    assert_eq!(seen[0]["kind"], KIND);
    assert_eq!(seen[1]["user"], "bob");
    assert!(seen[1]["except"].is_null());
}

#[tokio::test]
async fn a_failed_publish_is_logged_and_ignored() {
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(300))
        .connect_lazy("postgres://nobody:nothing@127.0.0.1:1/none")
        .expect("a lazy pool");
    let hints = RelayHints::new(Arc::new(pool));
    tokio::time::timeout(Duration::from_secs(5), hints.changed(&user("alice"), None))
        .await
        .expect("a failed publish returns");
}
