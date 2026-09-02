//! The knowledge-bank server's tool contract, the boot-failure message, and
//! the REQ-052 principal seam.
//!
//! Three retrieval tools mirror the Node project-context server (search /
//! list_sources / index_stats); `sync_knowledge` is every user's own sync,
//! `add_knowledge_document` drops a hand-authored file into a files category
//! (FR-ING-6), and `prune_knowledge` is the administrator's broom (FR-ACC-1).
//! There is no stub backend any more: a database without pgvector fails the
//! boot with a message that names the database and nothing secret about it.
//!
//! [`Principal::subjects`] is pinned here because it is the whole input to
//! the grant join: a principal that produces no subjects can match no grant,
//! which is what makes an unrecognised caller see nothing rather than
//! everything.

use systemprompt::identifiers::UserId;
use systemprompt::traits::ExtensionError;
use systemprompt_mcp_knowledge_bank::Principal;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::schema::{describe_database_url, pgvector_missing};
use systemprompt_mcp_knowledge_bank::tools::{
    ADD_DOCUMENT_TOOL, INDEX_STATS_TOOL, LIST_SOURCES_TOOL, PRUNE_TOOL, SEARCH_TOOL, SERVER_NAME,
    SYNC_TOOL, list_tools, prune_input_schema, search_input_schema, sync_input_schema,
};

fn principal() -> Principal {
    Principal {
        user_id: UserId::new("u-1"),
        roles: vec!["admin".to_owned()],
        attributes: [("adfs_group".to_owned(), "Systemprompt-Nava".to_owned())]
            .into_iter()
            .collect(),
    }
}

#[test]
fn six_tools_are_exposed_with_schemas_and_ui_meta() {
    let tools = list_tools();
    assert_eq!(SERVER_NAME, "knowledge-bank");

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(
        names,
        vec![
            SEARCH_TOOL,
            LIST_SOURCES_TOOL,
            INDEX_STATS_TOOL,
            SYNC_TOOL,
            ADD_DOCUMENT_TOOL,
            PRUNE_TOOL
        ]
    );

    for tool in &tools {
        assert!(tool.title.is_some());
        assert!(tool.description.is_some());
        assert!(!tool.input_schema.is_empty());
        let output = tool.output_schema.as_ref().expect("output schema is set");
        assert!(!output.is_empty());
        assert!(
            tool.meta.is_some(),
            "UI meta is what attributes the call to this server"
        );
    }
}

#[test]
fn search_schema_requires_query_with_optional_source_types_and_bounded_top_k() {
    let schema = search_input_schema();
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(required, vec!["query"]);
    assert!(schema["properties"]["source_types"].is_object());

    // Why: schemars nests Option<u8> bounds differently across versions
    // (inline vs anyOf), so assert on the rendered property rather than a
    // hard-coded path.
    let top_k = serde_json::to_string(&schema["properties"]["top_k"]).expect("top_k serializes");
    assert!(top_k.contains("\"minimum\":1"));
    assert!(top_k.contains("\"maximum\":20"));
}

#[test]
fn sync_and_prune_schemas_have_no_required_arguments() {
    let schema = sync_input_schema();
    assert!(
        schema
            .get("required")
            .is_none_or(|r| r.as_array().is_some_and(Vec::is_empty)),
        "a user can start their full incremental sync with an empty object"
    );
    assert!(schema["properties"]["rebuild"].is_object());
    assert!(schema["properties"]["source_types"].is_object());

    let schema = prune_input_schema();
    assert!(
        schema
            .get("required")
            .is_none_or(|r| r.as_array().is_some_and(Vec::is_empty))
    );
    assert!(schema["properties"]["dry_run"].is_object());
    let stale = serde_json::to_string(&schema["properties"]["stale_days"]).expect("serializes");
    assert!(stale.contains("\"minimum\":1"));
}

#[test]
fn the_pgvector_boot_failure_names_the_database_without_its_password() {
    let described = describe_database_url(
        "postgres://knowledge:s3cret@sp-a2f658d8bc5f-knowledge.flycast:5432/knowledge?sslmode=disable",
    );
    assert_eq!(
        described,
        "sp-a2f658d8bc5f-knowledge.flycast:5432/knowledge"
    );
    let message = pgvector_missing(&described);
    assert!(message.starts_with("knowledge-bank: database sp-a2f658d8bc5f-knowledge.flycast"));
    assert!(message.contains("has no pgvector; it is required"));
    assert!(message.contains("docs/install/knowledge-db.md"));
    assert!(!message.contains("s3cret"));
    let err = KnowledgeBankError::NotConfigured(message);
    assert_eq!(err.code(), "BACKEND_NOT_CONFIGURED");
}

#[test]
fn principal_subjects_cover_identity_roles_and_attribute_bands() {
    let subjects = principal().subjects();
    let pairs: Vec<(&str, &str)> = subjects
        .iter()
        .map(|s| (s.kind.as_str(), s.value.as_str()))
        .collect();

    assert!(pairs.contains(&("user", "u-1")));
    assert!(pairs.contains(&("role", "admin")));
    assert!(pairs.contains(&("adfs_group", "Systemprompt-Nava")));
}

#[test]
fn an_unrecognised_caller_matches_only_its_own_user_subject() {
    let anonymous = Principal {
        user_id: UserId::new("u-unknown"),
        roles: Vec::new(),
        attributes: std::collections::BTreeMap::new(),
    };
    let subjects = anonymous.subjects();

    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].kind, "user");
    assert_eq!(subjects[0].value, "u-unknown");
}

#[test]
fn unknown_attributes_are_not_turned_into_subjects() {
    let with_noise = Principal {
        user_id: UserId::new("u-2"),
        roles: Vec::new(),
        attributes: [("favourite_colour".to_owned(), "green".to_owned())]
            .into_iter()
            .collect(),
    };
    let kinds: Vec<String> = with_noise.subjects().into_iter().map(|s| s.kind).collect();
    assert_eq!(kinds, vec!["user"]);
}

#[test]
fn forbidden_and_config_errors_carry_their_own_codes() {
    assert_eq!(
        KnowledgeBankError::Forbidden("x".to_owned()).code(),
        "FORBIDDEN"
    );
    assert_eq!(
        KnowledgeBankError::Config("x".to_owned()).code(),
        "INVALID_CONFIG"
    );
    assert_eq!(
        KnowledgeBankError::Forbidden("x".to_owned()).status(),
        axum::http::StatusCode::FORBIDDEN
    );
    assert_eq!(
        KnowledgeBankError::MissingHeader("x-atlassian-email").code(),
        "MISSING_HEADER"
    );
}
