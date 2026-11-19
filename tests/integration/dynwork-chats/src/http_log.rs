//! `GET /threads/{id}/log`: server-sent `OpenCode` v2 events, replayed from
//! the store and then followed live, resumed by `after` or `Last-Event-ID`
//! with no gap and no duplicate, and a follower going away never stops the
//! reply.

use std::time::Duration;

use axum::http::StatusCode;
use serde_json::{Value, json};
use tokio::time::Instant;

use crate::http_support::{
    App, HEARTBEAT, desktop_token_expiring, http_suite, mobile_token_expiring,
};
use crate::reply_support::{LogEvent, WAIT, deltas, kinds};

// What a test allows the server beyond the exact moment, for a busy machine.
const SLACK: Duration = Duration::from_secs(1);

const THREAD: &str = "ses_01LOGSTREAM00000000000001";
const ASK: &str = "msg_01LOGPROMPT00000000000001";

const LIFECYCLE: [&str; 11] = [
    "session.created",
    "session.execution.started",
    "session.step.started",
    "session.text.started",
    "session.text.delta",
    "session.text.delta",
    "session.text.delta",
    "session.text.ended",
    "session.step.ended",
    "session.usage.updated",
    "session.execution.succeeded",
];

fn ids(events: &[LogEvent]) -> Vec<i64> {
    events
        .iter()
        .map(|event| event.id.expect("every event has an id"))
        .collect()
}

fn strictly_increasing(ids: &[i64]) -> bool {
    ids.windows(2).all(|pair| pair[0] < pair[1])
}

async fn next_delta(log: &mut crate::reply_support::Log) -> String {
    loop {
        let event = log.next(WAIT).await.expect("a delta");
        if event.kind() == "session.text.delta" {
            return event.data["data"]["delta"]
                .as_str()
                .unwrap_or("")
                .to_owned();
        }
    }
}

async fn the_log_replays_then_follows(app: &App) {
    app.create_thread("alice", THREAD).await;
    let script = app.script();
    assert_eq!(
        app.prompt("alice", THREAD, ASK, "count").await.status,
        StatusCode::ACCEPTED
    );
    let mut log = app.log("alice", THREAD, "?follow=1").await;
    assert_eq!(log.status, StatusCode::OK);
    assert!(
        log.content_type.starts_with("text/event-stream"),
        "{}",
        log.content_type
    );
    for text in ["one ", "two ", "three"] {
        script.text(text);
        assert_eq!(next_delta(&mut log).await, text);
    }
    script.usage(10, 3);
    drop(script);
    let rest = log.until_terminal().await;
    assert_eq!(
        rest.last().map(LogEvent::kind),
        Some("session.execution.succeeded")
    );
    let all = app
        .log("alice", THREAD, "?follow=0")
        .await
        .until_closed()
        .await;
    assert_eq!(kinds(&all), LIFECYCLE);
    assert!(strictly_increasing(&ids(&all)), "{:?}", ids(&all));
    assert_eq!(deltas(&all), "one two three");
}

async fn a_reader_joining_mid_reply_gets_the_earlier_deltas_then_the_live_ones(app: &App) {
    app.create_thread("alice", THREAD).await;
    let script = app.script();
    assert_eq!(
        app.prompt("alice", THREAD, ASK, "go").await.status,
        StatusCode::ACCEPTED
    );
    let mut first = app.log("alice", THREAD, "?follow=1").await;
    script.text("early ");
    assert_eq!(next_delta(&mut first).await, "early ");
    let mut late = app.log("alice", THREAD, "?follow=1").await;
    script.text("late");
    script.usage(1, 2);
    drop(script);
    let seen_late = late.until_terminal().await;
    assert_eq!(deltas(&seen_late), "early late");
    assert_eq!(
        seen_late.first().map(LogEvent::kind),
        Some("session.created")
    );
    assert!(strictly_increasing(&ids(&seen_late)));
    let seen_first = first.until_terminal().await;
    assert_eq!(deltas(&seen_first), "late");
}

async fn after_and_last_event_id_resume_without_gap_or_duplicate(app: &App) {
    app.create_thread("alice", THREAD).await;
    app.scripted(&["a", "b", "c"]);
    assert_eq!(
        app.prompt("alice", THREAD, ASK, "abc").await.status,
        StatusCode::ACCEPTED
    );
    app.settled(THREAD).await;
    let all = ids(&app
        .log("alice", THREAD, "?follow=0")
        .await
        .until_closed()
        .await);
    let cut = all[4];
    let rest: Vec<i64> = all.iter().copied().filter(|id| *id > cut).collect();
    let after = app
        .log("alice", THREAD, &format!("?after={cut}"))
        .await
        .until_closed()
        .await;
    assert_eq!(ids(&after), rest);
    let header = app
        .log_with("alice", THREAD, "?follow=0", Some(&cut.to_string()))
        .await
        .until_closed()
        .await;
    assert_eq!(ids(&header), rest);
    let both = app
        .log_with("alice", THREAD, "?after=1", Some(&cut.to_string()))
        .await
        .until_closed()
        .await;
    assert_eq!(ids(&both), rest);
    let past_the_end = app
        .log("alice", THREAD, &format!("?after={}", all[all.len() - 1]))
        .await
        .until_closed()
        .await;
    assert!(past_the_end.is_empty());
}

