//! Claude Code stream-json decoding against a real `claude -p` transcript.

use std::path::PathBuf;

use systemprompt_web_admin_farm::services::runner::stream_json::{
    StreamEvent, cap_text, parse_stream_line, to_payload,
};
use systemprompt_web_admin_farm::services::runner::stream_payload::TEXT_CAP_BYTES;
use systemprompt_web_admin_farm::types::{RunEventKind, RunEventPayload};

fn sample_events() -> Vec<StreamEvent> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/agent-farm/claude-stream-sample.jsonl");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{path:?}: {err}"));
    raw.lines().flat_map(parse_stream_line).collect()
}

#[test]
fn init_carries_the_claude_session_id() {
    let events = sample_events();
    let init = events.iter().find_map(|event| match event {
        StreamEvent::Init {
            session_id, tools, ..
        } => Some((session_id.clone(), tools.len())),
        _ => None,
    });
    let (session_id, tool_count) = init.unwrap_or_else(|| panic!("no init event"));
    assert_eq!(session_id, "c09fe04e-19d8-4776-b09a-be25d0f53c90");
    assert!(tool_count > 0);
}

#[test]
fn assistant_text_tool_use_and_tool_result_are_decoded_in_order() {
    let events = sample_events();
    let texts: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::AssistantText(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, vec!["Creating the file now.", "done"]);

    let uses: Vec<(&str, &str)> = events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::ToolUse { id, name, .. } => Some((id.as_str(), name.as_str())),
            _ => None,
        })
        .collect();
    assert_eq!(uses, vec![("toolu_014PiN8xptAtW6ZUFftPuwme", "Write")]);

    let results: Vec<(&str, bool)> = events
        .iter()
        .filter_map(|event| match event {
            StreamEvent::ToolResult {
                tool_use_id,
                is_error,
                content,
            } => {
                assert!(content.starts_with("File created successfully"));
                Some((tool_use_id.as_str(), *is_error))
            },
            _ => None,
        })
        .collect();
    assert_eq!(results, vec![("toolu_014PiN8xptAtW6ZUFftPuwme", false)]);
}

#[test]
fn final_result_reports_cost_turns_and_usage() {
    let events = sample_events();
    let Some(StreamEvent::Result(summary)) = events.last() else {
        panic!("last event is not the result");
    };
    assert!(summary.succeeded());
    assert_eq!(summary.num_turns, Some(2));
    assert_eq!(summary.result.as_deref(), Some("done"));
    let cost = summary.total_cost_usd.unwrap_or_default();
    assert!((cost - 0.397).abs() < 0.001, "cost {cost}");
    assert_eq!(summary.usage.input, 34);
    assert_eq!(summary.usage.output, 150);
    assert_eq!(summary.usage.cache_read_input, 41085);
    assert_eq!(summary.usage.cache_creation_input, 18948);
}

#[test]
fn rate_limit_and_hook_start_lines_yield_nothing() {
    assert!(parse_stream_line(r#"{"type":"rate_limit_event","rate_limit_info":{}}"#).is_empty());
    assert!(
        parse_stream_line(r#"{"type":"system","subtype":"hook_started","hook_name":"x"}"#)
            .is_empty()
    );
    assert!(parse_stream_line("not json").is_empty());
    assert!(parse_stream_line("").is_empty());
}

#[test]
fn every_event_maps_to_a_payload_of_the_matching_kind() {
    for event in sample_events() {
        let payload = to_payload(&event);
        let expected = match event {
            StreamEvent::Init { .. } => RunEventKind::Init,
            StreamEvent::Hook { .. } => RunEventKind::Hook,
            StreamEvent::AssistantText(_) => RunEventKind::AssistantText,
            StreamEvent::ToolUse { .. } => RunEventKind::ToolUse,
            StreamEvent::ToolResult { .. } => RunEventKind::ToolResult,
            StreamEvent::Result(_) => RunEventKind::Result,
        };
        assert_eq!(payload.kind(), expected);
        let json = serde_json::to_value(&payload).unwrap_or_else(|err| panic!("{err}"));
        let back: RunEventPayload =
            serde_json::from_value(json).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(back, payload);
    }
}

#[test]
fn long_text_is_capped_on_a_char_boundary() {
    let long = "é".repeat(TEXT_CAP_BYTES);
    let capped = cap_text(&long);
    assert!(capped.len() < long.len());
    assert!(capped.contains("[truncated"));
    assert_eq!(cap_text("short"), "short");
}
