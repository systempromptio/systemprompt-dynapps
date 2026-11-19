//! `GET /changes`: the full list, the opaque cursor and its round trip,
//! tombstones after a cursor, paging, and what it refuses.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::http_support::{Answer, App, desktop_token, http_suite};

fn path(n: u8) -> String {
    format!("/threads/ses_01FEED{n:020}")
}

async fn create(app: &App, n: u8) {
    let answer = app
        .call(Method::PUT, &path(n))
        .bearer(&desktop_token("alice"))
        .json(&json!({ "title": format!("thread {n}") }))
        .send()
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{:?}", answer.body);
}

async fn feed(app: &App, query: &str) -> Answer {
    let answer = app
        .call(Method::GET, &format!("/changes{query}"))
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{query}: {:?}", answer.body);
    answer
}

fn titles(answer: &Answer) -> Vec<String> {
    answer.body["threads"]
        .as_array()
        .expect("a thread list")
        .iter()
        .map(|thread| thread["title"].as_str().unwrap_or("").to_owned())
        .collect()
}

fn cursor(answer: &Answer) -> String {
    answer.body["cursor"].as_str().expect("a cursor").to_owned()
}

async fn the_full_list_has_live_threads_only(app: &App) {
    let empty = feed(app, "").await;
    assert_eq!(empty.body["threads"], json!([]));
    assert_eq!(empty.body["more"], false);
    assert!(!cursor(&empty).is_empty());

    create(app, 1).await;
    create(app, 2).await;
    let deleted = app
        .call(Method::DELETE, &path(1))
        .bearer(&desktop_token("alice"))
        .if_match("1")
        .send()
        .await;
    assert_eq!(deleted.status, StatusCode::OK);

    let full = feed(app, "").await;
    assert_eq!(titles(&full), vec!["thread 2"]);
    assert_eq!(full.body["more"], false);
}

async fn the_cursor_round_trips_and_carries_tombstones(app: &App) {
    create(app, 1).await;
    create(app, 2).await;
    let first = feed(app, "").await;
    let since = cursor(&first);

    let renamed = app
        .call(Method::PUT, &path(2))
        .bearer(&desktop_token("alice"))
        .if_match("1")
        .json(&json!({ "title": "renamed" }))
        .send()
        .await;
    assert_eq!(renamed.status, StatusCode::OK);
    let deleted = app
        .call(Method::DELETE, &path(1))
        .bearer(&desktop_token("alice"))
        .if_match("1")
        .send()
        .await;
    assert_eq!(deleted.status, StatusCode::OK);

    let changes = feed(app, &format!("?since={since}")).await;
    let rows = changes.body["threads"].as_array().expect("rows").clone();
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0]["title"], "renamed");
    assert_eq!(rows[1]["deleted"], true);
    assert_eq!(rows[1]["id"], path(1).trim_start_matches("/threads/"));
    assert_ne!(cursor(&changes), since);

    let next = cursor(&changes);
    let quiet = feed(app, &format!("?since={next}")).await;
    assert_eq!(quiet.body["threads"], json!([]));
    assert_eq!(cursor(&quiet), next, "an empty page keeps the cursor");
}

async fn the_feed_pages_and_says_more(app: &App) {
    for n in 1..=3 {
        create(app, n).await;
    }
    let first = feed(app, "?limit=2").await;
    assert_eq!(titles(&first), vec!["thread 1", "thread 2"]);
    assert_eq!(first.body["more"], true);

    let rest = feed(app, &format!("?since={}&limit=2", cursor(&first))).await;
    assert_eq!(titles(&rest), vec!["thread 3"]);
    assert_eq!(rest.body["more"], false);
}

async fn a_bad_cursor_or_limit_is_refused(app: &App) {
    for (query, key) in [
        ("?since=not-a-cursor", "invalid_cursor"),
        ("?since=AAAA", "invalid_cursor"),
        ("?limit=0", "bad_request"),
        ("?limit=many", "bad_request"),
        ("?limit=-3", "bad_request"),
    ] {
        let answer = app
            .call(Method::GET, &format!("/changes{query}"))
            .bearer(&desktop_token("alice"))
            .send()
            .await;
        assert_eq!(answer.status, StatusCode::BAD_REQUEST, "{query}");
        assert_eq!(answer.key(), key, "{query}");
        assert!(answer.body["error"]["message"].is_string());
        assert_eq!(answer.body.get("threads"), None::<&Value>);
    }
}

http_suite!(
    the_full_list_has_live_threads_only,
    the_cursor_round_trips_and_carries_tombstones,
    the_feed_pages_and_says_more,
    a_bad_cursor_or_limit_is_refused,
);
