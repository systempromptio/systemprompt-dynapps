//! The knowledge-bank server's tool contract, its stub backend, and the
//! REQ-052 principal seam.
//!
//! Three tools mirror the Node project-context server (search / list_sources
//! / index_stats), with `query` as search's only required argument. The
//! `StubRetriever` — still the backend on an instance with no Bedrock key —
//! refuses to answer (`BACKEND_NOT_CONFIGURED`) rather than fabricate
//! results, and must keep doing so under the principal-carrying signature.
//!
//! [`Principal::subjects`] is pinned here because it is the whole input to
//! the grant join: a principal that produces no subjects can match no grant,
//! which is what makes an unrecognised caller see nothing rather than
//! everything.

use systemprompt::identifiers::UserId;
use systemprompt::traits::ExtensionError;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::tools::{
    INDEX_STATS_TOOL, LIST_SOURCES_TOOL, SEARCH_TOOL, SERVER_NAME, list_tools, search_input_schema,
};
use systemprompt_mcp_knowledge_bank::{KnowledgeRetriever, Principal, SearchFilter, StubRetriever};

fn principal() -> Principal {
    Principal {
        user_id: UserId::new("u-1"),
        roles: vec!["admin".to_owned()],
        attributes: [("department".to_owned(), "engineering".to_owned())]
            .into_iter()
            .collect(),
    }
}

#[test]
fn three_tools_are_exposed_with_schemas_and_ui_meta() {
    let tools = list_tools();
    assert_eq!(tools.len(), 3);
    assert_eq!(SERVER_NAME, "knowledge-bank");

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(
        names,
        vec![SEARCH_TOOL, LIST_SOURCES_TOOL, INDEX_STATS_TOOL]
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

    let source_types = &schema["properties"]["source_types"];
    assert!(source_types.is_object());

    // Why: schemars nests Option<u8> bounds differently across versions
    // (inline vs anyOf), so assert on the rendered property rather than a
    // hard-coded path.
    let top_k = serde_json::to_string(&schema["properties"]["top_k"]).expect("top_k serializes");
    assert!(top_k.contains("\"minimum\":1"));
    assert!(top_k.contains("\"maximum\":20"));
}

#[tokio::test]
async fn stub_search_reports_not_configured_instead_of_fake_data() {
    let filter = SearchFilter {
        source_types: vec!["confluence".to_owned()],
        top_k: Some(5),
    };
    let err = StubRetriever
        .search(&principal(), "deployment runbook", &filter)
        .await
        .expect_err("the stub must not return results");
    assert!(matches!(err, KnowledgeBankError::NotConfigured(_)));
    assert_eq!(err.code(), "BACKEND_NOT_CONFIGURED");
}

#[tokio::test]
async fn stub_lists_no_sources_and_has_no_index() {
    let sources = StubRetriever
        .list_sources(&principal())
        .await
        .expect("listing works");
    assert!(sources.is_empty());

    let err = StubRetriever
        .index_stats()
        .await
        .expect_err("the stub has no index to report on");
    assert!(matches!(err, KnowledgeBankError::NotConfigured(_)));
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
    assert!(pairs.contains(&("department", "engineering")));
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
    let subjects = with_noise.subjects();
    let kinds: Vec<&str> = subjects.iter().map(|s| s.kind.as_str()).collect();

    assert_eq!(kinds, vec!["user"]);
}
