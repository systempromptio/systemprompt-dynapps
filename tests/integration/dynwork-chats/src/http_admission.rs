//! A model the instance's rules do not allow the caller: `403
//! model_not_allowed` from a prompt and from a thread write, and nothing is
//! written, started or announced.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::http_support::{Answer, App, desktop_token, http_suite};
use crate::models_support::DENIED_ROLE;

const THREAD: &str = "ses_01ADMIT00000000000000001";
const OTHER: &str = "ses_01ADMIT00000000000000002";
const ASK: &str = "msg_01ADMIT00000000000000001";
const ASK_AGAIN: &str = "msg_01ADMIT00000000000000002";

fn opus() -> Value {
    json!({ "providerID": "anthropic", "id": "claude-opus" })
}

fn sonnet() -> Value {
    json!({ "providerID": "anthropic", "id": "claude-sonnet" })
}

async fn prompt_with(app: &App, user: &str, message: &str, model: Option<Value>) -> Answer {
    let mut body = json!({ "messageID": message, "parts": [{ "type": "text", "text": "hi" }] });
    if let Some(model) = model {
        body["model"] = model;
    }
    app.call(Method::POST, &format!("/threads/{THREAD}/prompt"))
        .bearer(&desktop_token(user))
        .json(&body)
        .send()
        .await
}

async fn put_thread(app: &App, user: &str, id: &str, revision: Option<&str>, body: Value) -> Answer {
    let mut call = app
        .call(Method::PUT, &format!("/threads/{id}"))
        .bearer(&desktop_token(user))
        .json(&body);
    if let Some(revision) = revision {
        call = call.if_match(revision);
    }
    call.send().await
}

async fn an_admitted_model_answers_202(app: &App) {
    app.source.set_roles("alice", &["staff"]);
    app.source.set_roles("bob", &[DENIED_ROLE]);
    app.create_thread("alice", THREAD).await;
    app.scripted(&["fine"]);
    let named = prompt_with(app, "alice", ASK, Some(opus())).await;
    assert_eq!(named.status, StatusCode::ACCEPTED, "{:?}", named.body);
    app.settled(THREAD).await;
    app.create_thread("bob", OTHER).await;
    app.scripted(&["fine"]);
    let own = app
        .call(Method::POST, &format!("/threads/{OTHER}/prompt"))
        .bearer(&desktop_token("bob"))
        .json(&json!({ "messageID": ASK, "parts": [{ "type": "text", "text": "hi" }] }))
        .send()
        .await;
    assert_eq!(own.status, StatusCode::ACCEPTED, "{:?}", own.body);
}

async fn a_model_the_rules_hide_is_403_and_nothing_starts(app: &App) {
    app.source.set_roles("bob", &[DENIED_ROLE]);
    app.create_thread("bob", THREAD).await;
    app.hints.clear();
    let refused = prompt_with(app, "bob", ASK, Some(opus())).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{:?}", refused.body);
    assert_eq!(refused.key(), "model_not_allowed");
    assert!(app.model.requests().is_empty());
    assert!(app.messages("bob", THREAD).await.is_empty());
    assert!(app.hints.sent().is_empty(), "{:?}", app.hints.sent());
    app.scripted(&["fine"]);
    let allowed = prompt_with(app, "bob", ASK, Some(sonnet())).await;
    assert_eq!(allowed.status, StatusCode::ACCEPTED, "{:?}", allowed.body);
}

async fn a_model_the_instance_does_not_have_is_403(app: &App) {
    app.create_thread("alice", THREAD).await;
    let unknown = json!({ "providerID": "anthropic", "id": "claude-nope" });
    let refused = prompt_with(app, "alice", ASK, Some(unknown)).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{:?}", refused.body);
    let wrong_provider = json!({ "providerID": "deepseek", "id": "claude-sonnet" });
    let refused = prompt_with(app, "alice", ASK_AGAIN, Some(wrong_provider)).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
}

async fn the_threads_own_model_is_judged_when_the_prompt_names_none(app: &App) {
    app.source.set_roles("alice", &["staff"]);
    let created = put_thread(app, "alice", THREAD, None, json!({ "model": opus() })).await;
    assert_eq!(created.status, StatusCode::OK, "{:?}", created.body);
    app.source.set_roles("alice", &[DENIED_ROLE]);
    let refused = prompt_with(app, "alice", ASK, None).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{:?}", refused.body);
    assert_eq!(refused.key(), "model_not_allowed");
    assert!(app.model.requests().is_empty());
    app.scripted(&["fine"]);
    let overridden = prompt_with(app, "alice", ASK_AGAIN, Some(sonnet())).await;
    assert_eq!(overridden.status, StatusCode::ACCEPTED, "{:?}", overridden.body);
}

async fn a_thread_write_naming_a_hidden_model_is_403_and_changes_nothing(app: &App) {
    app.source.set_roles("bob", &[DENIED_ROLE]);
    let refused = put_thread(app, "bob", THREAD, None, json!({ "title": "t", "model": opus() })).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{:?}", refused.body);
    assert_eq!(refused.key(), "model_not_allowed");
    let created = put_thread(app, "bob", THREAD, None, json!({ "title": "t" })).await;
    assert_eq!(created.status, StatusCode::OK, "no row was left behind: {:?}", created.body);
    let revision = created.body["revision"].as_i64().expect("a revision");
    app.hints.clear();
    let edit = put_thread(app, "bob", THREAD, Some(&revision.to_string()), json!({ "model": opus() })).await;
    assert_eq!(edit.status, StatusCode::FORBIDDEN, "{:?}", edit.body);
    assert!(app.hints.sent().is_empty());
    let moved = put_thread(app, "bob", THREAD, Some(&revision.to_string()), json!({ "model": sonnet() })).await;
    assert_eq!(moved.status, StatusCode::OK, "{:?}", moved.body);
    assert_eq!(moved.body["revision"].as_i64(), Some(revision + 1));
    assert_eq!(moved.body["model"], sonnet());
}

async fn another_users_thread_is_404_before_the_model_is_judged(app: &App) {
    app.source.set_roles("bob", &[DENIED_ROLE]);
    app.create_thread("alice", THREAD).await;
    let foreign = prompt_with(app, "bob", ASK, Some(opus())).await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND, "{:?}", foreign.body);
}

async fn without_a_model_service_a_model_is_never_admitted(app: &App) {
    app.create_thread("alice", THREAD).await;
    app.source.switch_off();
    let refused = put_thread(app, "alice", OTHER, None, json!({ "model": sonnet() })).await;
    assert_eq!(refused.status, StatusCode::SERVICE_UNAVAILABLE, "{:?}", refused.body);
    assert_eq!(refused.key(), "ai_unavailable");
}

http_suite!(
    an_admitted_model_answers_202,
    a_model_the_rules_hide_is_403_and_nothing_starts,
    a_model_the_instance_does_not_have_is_403,
    the_threads_own_model_is_judged_when_the_prompt_names_none,
    a_thread_write_naming_a_hidden_model_is_403_and_changes_nothing,
    another_users_thread_is_404_before_the_model_is_judged,
    without_a_model_service_a_model_is_never_admitted,
);
