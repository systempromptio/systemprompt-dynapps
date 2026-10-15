//! The run report's tool tally: a call is a write when a segment of its name
//! is a mutating verb, MCP calls are counted per server, and the summary is
//! delivered only by a `DynBrain` write that did not fail and names the
//! created document's `id`. An `{"error": …}` body is a failure even without
//! the host's error flag.

use systemprompt::agent_runs::{RunEvent, RunEventPayload};
use systemprompt_web_admin_farm::services::runner::tool_tally::{
    CallOutcome, DeliveryStatus, ServerCalls, call_outcome, is_write_tool, tally,
};

fn event(seq: i64, payload: RunEventPayload) -> RunEvent {
    RunEvent {
        seq,
        kind: payload.kind(),
        tool_name: None,
        tool_use_id: payload.tool_use_id().map(str::to_owned),
        payload,
        created_at: chrono::Utc::now(),
    }
}

fn call(seq: i64, name: &str, result: Option<(bool, &str)>) -> Vec<RunEvent> {
    let id = format!("t{seq}");
    let mut events = vec![event(
        seq,
        RunEventPayload::ToolUse {
            tool_use_id: id.clone(),
            name: name.to_owned(),
            input: serde_json::json!({}),
        },
    )];
    if let Some((is_error, content)) = result {
        events.push(event(
            seq + 1,
            RunEventPayload::ToolResult {
                tool_use_id: id,
                content: content.to_owned(),
                is_error,
            },
        ));
    }
    events
}

fn run(calls: &[(&str, Option<(bool, &str)>)]) -> Vec<RunEvent> {
    calls
        .iter()
        .enumerate()
        .flat_map(|(i, (name, err))| call(i64::try_from(i * 2).unwrap_or(0), name, *err))
        .collect()
}

#[test]
fn writes_are_named_by_a_mutating_verb_segment() {
    for write in [
        "mcp__dynapps-odoo__orm_write",
        "mcp__dynapps-odoo__orm_create",
        "mcp__dynapps-odoo__orm_unlink",
        "mcp__dynapps-odoo__orm_execute_kw",
        "mcp__dynapps-odoo__odoo_request_upload",
        "mcp__dynbrain__document_create",
    ] {
        assert!(is_write_tool(write), "{write}");
    }
    for read in [
        "mcp__dynapps-odoo__orm_search_read",
        "mcp__dynapps-odoo__odoo_describe_model",
        "mcp__dynapps-odoo__odoo_attachment_link",
        "mcp__dynbrain__search",
        "mcp__dynapps-odoo__address_lookup",
    ] {
        assert!(!is_write_tool(read), "{read}");
    }
}

#[test]
fn calls_are_counted_per_server_and_built_ins_are_ignored() {
    let tally = tally(&run(&[
        ("mcp__dynapps-odoo__orm_search_read", Some((false, "[]"))),
        ("mcp__dynapps-odoo__orm_write", Some((true, "denied"))),
        ("mcp__dynbrain__search", Some((false, "[]"))),
        ("Read", Some((false, ""))),
    ]));
    assert_eq!(
        tally.servers,
        [
            ServerCalls {
                server: "dynapps-odoo".to_owned(),
                reads: 1,
                writes: 1,
                failed: 1,
            },
            ServerCalls {
                server: "dynbrain".to_owned(),
                reads: 1,
                writes: 0,
                failed: 0,
            },
        ]
    );
    assert_eq!((tally.reads(), tally.writes()), (2, 1));
    assert_eq!(tally.delivery, DeliveryStatus::NotAttempted);
}

#[test]
fn delivery_needs_a_dynbrain_write_that_names_its_document() {
    let create = "mcp__dynbrain__docs_create";
    let delivered = tally(&run(&[
        (create, Some((true, "boom"))),
        (
            create,
            Some((false, r#"{"id": "doc-7", "title": "Audit"}"#)),
        ),
    ]));
    assert_eq!(delivered.delivery, DeliveryStatus::Delivered);
    let error_body = tally(&run(&[(
        create,
        Some((false, r#"{"error": "Parent not found or not accessible."}"#)),
    )]));
    assert_eq!(error_body.delivery, DeliveryStatus::Failed);
    assert_eq!(
        error_body.servers[0].failed, 1,
        "an error body is a failed call"
    );
    let no_id = tally(&run(&[(create, Some((false, r#"{"ok": true}"#)))]));
    assert_eq!(no_id.delivery, DeliveryStatus::Failed);
    let unanswered = tally(&run(&[(create, None)]));
    assert_eq!(unanswered.delivery, DeliveryStatus::Failed);
    assert_eq!(DeliveryStatus::Delivered.as_str(), "delivered");
    assert_eq!(DeliveryStatus::NotAttempted.as_str(), "not_attempted");
}

#[test]
fn an_error_key_fails_a_call_wherever_the_body_wraps_it() {
    assert_eq!(call_outcome(true, r#"{"id": 1}"#), CallOutcome::Failed);
    assert_eq!(
        call_outcome(false, r#"{"error": "x"}"#),
        CallOutcome::Failed
    );
    assert_eq!(
        call_outcome(false, r#"[{"type": "text", "text": "{\"error\": \"x\"}"}]"#),
        CallOutcome::Failed
    );
    assert_eq!(
        call_outcome(false, r#"[{"type": "text", "text": "{\"id\": \"d1\"}"}]"#),
        CallOutcome::Succeeded { has_id: true }
    );
    assert_eq!(
        call_outcome(false, "plain text"),
        CallOutcome::Succeeded { has_id: false }
    );
    assert_eq!(
        call_outcome(false, r#"{"id": null}"#),
        CallOutcome::Succeeded { has_id: false }
    );
}
