//! Observable reporting invariants, including hostile input and incomplete
//! source data.
use chrono::{TimeZone, Utc};
use serde_json::json;
use systemprompt_mcp_agent::reports::{ReportInput, normalize_cli, project_metrics};

#[test]
fn project_metrics_separate_stale_overdue_and_completed_work() {
    let now = Utc
        .with_ymd_and_hms(2026, 9, 7, 12, 0, 0)
        .single()
        .expect("date");
    let issues = vec![
        json!({"fields":{"status":{"statusCategory":{"key":"indeterminate"}},"assignee":null,"duedate":"2026-09-06","updated":"2026-08-01T00:00:00Z"}}),
        json!({"fields":{"status":{"statusCategory":{"key":"done"}},"assignee":null,"duedate":"2026-08-01","updated":"2026-08-01T00:00:00Z","resolutiondate":"2026-09-06T00:00:00Z"}}),
        json!({"fields":{"status":{"statusCategory":{"key":"new"}},"assignee":{"displayName":"A"},"duedate":"2026-09-07","updated":"2026-09-06T00:00:00Z"}}),
    ];
    let result = project_metrics(&issues, now, 7);
    assert_eq!(result["Overdue unresolved"], 1);
    assert_eq!(result["Unassigned unresolved"], 1);
    assert_eq!(result["Unchanged for 7 days"], 1);
    assert_eq!(result["Completed in period"], 1);
    assert_eq!(result["Issues in sample"], 3);
}

#[test]
fn unsupported_cli_text_is_an_error_not_an_empty_report() {
    assert!(normalize_cli(&json!({"content":"Permission denied"})).is_err());
    assert!(normalize_cli(&json!({"something":"else"})).is_err());
    assert!(
        normalize_cli(&json!({"artifact_type":"table","items":[]}))
            .expect("empty")
            .is_empty()
    );
    let wrapped = json!({"content":"{\"items\":[{\"total_cost_microdollars\":1234567}]}"});
    assert_eq!(
        normalize_cli(&wrapped).expect("wrapped")[0]["total_cost_microdollars"],
        1234567
    );
}

#[test]
fn report_input_cannot_override_command_or_identity() {
    for field in ["command", "profile", "database_url", "user_id"] {
        assert!(
            serde_json::from_value::<ReportInput>(json!({"report":"operations",field:"injected"}))
                .is_err()
        );
    }
    assert!(
        serde_json::from_value::<ReportInput>(json!({"report":"projects","project_key":"DEMO"}))
            .is_err()
    );
}

#[tokio::test]
async fn atlassian_reads_forward_the_callers_bearer_and_decode_structured_results() {
    use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId};
    use systemprompt::models::execution::context::RequestContext;
    use systemprompt_mcp_agent::reports::Atlassian;
    use wiremock::matchers::{body_partial_json, header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    let server = MockServer::start().await;
    // MCP notification requests have no id and need only an HTTP acknowledgement.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(202))
        .mount(&server)
        .await;
    for (rpc_method, result) in [
        (
            "initialize",
            json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"atlassian-test","version":"1"}}),
        ),
        (
            "tools/list",
            json!({"tools":[{"name":"getAccessibleAtlassianResources","inputSchema":{"type":"object","properties":{}}}]}),
        ),
        (
            "tools/call",
            json!({"content":[],"structuredContent":{"resources":[{"id":"site-a","name":"Account A"}]}}),
        ),
    ] {
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer account-a-token"))
            .and(body_partial_json(json!({"method":rpc_method})))
            .respond_with(move |request: &wiremock::Request| {
                let body: serde_json::Value =
                    serde_json::from_slice(&request.body).expect("rpc request");
                ResponseTemplate::new(200)
                    .set_body_json(json!({"jsonrpc":"2.0","id":body["id"],"result":result}))
            })
            .with_priority(1)
            .mount(&server)
            .await;
    }
    let context = RequestContext::new(
        SessionId::new("report-session"),
        TraceId::new("report-trace"),
        ContextId::new_unchecked("00000000-0000-4000-8000-000000000001"),
        AgentName::new("report-test"),
    );
    let client = Atlassian::connect_gateway(&server.uri(), "account-a-token", &context)
        .await
        .expect("connect");
    let data = client
        .read(&["getAccessibleAtlassianResources"], json!({}))
        .await
        .expect("read");
    assert_eq!(data["resources"][0]["id"], "site-a");
}

#[tokio::test]
async fn report_data_renders_as_a_deterministic_mcp_asset() {
    use systemprompt::identifiers::{ArtifactId, ContextId};
    use systemprompt::mcp::services::ui_renderer::{RenderTarget, artifact_ui_resource};
    let data = json!({"report":"costs","title":"AI Cost & Adoption","checked_at":"2026-09-07T12:00:00Z",
        "period":"7 days","complete":true,"sources":[],"metrics":{},"tables":[],
        "request":{"report":"costs","days":7}});
    let html = artifact_ui_resource(&RenderTarget {
        artifact_id: &ArtifactId::generate(),
        artifact_type: "dashboard",
        payload: &data,
        context_id: ContextId::new_unchecked("00000000-0000-4000-8000-000000000002"),
        title: Some("Costs".into()),
    })
    .await
    .expect("render")
    .html;
    assert!(html.contains("ASTOUND DIGITAL"));
    assert!(html.contains("admin-read-"));
    assert!(html.contains("2026-09-07T12:00:00Z"));
    assert!(!html.contains("/*REPORT_DATA*/"));
    assert!(!html.contains("/*MCP_BRIDGE*/"));
}

#[test]
fn cost_chart_normalization_preserves_dollars_and_refuses_misaligned_data() {
    let chart = json!({"labels":["2026-09-07"],"datasets":[
        {"label":"cost_usd","data":[1.25]}, {"label":"requests","data":[4.0]}
    ]});
    let rows = normalize_cli(&chart).expect("chart");
    assert_eq!(rows[0]["cost_usd"], 1.25);
    assert_eq!(rows[0]["period"], "2026-09-07");
    assert!(
        normalize_cli(&json!({"labels":["today"],"datasets":[{"label":"cost_usd","data":[]}]}))
            .is_err()
    );
}
