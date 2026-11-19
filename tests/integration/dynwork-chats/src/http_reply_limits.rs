//! The limits on a person's replies: at most three running at once (a fourth
//! prompt is `429 too_many_replies` and claims nothing), and a model that goes
//! quiet ends its reply as failed and frees the thread.

use axum::http::StatusCode;
use serde_json::json;

use crate::http_support::{App, http_suite};
use crate::reply_support::kinds;

const THREADS: [&str; 4] = [
    "ses_01LIMITS0000000000000000001",
    "ses_01LIMITS0000000000000000002",
    "ses_01LIMITS0000000000000000003",
    "ses_01LIMITS0000000000000000004",
];
const BOBS: &str = "ses_01LIMITSBOB00000000000001";
const ASK: &str = "msg_01LIMITSASK00000000000001";
const ASK_AGAIN: &str = "msg_01LIMITSASK00000000000002";

async fn a_fourth_running_reply_of_one_person_is_429(app: &App) {
    for thread in THREADS {
        app.create_thread("alice", thread).await;
    }
    let mut scripts = Vec::new();
    for (count, thread) in THREADS[..3].iter().enumerate() {
        let script = app.script();
        let answer = app.prompt("alice", thread, ASK, "keep going").await;
        assert_eq!(answer.status, StatusCode::ACCEPTED, "{:?}", answer.body);
        app.asked(count + 1).await;
        script.text("working ");
        scripts.push(script);
    }
    let refused = app.prompt("alice", THREADS[3], ASK, "one more").await;
    assert_eq!(
        refused.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{:?}",
        refused.body
    );
    assert_eq!(refused.key(), "too_many_replies");
    assert!(
        app.messages("alice", THREADS[3]).await.is_empty(),
        "a refused prompt writes nothing"
    );

    app.create_thread("bob", BOBS).await;
    app.scripted(&["bob's own"]);
    let bobs = app.prompt("bob", BOBS, ASK, "mine").await;
    assert_eq!(
        bobs.status,
        StatusCode::ACCEPTED,
        "another person's replies do not count"
    );
    app.settled(BOBS).await;

    drop(scripts.remove(0));
    app.settled(THREADS[0]).await;
    app.scripted(&["now it fits"]);
    let fits = app.prompt("alice", THREADS[3], ASK_AGAIN, "one more").await;
    assert_eq!(fits.status, StatusCode::ACCEPTED, "{:?}", fits.body);
    app.settled(THREADS[3]).await;
    drop(scripts);
    for thread in &THREADS[1..3] {
        app.settled(thread).await;
    }
}

async fn a_model_that_stalls_fails_the_reply_and_frees_the_thread(app: &App) {
    let thread = THREADS[0];
    app.create_thread("alice", thread).await;
    let stalled = app.script();
    let answer = app.prompt("alice", thread, ASK, "hello?").await;
    assert_eq!(answer.status, StatusCode::ACCEPTED);
    stalled.text("Partial ");
    let mut log = app.log("alice", thread, "?follow=1").await;
    let events = log.until_terminal().await;
    assert_eq!(
        kinds(&events).last().copied(),
        Some("session.execution.failed")
    );
    let failed = &events[events.len() - 1].data["data"]["error"];
    assert_eq!(failed["type"], "provider.timeout");
    app.settled(thread).await;
    let messages = app.messages("alice", thread).await;
    let answered = &messages[0];
    assert_eq!(answered["finish"], "error");
    assert_eq!(answered["error"]["type"], "provider.timeout");
    assert_eq!(
        answered["content"],
        json!([{ "type": "text", "text": "Partial " }])
    );
    app.scripted(&["back"]);
    let next = app.prompt("alice", thread, ASK_AGAIN, "again").await;
    assert_eq!(next.status, StatusCode::ACCEPTED, "{:?}", next.body);
    app.settled(thread).await;
    drop(stalled);
}

http_suite!(
    a_fourth_running_reply_of_one_person_is_429,
    a_model_that_stalls_fails_the_reply_and_frees_the_thread,
);
