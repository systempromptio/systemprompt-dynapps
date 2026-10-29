//! The `documents` MCP server: it advertises exactly core's sixteen document
//! tools under the `documents` server name, and its extension manifest and
//! `services/mcp/documents.yaml` agree on the binary and the port, and that the
//! server is declared with user-scoped OAuth and off by default.

use std::path::PathBuf;

use serde_yaml::Value;
use systemprompt::runtime::documents::mcp::TOOL_NAMES;
use systemprompt_mcp_documents::{SERVER_NAME, tool_names};

const EXPECTED: [&str; 16] = [
    "documents_list",
    "documents_search",
    "documents_create",
    "documents_read",
    "documents_edit",
    "documents_comment",
    "documents_history",
    "documents_proposals",
    "documents_share",
    "documents_access",
    "documents_copy",
    "documents_update",
    "documents_trash",
    "documents_untrash",
    "documents_export",
    "documents_upload",
    "documents_guide",
    "documents_people",
];

fn repo_file(rel: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(rel);
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn the_server_lists_exactly_cores_document_tools() {
    assert_eq!(tool_names(), TOOL_NAMES);
    assert_eq!(tool_names(), EXPECTED);
    assert_eq!(SERVER_NAME, "documents");
}

#[test]
fn manifest_and_services_yaml_agree_and_the_server_is_off_by_default() {
    let manifest = &repo_file("extensions/mcp/documents/manifest.yaml")["extension"];
    let server = &repo_file("services/mcp/documents.yaml")["mcp_servers"][SERVER_NAME];

    assert_eq!(manifest["name"].as_str(), Some(SERVER_NAME));
    assert_eq!(
        manifest["binary"].as_str(),
        Some("systemprompt-mcp-documents")
    );
    assert_eq!(server["binary"], manifest["binary"]);
    assert_eq!(server["port"], manifest["port"]);
    assert_eq!(server["port"].as_u64(), Some(5012));
    assert_eq!(server["enabled"].as_bool(), Some(false));
    assert_eq!(manifest["enabled"].as_bool(), Some(true));
    assert_eq!(server["server_type"].as_str(), Some("internal"));
    assert_eq!(server["oauth"]["required"].as_bool(), Some(true));
    assert_eq!(server["oauth"]["audience"].as_str(), Some("mcp"));
    assert_eq!(server["oauth"]["scopes"][0].as_str(), Some("user"));
}
