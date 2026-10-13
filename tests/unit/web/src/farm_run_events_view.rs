//! The run page's event view model: titles and bodies per payload,
//! `tool_use` ⇄ `tool_result` pairing, offsets from the run start, body
//! folding, the after-seq filter the live fragment uses, and the timeline
//! geometry. Events are core's `RunEvent`s, read from `agent_run_events`.

use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::{Value, json};
use systemprompt_web_admin_farm::types::{RunEvent, RunEventPayload};
use systemprompt_web_admin_farm::views::run_events::{
    FOLD_CHARS, FRAGMENT_EVENT_CAP, MIN_MARK_WIDTH, PairKey, RunEventView, VIEW_WIDTH, event_views,
    events_after, pair_seqs, scale, timeline_view, transcript_views,
};

fn start() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 12, 9, 0, 0).unwrap()
}

fn row(seq: i64, at_ms: i64, payload: Value) -> RunEvent {
    let payload = serde_json::from_value(payload).expect("payload matches core's contract");
    let event = RunEvent {
        seq,
        kind: RunEventPayload::kind(&payload),
        tool_name: None,
        tool_use_id: None,
        payload,
        created_at: start() + Duration::milliseconds(at_ms),
    };
    with_tool_fields(event)
}

fn with_tool_fields(mut event: RunEvent) -> RunEvent {
    event.tool_use_id = event.payload.tool_use_id().map(str::to_owned);
    if let RunEventPayload::ToolUse { name, .. } = &event.payload {
        event.tool_name = Some(name.clone());
    }
    event
}

fn sample() -> Vec<RunEvent> {
    vec![
        row(
            1,
            0,
            json!({"kind": "init", "session_id": "s-1", "model": "claude", "tools": ["Bash"], "cwd": "/w"}),
        ),
        row(
            2,
            1_000,
            json!({"kind": "assistant_text", "text": "Looking at the repo."}),
        ),
        row(
            3,
            2_000,
            json!({"kind": "tool_use", "tool_use_id": "tu-1", "name": "Bash", "input": {"command": "ls -la"}}),
        ),
        row(
            4,
            4_000,
            json!({"kind": "tool_result", "tool_use_id": "tu-1", "content": "a.txt", "is_error": false}),
        ),
        row(
            5,
            5_000,
            json!({"kind": "tool_use", "tool_use_id": "tu-2", "name": "Read", "input": {"file_path": "/w/a.txt"}}),
        ),
        row(6, 6_000, json!({"kind": "stderr", "line": "warning"})),
        row(
            7,
            9_000,
            json!({"kind": "result", "subtype": "success", "is_error": false, "num_turns": 2, "duration_ms": 9000, "input_tokens": 10, "output_tokens": 5, "cost_microdollars": 10_000}),
        ),
    ]
}

fn views() -> Vec<RunEventView> {
    event_views(&sample(), Some(start()))
}

fn by_seq(views: &[RunEventView], seq: i64) -> &RunEventView {
    views.iter().find(|v| v.seq == seq).unwrap()
}

#[test]
fn tool_use_and_result_pair_both_ways() {
    let v = views();
    assert_eq!(by_seq(&v, 3).paired_seq, Some(4));
    assert_eq!(by_seq(&v, 4).paired_seq, Some(3));
    assert!(by_seq(&v, 3).is_tool_use);
    assert!(!by_seq(&v, 4).is_tool_use);
    assert_eq!(
        by_seq(&v, 5).paired_seq,
        None,
        "a call still running has no result"
    );
    assert_eq!(by_seq(&v, 2).paired_seq, None);
}

#[test]
fn a_result_borrows_its_calls_tool_name() {
    let rows = sample();
    assert_eq!(
        rows[3].tool_name, None,
        "core leaves tool_name off a result"
    );
    let v = event_views(&rows, Some(start()));
    assert_eq!(by_seq(&v, 4).tool_name.as_deref(), Some("Bash"));
    assert_eq!(by_seq(&v, 4).title, "Bash returned");
    assert_eq!(by_seq(&v, 3).title, "Bash · ls -la");
    assert_eq!(by_seq(&v, 5).title, "Read · /w/a.txt");
}

#[test]
fn pairing_ignores_orphans_and_duplicates() {
    let keys = [
        PairKey {
            seq: 1,
            tool_use_id: Some("a"),
            is_use: true,
        },
        PairKey {
            seq: 2,
            tool_use_id: Some("b"),
            is_use: false,
        },
        PairKey {
            seq: 3,
            tool_use_id: Some("a"),
            is_use: false,
        },
        PairKey {
            seq: 4,
            tool_use_id: Some("a"),
            is_use: false,
        },
        PairKey {
            seq: 5,
            tool_use_id: None,
            is_use: false,
        },
    ];
    assert_eq!(
        pair_seqs(&keys),
        vec![Some(3), None, Some(1), Some(1), None]
    );
}

