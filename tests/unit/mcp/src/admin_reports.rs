//! Observable reporting invariants, including hostile input and incomplete
//! source data.
use serde_json::json;
use systemprompt_mcp_agent::reports::{ReportInput, normalize_cli, with_dollar_siblings};

#[test]
fn report_input_cannot_override_command_or_identity() {
    // `deny_unknown_fields` is what keeps a caller from smuggling a CLI
    // argument or an identity through the report input.
    for field in ["command", "profile", "database_url", "user_id"] {
        assert!(
            serde_json::from_value::<ReportInput>(json!({"report":"costs",field:"injected"}))
                .is_err(),
            "`{field}` must not deserialize into a report request"
        );
        assert!(
            serde_json::from_value::<ReportInput>(json!({"report":"costs","days":7,field:"x"}))
                .is_err(),
            "`{field}` must not deserialize alongside a valid request"
        );
    }
    // An unknown report kind is refused at the discriminator.
    // Every kind but `costs` is gone: Jira and Confluence are read by the
    // client directly, so a request naming them must be refused rather than
    // silently treated as a cost report.
    for gone in ["operations", "users", "projects", "brief", "activity"] {
        assert!(
            serde_json::from_value::<ReportInput>(json!({ "report": gone })).is_err(),
            "`{gone}` is not a report this server serves"
        );
    }
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
    // The brand mark is written once, in title case, and uppercased by CSS —
    // asserting the rendered casing pinned a stylesheet detail rather than the
    // brand. "Astound Digital" is the required display name (CLAUDE.md rule 5).
    assert!(html.contains("Astound Digital"));
    // The page must not claim to show delivery health: its data is the audit
    // tables, and Jira never reaches it.
    assert!(html.contains("AI Usage"));
    assert!(!html.contains("traffic light"));
    assert!(html.contains("admin-read-"));
    assert!(html.contains("2026-09-07T12:00:00Z"));
    // Both placeholders must be substituted, or the page renders with no data
    // and no bridge and silently shows nothing.
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

// Why: a renderer that cannot read its payload must SAY so. It used to fall
// through to core's DashboardRenderer, which cannot read a ReportOutput either
// — so the render failed, core dropped the embedded resource, and the tool
// returned success with no dashboard and no explanation. Silence looked
// identical to "this host does not support artifacts".
#[tokio::test]
async fn a_payload_that_is_not_a_report_fails_loudly() {
    use systemprompt::identifiers::{ArtifactId, ContextId};
    use systemprompt::mcp::services::ui_renderer::{RenderTarget, artifact_ui_resource};

    let error = artifact_ui_resource(&RenderTarget {
        artifact_id: &ArtifactId::generate(),
        artifact_type: "dashboard",
        payload: &json!({"report": "costs", "tables": "not-an-array"}),
        context_id: ContextId::new_unchecked("00000000-0000-4000-8000-000000000004"),
        title: Some("Broken".into()),
    })
    .await;

    assert!(
        error.is_err(),
        "a payload that does not match ReportOutput must not render as if it did"
    );
}

// Why: the model that read this report quoted "$12.11 millicents" from
// `avg_cost_per_request_microdollars`. Every microdollar cell must arrive
// with a `_usd` sibling so the converted figure is the one to quote, and the
// raw store stays beside it for reconciliation.
#[test]
fn every_microdollar_cell_gains_a_dollar_sibling() {
    let rows = with_dollar_siblings(vec![
        json!({"model":"claude-sonnet-5","total_cost_microdollars":15_150_809,"request_count":320}),
        json!({"heading":"total_cost_microdollars","content":30_569_751}),
        json!({"heading":"total_requests","content":2524}),
        json!("not an object"),
    ]);

    assert_eq!(rows[0]["total_cost_usd"], json!(15.150_809));
    assert_eq!(
        rows[0]["total_cost_microdollars"],
        json!(15_150_809),
        "the raw value is kept, not replaced"
    );
    assert!(
        rows[0].get("request_count_usd").is_none(),
        "only money is converted"
    );

    assert_eq!(rows[1]["heading"], json!("total_cost_microdollars"));
    assert_eq!(
        rows[2],
        json!({"heading":"total_cost_usd","content":30.569_751}),
        "a heading/content pair gains a sibling row, keeping the two-column shape"
    );
    assert_eq!(
        rows[3]["heading"],
        json!("total_requests"),
        "non-money pairs are untouched"
    );
    assert_eq!(
        rows[4],
        json!("not an object"),
        "non-object rows pass through"
    );
    assert_eq!(rows.len(), 5);
}
