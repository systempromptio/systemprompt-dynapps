//! `bound_artifact` keeps a passthrough result under the wire bound: a table
//! that fits is untouched, a table that does not loses trailing rows and
//! says so in its title, and any other oversized shape becomes truncated
//! text — never a spilled file.

use serde_json::json;
use systemprompt::models::artifacts::{
    CliArtifact, Column, ColumnType, TableArtifact, TextArtifact,
};
use systemprompt_mcp_agent::bounds::{MAX_RESULT_BYTES, bound_artifact};

fn table(rows: usize, cell: &str) -> CliArtifact {
    let items = (0..rows)
        .map(|i| json!({"request_id": format!("req_{i}"), "content": cell}))
        .collect();
    CliArtifact::Table {
        artifact: TableArtifact::new(vec![
            Column::new("request_id", ColumnType::String),
            Column::new("content", ColumnType::String),
        ])
        .with_title("AI Requests")
        .with_rows(items),
    }
}

#[test]
fn a_small_table_passes_through_untouched() {
    let bounded = bound_artifact(table(5, "hello"), "infra logs request list");
    assert!(!bounded.truncated);
    assert!(bounded.rows.is_none());
    let CliArtifact::Table { artifact } = bounded.artifact else {
        panic!("shape must be preserved");
    };
    assert_eq!(artifact.items.len(), 5);
    assert_eq!(artifact.title.as_deref(), Some("AI Requests"));
}

#[test]
fn an_oversized_table_keeps_a_prefix_of_rows_and_names_the_cut() {
    let big = "x".repeat(1024);
    let bounded = bound_artifact(table(200, &big), "infra logs request list -n 200");
    assert!(bounded.truncated);
    let (kept, received) = bounded.rows.expect("row accounting");
    assert_eq!(received, 200);
    assert!(kept > 0 && kept < 200, "kept {kept}");
    let CliArtifact::Table { artifact } = bounded.artifact else {
        panic!("a table stays a table");
    };
    assert_eq!(artifact.items.len(), kept);
    assert!(serde_json::to_vec(&artifact).unwrap().len() <= MAX_RESULT_BYTES);
    assert_eq!(
        artifact.items[0]["request_id"], "req_0",
        "the newest rows survive"
    );
    let title = artifact.title.unwrap_or_default();
    assert!(title.contains(&format!("{kept} of 200 rows")), "{title}");
    assert!(title.contains("--before"), "{title}");
    assert!(title.contains("conversation_audit"), "{title}");
}

#[test]
fn an_oversized_card_becomes_bounded_text_on_a_char_boundary() {
    let body = "é".repeat(MAX_RESULT_BYTES);
    let big = CliArtifact::text(TextArtifact::new(body).with_title("Audit"));
    let bounded = bound_artifact(big, "infra logs audit req_1");
    assert!(bounded.truncated);
    let CliArtifact::Text { artifact } = bounded.artifact else {
        panic!("oversized non-table output becomes text");
    };
    assert!(artifact.content.ends_with("..."));
    assert!(artifact.content.len() <= MAX_RESULT_BYTES + 3);
    assert!(artifact.title.unwrap_or_default().contains("truncated"));
}
