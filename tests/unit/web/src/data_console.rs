//! Read-only SQL validation and malformed transcript evidence regressions.
use systemprompt_web_admin_analytics::repositories::analytics::context_tool_calls::parse_tool_input;
use systemprompt_web_admin_platform::repositories::data_console::query::validate_query;
use systemprompt_web_admin_platform::repositories::data_console::quote_identifier;

#[test]
fn data_console_accepts_read_queries_and_rejects_writes_anywhere() {
    for sql in [
        "SELECT 1",
        "WITH x AS (SELECT 1) SELECT * FROM x",
        "SELECT ';' AS value;",
    ] {
        assert!(validate_query(sql).is_ok(), "{sql}");
    }
    for sql in [
        "DELETE FROM logs",
        "SELECT 1; DELETE FROM logs",
        "SELECT * INTO copy FROM logs",
        "WITH x AS (DELETE FROM logs RETURNING *) SELECT * FROM x",
        "SELECT * FROM logs FOR UPDATE",
        "SELECT * FROM (SELECT * FROM logs FOR SHARE) x",
        "SELECT pg_advisory_lock(1)",
        "SELECT set_config('transaction_read_only','off',true)",
        "SELECT pg_notify('x','y')",
        "",
    ] {
        assert!(validate_query(sql).is_err(), "{sql}");
    }
}
#[test]
fn data_console_quotes_catalog_identifiers() {
    assert_eq!(quote_identifier("odd\"name"), "\"odd\"\"name\"");
}
#[test]
fn malformed_tool_input_preserves_raw_evidence() {
    for raw in [
        "",
        "not JSON",
        "{\"file_path\":\"/tmp/a\",\"content\":\"unfinished",
        "{broken}",
    ] {
        let (parsed, stored, error) = parse_tool_input(raw);
        assert!(parsed.is_null());
        assert_eq!(stored.as_deref(), Some(raw));
        assert!(error.is_some());
    }
    let raw = serde_json::json!({"content":"a".repeat(400_000)}).to_string();
    let (value, stored, error) = parse_tool_input(&raw);
    assert_eq!(value["content"].as_str().unwrap().len(), 400_000);
    assert!(stored.is_none());
    assert!(error.is_none());
}
