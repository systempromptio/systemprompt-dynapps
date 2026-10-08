//! The trace page must answer an MCP execution's ids, not only a gateway
//! trace: the pure rules behind `resolve_trace_session` and the ledger span
//! source, which choose the key an execution is queried by and the status
//! its span shows. Both are exercised without a database.


use systemprompt::identifiers::{McpExecutionId, SessionId, TraceId};
use systemprompt_web_admin_analytics::repositories::traces::{
    execution_label, execution_status, execution_trace_key,
};

#[test]
fn execution_with_a_session_keys_on_that_session() {
    let key = execution_trace_key(
        Some(SessionId::new("sess_1")),
        Some(&TraceId::new("trace-only-here")),
        &McpExecutionId::new("exec-1"),
    );
    assert_eq!(key.as_str(), "sess_1");
}

#[test]
fn execution_only_trace_id_resolves_to_itself() {
    // Why: a proxied call's trace id exists in mcp_tool_executions alone —
    // the id the MCP server page links must resolve, not 404.
    let key = execution_trace_key(
        None,
        Some(&TraceId::new("trace-only-here")),
        &McpExecutionId::new("exec-1"),
    );
    assert_eq!(key.as_str(), "trace-only-here");
}

#[test]
fn execution_without_session_or_trace_keys_on_its_own_id() {
    let key = execution_trace_key(None, None, &McpExecutionId::new("exec-1"));
    assert_eq!(key.as_str(), "exec-1");
}

#[test]
fn execution_status_follows_the_execution_then_the_result() {
    let cases = [
        ("success", false, "ok"),
        ("success", true, "error"),
        ("pending", false, "pending"),
        ("failed", false, "error"),
        ("timeout", false, "error"),
    ];
    for (status, is_error, expected) in cases {
        assert_eq!(
            execution_status(status, is_error).as_str(),
            expected,
            "status={status} is_error={is_error}"
        );
    }
}

#[test]
fn execution_label_names_server_and_source() {
    assert_eq!(execution_label("crm", "proxy"), "crm · proxy");
}
