//! Project reporting preserves evidence across pagination and partial failures.
use serde_json::{Value, json};
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_agent::reports::{Atlassian, ReportInput, project_report};
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_rpc(server: &MockServer, pattern: Value, result: Value) {
    let priority = if pattern["params"]["arguments"]["nextPageToken"].is_string() {
        1
    } else {
        2
    };
    Mock::given(method("POST"))
        .and(body_partial_json(pattern))
        .respond_with(move |request: &wiremock::Request| {
            let body: Value = serde_json::from_slice(&request.body).expect("RPC body");
            ResponseTemplate::new(200)
                .set_body_json(json!({"jsonrpc":"2.0","id":body["id"],"result":result}))
        })
        .with_priority(priority)
        .mount(server)
        .await;
}

async fn fixture() -> (MockServer, Atlassian) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(202))
        .mount(&server)
        .await;
    mock_rpc(&server, json!({"method":"initialize"}), json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"atlassian-test","version":"1"}})).await;
    mock_rpc(&server, json!({"method":"tools/list"}), json!({"tools":[
        {"name":"getAccessibleAtlassianResources","inputSchema":{"type":"object","properties":{}}},
        {"name":"searchJiraIssuesUsingJql","inputSchema":{"type":"object","properties":{}}}
    ]})).await;
    mock_rpc(&server, json!({"method":"tools/call","params":{"name":"getAccessibleAtlassianResources"}}), json!({"content":[],"structuredContent":{"resources":[{"id":"site-a","url":"https://example.atlassian.net"}]}})).await;
    let context = RequestContext::new(
        SessionId::new("report-session"),
        TraceId::new("report-trace"),
        ContextId::new_unchecked("00000000-0000-4000-8000-000000000003"),
        AgentName::new("report-test"),
    );
    let client = Atlassian::connect_gateway(&server.uri(), "test-token", &context)
        .await
        .expect("connect");
    (server, client)
}

fn request(cloud: &str, project: &str) -> ReportInput {
    ReportInput::Projects {
        cloud_id: cloud.into(),
        project_key: project.into(),
        days: 7,
        space_id: None,
    }
}

#[tokio::test]
async fn project_pagination_deduplicates_and_marks_complete_only_after_the_last_page() {
    let (server, client) = fixture().await;
    let issue = json!({"key":"DEMO-1","fields":{"summary":"First","status":{"name":"Open","statusCategory":{"key":"new"}}}});
    mock_rpc(&server, json!({"method":"tools/call","params":{"name":"searchJiraIssuesUsingJql"}}), json!({"content":[],"structuredContent":{"issues":[issue.clone()],"nextPageToken":"page-2","isLast":false}})).await;
    mock_rpc(&server, json!({"method":"tools/call","params":{"name":"searchJiraIssuesUsingJql","arguments":{"nextPageToken":"page-2"}}}), json!({"content":[],"structuredContent":{"issues":[issue],"isLast":true}})).await;
    let report = project_report(&client, &request("site-a", "DEMO"))
        .await
        .expect("report");
    assert!(report.complete);
    assert_eq!(report.metrics["Issues in sample"], 1);
    assert_eq!(report.tables[0].rows.len(), 1);
    assert_eq!(
        report.tables[0].rows[0]["url"],
        "https://example.atlassian.net/browse/DEMO-1"
    );
    let calls = server.received_requests().await.expect("requests");
    let searches: Vec<Value> = calls
        .iter()
        .filter_map(|r| serde_json::from_slice::<Value>(&r.body).ok())
        .filter(|r| r["params"]["name"] == "searchJiraIssuesUsingJql")
        .collect();
    assert_eq!(searches.len(), 2);
    assert_eq!(
        searches[0]["params"]["arguments"]["jql"],
        "project = \"DEMO\" ORDER BY key ASC"
    );
}

#[tokio::test]
async fn project_reports_keep_partial_rows_and_reject_inaccessible_sites_and_jql_injection() {
    let (server, client) = fixture().await;
    mock_rpc(&server, json!({"method":"tools/call","params":{"name":"searchJiraIssuesUsingJql"}}), json!({"content":[],"structuredContent":{"issues":[{"key":"DEMO-1","fields":{}}],"nextPageToken":"page-2","isLast":false}})).await;
    mock_rpc(&server, json!({"method":"tools/call","params":{"name":"searchJiraIssuesUsingJql","arguments":{"nextPageToken":"page-2"}}}), json!({"isError":true,"content":[]})).await;
    let report = project_report(&client, &request("site-a", "DEMO"))
        .await
        .expect("partial report");
    assert!(!report.complete);
    assert_eq!(report.metrics["Issues in sample"], 1);
    assert_eq!(report.metrics["Unassigned unresolved"], 0);
    assert!(
        report
            .sources
            .iter()
            .any(|s| s.source == "Jira field coverage" && !s.complete)
    );
    assert!(
        project_report(&client, &request("site-b", "DEMO"))
            .await
            .is_err()
    );
    assert!(
        project_report(
            &client,
            &request("site-a", "DEMO\" OR project IS NOT EMPTY")
        )
        .await
        .is_err()
    );
}