async fn events_carry_the_desktop_envelope(app: &App) {
    app.create_thread("alice", THREAD).await;
    app.scripted(&["Hi"]);
    let answer = app.prompt("alice", THREAD, ASK, "hi").await;
    let reply = answer.body["assistantMessageID"].clone();
    app.settled(THREAD).await;
    let all = app
        .log("alice", THREAD, "?follow=0")
        .await
        .until_closed()
        .await;
    for event in &all {
        let data = &event.data;
        let kind = event.kind();
        let seq = event.id.expect("an id");
        assert!(
            data["id"].as_str().is_some_and(|id| id.starts_with("evt_")),
            "{data}"
        );
        assert!(data["created"].is_i64(), "{data}");
        assert_eq!(data["data"]["sessionID"], THREAD, "{data}");
        let ephemeral = matches!(kind, "session.text.delta" | "session.usage.updated");
        if ephemeral {
            assert!(data.get("durable").is_none(), "{data}");
        } else {
            assert_eq!(
                data["durable"],
                json!({ "aggregateID": THREAD, "seq": seq, "version": 1 }),
                "{data}"
            );
        }
        let located = kind == "session.created"
            || kind.starts_with("session.step.")
            || kind.starts_with("session.text.");
        assert_eq!(
            data.get("location"),
            located.then(|| json!({ "directory": "" })).as_ref(),
            "{data}"
        );
        if kind.starts_with("session.text.") || kind.starts_with("session.step.") {
            assert_eq!(data["data"]["assistantMessageID"], reply, "{data}");
        }
    }
    let created = &all[0].data["data"];
    assert_eq!(created["subpath"], "");
    assert_eq!(created["title"], "Cloud chat");
    assert_eq!(
        created["model"],
        json!({ "id": "claude-sonnet", "providerID": "anthropic" })
    );
    let ended: Vec<&Value> = all
        .iter()
        .filter(|event| event.kind() == "session.step.ended")
        .map(|event| &event.data["data"])
        .collect();
    assert_eq!(ended.len(), 1);
    assert_eq!(ended[0]["finish"], "stop");
    assert_eq!(ended[0]["rawFinish"], "end_turn");
}

async fn a_follower_going_away_never_stops_the_reply(app: &App) {
    app.create_thread("alice", THREAD).await;
    let script = app.script();
    assert_eq!(
        app.prompt("alice", THREAD, ASK, "keep going").await.status,
        StatusCode::ACCEPTED
    );
    let mut log = app.log("alice", THREAD, "?follow=1").await;
    let first = log.next(WAIT).await.expect("an event");
    assert_eq!(first.kind(), "session.created");
    drop(log);
    script.text("still here");
    drop(script);
    app.settled(THREAD).await;
    let messages = app.messages("alice", THREAD).await;
    assert_eq!(
        messages[0]["content"],
        json!([{ "type": "text", "text": "still here" }])
    );
    assert_eq!(messages[0]["finish"], "stop");
}

async fn malformed_log_queries_are_400(app: &App) {
    app.create_thread("alice", THREAD).await;
    for query in ["?follow=2", "?after=-1", "?after=x"] {
        let log = app.log("alice", THREAD, query).await;
        assert_eq!(log.status, StatusCode::BAD_REQUEST, "{query}");
    }
    let header = app.log_with("alice", THREAD, "", Some("seven")).await;
    assert_eq!(header.status, StatusCode::BAD_REQUEST);
}

async fn a_revoked_phone_loses_its_live_log_within_a_heartbeat(app: &App) {
    app.create_thread("alice", THREAD).await;
    let (device, token) = app.phone("alice").await;
    let mut log = app.log_as(&token, THREAD, "?follow=1").await;
    assert_eq!(log.status, StatusCode::OK);
    assert!(log.next(HEARTBEAT * 3).await.is_none());
    assert!(
        !log.closed(),
        "a live phone keeps its log across heartbeats"
    );
    app.revoke("alice", device).await;
    let revoked = Instant::now();
    let events = log.until_closed().await;
    assert!(events.is_empty(), "{events:?}");
    assert!(
        revoked.elapsed() < HEARTBEAT + SLACK,
        "closed {:?} after the revocation",
        revoked.elapsed()
    );
}

async fn the_log_ends_when_the_token_expires(app: &App) {
    app.create_thread("alice", THREAD).await;
    let ttl = chrono::Duration::seconds(2);
    let (device, _) = app.phone("alice").await;
    let mut desktop = app
        .log_as(&desktop_token_expiring("alice", ttl), THREAD, "?follow=1")
        .await;
    let mut phone = app
        .log_as(
            &mobile_token_expiring("alice", device, ttl),
            THREAD,
            "?follow=1",
        )
        .await;
    assert_eq!(desktop.status, StatusCode::OK);
    assert_eq!(phone.status, StatusCode::OK);
    let opened = Instant::now();
    assert!(desktop.until_closed().await.is_empty());
    assert!(phone.until_closed().await.is_empty());
    let limit = Duration::from_secs(2) + SLACK;
    let lasted = opened.elapsed();
    assert!(lasted < limit, "closed {lasted:?} after opening");
    assert!(
        lasted > Duration::from_millis(500),
        "closed {lasted:?} after opening, before the token expired"
    );
}

http_suite!(
    a_revoked_phone_loses_its_live_log_within_a_heartbeat,
    the_log_ends_when_the_token_expires,
    the_log_replays_then_follows,
    a_reader_joining_mid_reply_gets_the_earlier_deltas_then_the_live_ones,
    after_and_last_event_id_resume_without_gap_or_duplicate,
    events_carry_the_desktop_envelope,
    a_follower_going_away_never_stops_the_reply,
    malformed_log_queries_are_400,
);
