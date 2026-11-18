//! `PUT` and `DELETE /threads/{id}`: create, the `If-Match` revisions,
//! archive and model changes, tombstones, isolation between users, and what
//! is refused as a bad request.

use axum::http::{Method, StatusCode};
use chrono::DateTime;
use serde_json::{Value, json};

use crate::fixture::at;
use crate::http_support::{Answer, App, desktop_token, http_suite};

const A: &str = "/threads/ses_01THREADA00000000000000000";
const B: &str = "/threads/ses_01THREADB00000000000000000";

async fn create(app: &App, path: &str, token: &str, body: &Value) -> Answer {
    let answer = app
        .call(Method::PUT, path)
        .bearer(token)
        .json(body)
        .send()
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{:?}", answer.body);
    answer
}

async fn edit(app: &App, path: &str, revision: &str, body: &Value) -> Answer {
    app.call(Method::PUT, path)
        .bearer(&desktop_token("alice"))
        .if_match(revision)
        .json(body)
        .send()
        .await
}

fn millis(rfc3339: &Value) -> i64 {
    DateTime::parse_from_rfc3339(rfc3339.as_str().expect("an RFC 3339 time"))
        .expect("parse the time")
        .timestamp_millis()
}

async fn creating_answers_revision_one_in_the_wire_shape(app: &App) {
    let model = json!({ "providerID": "anthropic", "id": "claude-sonnet" });
    let answer = create(
        app,
        A,
        &desktop_token("alice"),
        &json!({ "title": "Plan", "model": model }),
    )
    .await;
    let thread = &answer.body;
    assert_eq!(thread["id"], "ses_01THREADA00000000000000000");
    assert_eq!(thread["title"], "Plan");
    assert_eq!(thread["model"], model);
    assert_eq!(thread["revision"], 1);
    assert_eq!(thread["deleted"], false);
    assert!(thread["time"]["archived"].is_null());
    assert!(thread["archived_at"].is_null() && thread["deleted_at"].is_null());
    assert_eq!(
        thread["time"]["created"].as_i64(),
        Some(millis(&thread["created_at"]))
    );
    assert_eq!(
        thread["time"]["updated"].as_i64(),
        Some(millis(&thread["updated_at"]))
    );
    assert!(thread["time"]["created"].as_i64() >= Some(at(0).timestamp_millis()));
    assert_eq!(
        answer
            .headers
            .get("etag")
            .map(axum::http::HeaderValue::as_bytes),
        Some(&b"\"1\""[..])
    );

    let untitled = create(app, B, &desktop_token("alice"), &json!({})).await;
    assert_eq!(untitled.body["title"], "");
    assert!(untitled.body["model"].is_null());
}

async fn creating_an_existing_id_is_a_conflict(app: &App) {
    let alice = desktop_token("alice");
    create(app, A, &alice, &json!({ "title": "first" })).await;

    let again = app
        .call(Method::PUT, A)
        .bearer(&alice)
        .json(&json!({ "title": "second" }))
        .send()
        .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(again.key(), "revision_conflict");
    assert_eq!(again.body["current"]["title"], "first");

    let bobs = app
        .call(Method::PUT, A)
        .bearer(&desktop_token("bob"))
        .json(&json!({ "title": "mine now" }))
        .send()
        .await;
    assert_eq!(bobs.status, StatusCode::CONFLICT);
    assert_eq!(bobs.key(), "id_taken");
    assert!(bobs.body.get("current").is_none(), "{:?}", bobs.body);
}

async fn edits_need_the_current_revision(app: &App) {
    create(app, A, &desktop_token("alice"), &json!({ "title": "one" })).await;

    let renamed = edit(app, A, "1", &json!({ "title": "two" })).await;
    assert_eq!(renamed.status, StatusCode::OK);
    assert_eq!(renamed.body["revision"], 2);
    assert_eq!(renamed.body["title"], "two");

    let stale = edit(app, A, "1", &json!({ "title": "lost" })).await;
    assert_eq!(stale.status, StatusCode::CONFLICT);
    assert_eq!(stale.key(), "revision_conflict");
    assert_eq!(stale.body["current"]["revision"], 2);
    assert_eq!(stale.body["current"]["title"], "two");

    let quoted = edit(app, A, "\"2\"", &json!({ "title": "three" })).await;
    assert_eq!(quoted.body["revision"], 3);
    let weak = edit(app, A, "W/\"3\"", &json!({ "title": "four" })).await;
    assert_eq!(weak.body["revision"], 4);

    for bad in ["abc", "0", "-1", "\"4"] {
        let refused = edit(app, A, bad, &json!({ "title": "x" })).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(refused.key(), "bad_request", "{bad}");
    }

    let untouched = edit(app, A, "4", &json!({})).await;
    assert_eq!(untouched.status, StatusCode::OK);
    assert_eq!(untouched.body["revision"], 4);
}

