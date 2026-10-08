//! Tool schema meta-validation (REQ-043).
//!
//! `systemprompt_mcp_shared::tool_schema` is the one checker the `systemprompt`
//! MCP server runs before it serves and the audit detail page runs on every
//! client schema. These cases pin the four outcomes the roadmap names: a well
//! formed schema passes, and a missing top-level `type`, a `required` key
//! `properties` does not declare, and a `$ref` that leaves the document each
//! fail with a message naming the violation. The repo fixtures under
//! `tests/fixtures/mcp-tool-schemas/` are the same ones the
//! `check-mcp-tool-schemas.sh` gate reads.

use serde_json::{Value, json};
use std::path::PathBuf;
use systemprompt_mcp_shared::tool_schema::{SchemaSide, schema_violations, tool_violations};

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/mcp-tool-schemas")
        .join(name);
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn a_well_formed_schema_passes_under_both_drafts() {
    for name in [
        "valid/draft-2020-12-object.json",
        "valid/draft-07-object.json",
    ] {
        assert_eq!(
            schema_violations(&fixture(name)),
            Vec::<String>::new(),
            "{name}"
        );
    }
}

#[test]
fn a_schema_without_top_level_type_object_fails() {
    let found = schema_violations(&fixture("invalid/missing-type.json"));
    assert!(
        found.iter().any(|v| v.contains("type: object")),
        "{found:?}"
    );
}

#[test]
fn a_required_key_that_properties_does_not_declare_fails() {
    let found = schema_violations(&fixture("invalid/required-not-declared.json"));
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("'workdir'"), "{found:?}");
}

#[test]
fn required_without_any_properties_fails() {
    let found = schema_violations(&json!({"type": "object", "required": ["command"]}));
    assert!(
        found.iter().any(|v| v.contains("no `properties`")),
        "{found:?}"
    );
}

#[test]
fn an_external_ref_fails_and_a_local_one_does_not() {
    let found = schema_violations(&fixture("invalid/external-ref.json"));
    assert!(
        found
            .iter()
            .any(|v| v.contains("https://example.com/schemas/filter.json")),
        "{found:?}"
    );
}

#[test]
fn a_meta_schema_violation_fails() {
    let found = schema_violations(&fixture("invalid/meta-schema-violation.json"));
    assert!(found.iter().any(|v| v.contains("meta-schema")), "{found:?}");
}

#[test]
fn a_violation_names_server_tool_and_side() {
    let input = json!({"type": "object", "properties": {}});
    let output = json!({"type": "string"});
    let found = tool_violations("systemprompt", "users", &input, Some(&output));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].side, SchemaSide::Output);
    let message = found[0].to_string();
    assert!(
        message.contains("'systemprompt'") && message.contains("'users'"),
        "{message}"
    );
    assert!(message.contains("outputSchema"), "{message}");
}