#[test]
fn offsets_measure_from_the_run_start_or_the_first_event() {
    let v = views();
    assert_eq!(by_seq(&v, 1).offset_ms, 0);
    assert_eq!(by_seq(&v, 4).offset_ms, 4_000);
    assert!(by_seq(&v, 4).offset_label.starts_with('+'));
    let late_start = event_views(&sample(), Some(start() + Duration::seconds(30)));
    assert_eq!(
        late_start[0].offset_ms, 0,
        "an event before the start clamps to zero"
    );
    let no_start = event_views(&sample()[2..], None);
    assert_eq!(no_start[0].offset_ms, 0);
    assert_eq!(no_start[1].offset_ms, 2_000);
}

#[test]
fn long_bodies_fold_and_short_ones_do_not() {
    let long = "x".repeat(FOLD_CHARS + 1);
    let many_lines = (0..40)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let rows = vec![
        row(1, 0, json!({"kind": "assistant_text", "text": "short"})),
        row(2, 0, json!({"kind": "assistant_text", "text": long})),
        row(
            3,
            0,
            json!({"kind": "tool_result", "tool_use_id": "t", "content": many_lines, "is_error": true}),
        ),
    ];
    let v = event_views(&rows, Some(start()));
    assert!(!v[0].body_is_long);
    assert!(v[1].body_is_long);
    assert!(v[1].preview.ends_with('…'));
    assert!(v[2].body_is_long);
    assert_eq!(v[2].preview, "line 0");
    assert!(v[2].is_error);
}

#[test]
fn events_after_is_strict_and_capped() {
    let v = views();
    let after: Vec<i64> = events_after(&v, 4).iter().map(|e| e.seq).collect();
    assert_eq!(after, vec![5, 6, 7]);
    assert!(events_after(&v, 7).is_empty());
    let many: Vec<RunEvent> = (1..=500)
        .map(|i| row(i, i, json!({"kind": "stderr", "line": "x"})))
        .collect();
    let mv = event_views(&many, Some(start()));
    let page = events_after(&mv, 0);
    assert_eq!(page.len(), FRAGMENT_EVENT_CAP);
    assert_eq!(page.last().unwrap().seq, FRAGMENT_EVENT_CAP as i64);
}

#[test]
fn transcript_keeps_only_prose() {
    let kinds: Vec<String> = transcript_views(&views())
        .into_iter()
        .map(|v| v.kind)
        .collect();
    assert_eq!(kinds, vec!["assistant_text", "result"]);
}

#[test]
fn timeline_draws_lanes_in_permille_of_the_window() {
    let t = timeline_view(&views(), 10_000);
    assert!((t.view_width - VIEW_WIDTH).abs() < f64::EPSILON);
    assert!(t.has_marks);
    let assistant = t.marks.iter().find(|m| m.lane == "assistant").unwrap();
    assert!((assistant.x - 100.0).abs() < 0.01);
    assert!(
        (assistant.width - 100.0).abs() < 0.01,
        "a turn lasts until the next event"
    );
    let done = t.marks.iter().find(|m| m.seq == 3).unwrap();
    assert_eq!(done.tone, "ok");
    assert!((done.x - 200.0).abs() < 0.01);
    assert!((done.width - 200.0).abs() < 0.01);
    let open = t.marks.iter().find(|m| m.seq == 5).unwrap();
    assert_eq!(open.tone, "open");
    assert!(
        (open.x + open.width - VIEW_WIDTH).abs() < 0.01,
        "a running call reaches the window's end"
    );
    assert!(
        t.marks
            .iter()
            .all(|m| m.width >= MIN_MARK_WIDTH && m.x + m.width <= VIEW_WIDTH + 0.01)
    );
    assert_eq!(t.ticks.len(), 1);
    assert_eq!(t.ticks[0].kind, "stderr");
    assert_eq!(t.axis.len(), 5);
}

#[test]
fn timeline_widens_to_the_last_event_and_survives_an_empty_run() {
    let t = timeline_view(&views(), 1_000);
    assert!(t.marks.iter().all(|m| m.x + m.width <= VIEW_WIDTH + 0.01));
    assert!((scale(9_000, 9_000) - VIEW_WIDTH).abs() < f64::EPSILON);
    let empty = timeline_view(&[], 0);
    assert!(!empty.has_marks);
    assert!(empty.marks.is_empty());
    assert!(
        (scale(5, 0) - 5_000.0).abs() < f64::EPSILON,
        "window clamps to 1 ms"
    );
}