async fn archiving_and_the_model_are_edits(app: &App) {
    create(app, A, &desktop_token("alice"), &json!({ "title": "t" })).await;

    let archived = edit(app, A, "1", &json!({ "archived": true })).await;
    assert_eq!(archived.body["revision"], 2);
    let at_ms = archived.body["time"]["archived"]
        .as_i64()
        .expect("archived");
    assert_eq!(at_ms, millis(&archived.body["archived_at"]));

    let back = edit(app, A, "2", &json!({ "archived": false })).await;
    assert!(back.body["time"].get("archived").is_none());
    assert!(back.body["archived_at"].is_null());

    let model = json!({ "providerID": "deepseek", "id": "deepseek-chat" });
    let moved = edit(app, A, "3", &json!({ "model": model })).await;
    assert_eq!(moved.body["model"], model);
    assert_eq!(moved.body["title"], "t");
    assert_eq!(moved.body["revision"], 4);
}

async fn deleting_needs_if_match_and_leaves_a_tombstone(app: &App) {
    let alice = desktop_token("alice");
    create(app, A, &alice, &json!({ "title": "doomed" })).await;

    let bare = app.call(Method::DELETE, A).bearer(&alice).send().await;
    assert_eq!(bare.status, StatusCode::PRECONDITION_REQUIRED);
    assert_eq!(bare.key(), "precondition_required");

    let stale = app
        .call(Method::DELETE, A)
        .bearer(&alice)
        .if_match("7")
        .send()
        .await;
    assert_eq!(stale.status, StatusCode::CONFLICT);
    assert_eq!(stale.body["current"]["revision"], 1);

    let gone = app
        .call(Method::DELETE, A)
        .bearer(&alice)
        .if_match("1")
        .send()
        .await;
    assert_eq!(gone.status, StatusCode::OK);
    assert_eq!(gone.body["deleted"], true);
    assert_eq!(gone.body["revision"], 2);
    assert!(gone.body["deleted_at"].is_string());

    let rename = edit(app, A, "2", &json!({ "title": "back?" })).await;
    assert_eq!(rename.status, StatusCode::CONFLICT);
    assert_eq!(rename.body["current"]["deleted"], true);

    let twice = app
        .call(Method::DELETE, A)
        .bearer(&alice)
        .if_match("2")
        .send()
        .await;
    assert_eq!(twice.status, StatusCode::CONFLICT);

    let history = app
        .call(Method::GET, &format!("{A}/messages"))
        .bearer(&alice)
        .send()
        .await;
    assert_eq!(history.status, StatusCode::NOT_FOUND);
}

async fn users_never_reach_each_others_threads(app: &App) {
    create(
        app,
        A,
        &desktop_token("alice"),
        &json!({ "title": "private" }),
    )
    .await;
    let bob = desktop_token("bob");

    let rename = app
        .call(Method::PUT, A)
        .bearer(&bob)
        .if_match("1")
        .json(&json!({ "title": "hijacked" }))
        .send()
        .await;
    assert_eq!(rename.status, StatusCode::NOT_FOUND);
    assert_eq!(rename.key(), "not_found");
    assert!(rename.body.get("current").is_none());

    let delete = app
        .call(Method::DELETE, A)
        .bearer(&bob)
        .if_match("1")
        .send()
        .await;
    assert_eq!(delete.status, StatusCode::NOT_FOUND);

    let history = app
        .call(Method::GET, &format!("{A}/messages"))
        .bearer(&bob)
        .send()
        .await;
    assert_eq!(history.status, StatusCode::NOT_FOUND);

    let feed = app.call(Method::GET, "/changes").bearer(&bob).send().await;
    assert_eq!(feed.body["threads"], json!([]));

    let alices = app
        .call(Method::GET, "/changes")
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    assert_eq!(alices.body["threads"][0]["title"], "private");
    assert_eq!(alices.body["threads"][0]["revision"], 1);
}

async fn malformed_ids_and_bodies_are_bad_requests(app: &App) {
    let alice = desktop_token("alice");
    let id = app
        .call(Method::PUT, "/threads/not-a-thread")
        .bearer(&alice)
        .json(&json!({}))
        .send()
        .await;
    assert_eq!(id.status, StatusCode::BAD_REQUEST);
    assert_eq!(id.key(), "invalid_id");

    let long_title = "x".repeat(513);
    for body in [
        json!({ "title": 5 }),
        json!({ "title": long_title }),
        json!({ "model": { "providerID": "", "id": "m" } }),
        json!({ "model": { "providerID": "p" } }),
        json!({ "archived": true }),
        json!([1, 2]),
    ] {
        let answer = app
            .call(Method::PUT, A)
            .bearer(&alice)
            .json(&body)
            .send()
            .await;
        assert_eq!(answer.status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(answer.key(), "bad_request", "{body}");
    }

    let huge = format!("{{\"title\":\"{}\"}}", "y".repeat(70 * 1024));
    let too_large = app
        .call(Method::PUT, A)
        .bearer(&alice)
        .raw(&huge)
        .send()
        .await;
    assert_eq!(too_large.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(too_large.key(), "too_large");

    let feed = app
        .call(Method::GET, "/changes")
        .bearer(&alice)
        .send()
        .await;
    assert_eq!(feed.body["threads"], json!([]), "nothing was created");
}

http_suite!(
    creating_answers_revision_one_in_the_wire_shape,
    creating_an_existing_id_is_a_conflict,
    edits_need_the_current_revision,
    archiving_and_the_model_are_edits,
    deleting_needs_if_match_and_leaves_a_tombstone,
    users_never_reach_each_others_threads,
    malformed_ids_and_bodies_are_bad_requests,
);
