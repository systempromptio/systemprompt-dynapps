use dynbrain::access::Level;
use rmcp::ErrorData;
use serde_json::json;

use crate::support::{Brain, anonymous};

fn not_found() -> ErrorData {
    ErrorData::resource_not_found("not found", None)
}

#[tokio::test]
async fn search_answers_as_the_person_and_hides_what_the_grant_does_not_allow() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    brain
        .article(space.id, None, "leave", "Vacaciones", "El destino zanzibar es el favorito.")
        .await;

    let own = brain
        .call("alice", "search", json!({"query": "zanzibar"}))
        .await
        .expect("alice searches");
    assert_eq!(own["total"], 1, "{own}");
    assert_eq!(own["results"][0]["title"], "Vacaciones");
    assert_eq!(own["mode"], "keyword");

    let stranger = brain
        .call("bob", "search", json!({"query": "zanzibar"}))
        .await
        .expect("bob searches");
    assert_eq!(stranger["total"], 0, "{stranger}");
    assert_eq!(stranger["results"], json!([]));

    brain.give(space.id, None, "bob", Level::Read).await;
    let granted = brain
        .call("bob", "search", json!({"query": "zanzibar"}))
        .await
        .expect("bob searches again");
    assert_eq!(granted["total"], 1, "{granted}");

    let answer = brain
        .call("bob", "answer", json!({"question": "zanzibar"}))
        .await
        .expect("bob asks");
    assert_eq!(answer["mode"], "context");
    assert_eq!(answer["citations"].as_array().map(Vec::len), Some(1), "{answer}");
    let other = brain
        .call("carol", "answer", json!({"question": "zanzibar"}))
        .await
        .expect("carol asks");
    assert_eq!(other["citations"], json!([]), "{other}");
    brain.finish().await;
}

#[tokio::test]
async fn get_reads_what_the_grant_allows_and_says_nothing_about_what_it_does_not() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    let article = brain
        .article(space.id, None, "leave", "Vacaciones", "Texto del artículo.")
        .await;

    let hidden = brain
        .call("bob", "get", json!({"article_id": article.id.0}))
        .await
        .expect_err("bob has no grant");
    let missing = brain
        .call("bob", "get", json!({"article_id": 999_999}))
        .await
        .expect_err("no such article");
    assert_eq!(hidden.code, not_found().code);
    assert_eq!(hidden.message, missing.message, "an article that exists answers as one that does not");

    brain.give(space.id, None, "bob", Level::Read).await;
    let read = brain
        .call("bob", "get", json!({"article_id": article.id.0}))
        .await
        .expect("bob reads");
    assert_eq!(read["markdown"], "Texto del artículo.");
    assert_eq!(read["can_write"], false);
    assert_eq!(read["title"], "Vacaciones");

    let by_path = brain
        .call("bob", "get", json!({"path": read["path"], "lang": "es"}))
        .await
        .expect("bob reads by path");
    assert_eq!(by_path["article_id"], read["article_id"]);
    brain.finish().await;
}

#[tokio::test]
async fn list_spaces_and_list_show_each_person_only_what_they_reach() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    brain.article(space.id, None, "leave", "Vacaciones", "texto").await;

    let alice = brain.call("alice", "list_spaces", json!({})).await.expect("alice");
    assert_eq!(alice["total"], 1, "{alice}");
    let bob = brain.call("bob", "list_spaces", json!({})).await.expect("bob");
    assert_eq!(bob["total"], 0, "{bob}");
    let denied = brain
        .call("bob", "list", json!({"space": space.id.0}))
        .await;
    assert!(denied.is_err() || denied.as_ref().is_ok_and(|list| list["total"] == 0), "{denied:?}");

    brain.give(space.id, None, "bob", Level::Read).await;
    let listed = brain
        .call("bob", "list", json!({"space": space.id.0}))
        .await
        .expect("bob lists");
    assert_eq!(listed["total"], 1, "{listed}");
    brain.finish().await;
}

#[tokio::test]
async fn a_call_without_a_person_is_refused_before_any_operation_runs() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    let article = brain.article(space.id, None, "leave", "Vacaciones", "texto").await;

    for (tool, arguments) in [
        ("search", json!({"query": "texto"})),
        ("get", json!({"article_id": article.id.0})),
        ("trash", json!({"article_id": article.id.0})),
    ] {
        let refused = systemprompt_mcp_brain::tools::run(&brain.native, &anonymous(), tool, arguments)
            .await
            .expect_err("no person");
        assert!(refused.message.starts_with("Forbidden"), "{tool}: {refused:?}");
    }
    let still_there = brain
        .call("alice", "get", json!({"article_id": article.id.0}))
        .await
        .expect("nothing was trashed");
    assert_eq!(still_there["found"], true);
    brain.finish().await;
}

#[tokio::test]
async fn an_unknown_tool_and_bad_arguments_are_refused_and_an_undeclared_filter_is_empty() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    brain.article(space.id, None, "leave", "Vacaciones", "zanzibar").await;
    let invalid = ErrorData::invalid_params("x", None).code;

    let unknown = brain.call("alice", "delete", json!({})).await.expect_err("unknown");
    assert_eq!(unknown.code, invalid);
    let wrong = brain
        .call("alice", "search", json!({"query": 3}))
        .await
        .expect_err("a number is no query");
    assert_eq!(wrong.code, invalid);

    let filtered = brain
        .call("alice", "search", json!({"query": "zanzibar", "where": {"owner": "x"}}))
        .await
        .expect("an empty reply");
    assert_eq!(filtered["results"], json!([]));
    assert_eq!(filtered["total"], 0);
    assert!(filtered["error"].as_str().is_some_and(|error| error.contains("'owner' is not a metadata field")), "{filtered}");
    let listed = brain
        .call("alice", "list", json!({"space": space.id.0, "where": {"owner": "x"}}))
        .await
        .expect("an empty reply");
    assert_eq!(listed["documents"], json!([]));
    assert!(listed["error"].is_string(), "{listed}");

    let declared = brain
        .call("alice", "search", json!({"query": "zanzibar", "where": {"status": "draft"}}))
        .await
        .expect("a declared property");
    assert!(declared.get("error").is_none(), "{declared}");
    brain.finish().await;
}
