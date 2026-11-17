//! Records a cloud reply's events as the phone's and the desktop's shape
//! checks read them: `fixtures/cloud-reply.jsonl`, one meta line and one
//! `{ "kind": "event", "event" }` line per `/log` event, in the line format
//! of the `@dynwork/mobile-model` fixtures. The thread, the prompt and the
//! clock are fixed and the server derives the reply's and the events' ids
//! from them, so the file only changes when the events do.

use std::path::PathBuf;

use axum::http::StatusCode;
use serde_json::json;

use crate::http_support::memory;
use crate::reply_support::{deltas, kinds};

const THREAD: &str = "ses_01JCLOUDREPLYRECORD0000001";
const ASK: &str = "msg_01JCLOUDREPLYPROMPT000001";

const CONTRACT_EVENTS: [&str; 26] = [
    "session.created",
    "session.renamed",
    "session.archived",
    "session.unarchived",
    "session.deleted",
    "session.model.selected",
    "session.execution.started",
    "session.execution.succeeded",
    "session.execution.failed",
    "session.execution.interrupted",
    "session.step.started",
    "session.step.ended",
    "session.usage.updated",
    "session.text.started",
    "session.text.delta",
    "session.text.ended",
    "session.reasoning.started",
    "session.reasoning.delta",
    "session.reasoning.ended",
    "session.tool.input.started",
    "session.tool.input.delta",
    "session.tool.input.ended",
    "session.tool.called",
    "session.tool.progress",
    "session.tool.success",
    "session.tool.failed",
];

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/cloud-reply.jsonl")
}

#[tokio::test]
async fn records_a_cloud_reply_for_the_shape_check() {
    let app = memory();
    app.create_thread("alice", THREAD).await;
    app.scripted(&[
        "Hello from the cloud. ",
        "This reply streams ",
        "as it is written.",
    ]);
    let answer = app.prompt("alice", THREAD, ASK, "Say hello").await;
    assert_eq!(answer.status, StatusCode::ACCEPTED);
    app.settled(THREAD).await;
    let events = app
        .log("alice", THREAD, "?follow=0")
        .await
        .until_closed()
        .await;
    assert!(
        kinds(&events)
            .iter()
            .all(|kind| CONTRACT_EVENTS.contains(kind)),
        "{:?}",
        kinds(&events)
    );
    let ended = events
        .iter()
        .find(|event| event.kind() == "session.text.ended")
        .expect("the text ends");
    assert_eq!(ended.data["data"]["text"], deltas(&events).as_str());
    let meta = json!({
        "kind": "meta",
        "scenario": "cloud-reply",
        "description": "A cloud chat reply recorded from dynwork-chats (phase 03 recorder, memory store, scripted model)",
        "version": "cloud",
        "root": THREAD,
        "sessions": [THREAD],
    });
    let mut lines = vec![meta.to_string()];
    lines.extend(
        events
            .iter()
            .map(|event| json!({ "kind": "event", "event": event.data }).to_string()),
    );
    let text = format!("{}\n", lines.join("\n"));
    let path = fixture();
    std::fs::create_dir_all(path.parent().expect("a fixtures dir")).expect("create fixtures/");
    std::fs::write(&path, text).expect("write the recording");
    let written = std::fs::read_to_string(&path).expect("read the recording back");
    assert_eq!(written.lines().count(), events.len() + 1);
    app.finish().await;
}
