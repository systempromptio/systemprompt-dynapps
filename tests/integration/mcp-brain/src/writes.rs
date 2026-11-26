use dynbrain::access::Level;
use rmcp::ErrorData;
use serde_json::json;

use crate::support::Brain;

#[tokio::test]
async fn writing_needs_write_access_and_a_reader_is_told_so() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    let article = brain
        .article(space.id, None, "leave", "Vacaciones", "texto")
        .await;
    brain.give(space.id, None, "bob", Level::Read).await;

    let refused = brain
        .call(
            "bob",
            "update",
            json!({"article_id": article.id.0, "markdown": "cambiado"}),
        )
        .await
        .expect_err("a reader cannot write");
    assert!(refused.message.starts_with("Forbidden"), "{refused:?}");
    let create = brain
        .call(
            "bob",
            "create",
            json!({"parent_id": article.id.0, "title": "Hija", "markdown": "x"}),
        )
        .await
        .expect_err("a reader cannot create");
    assert!(create.message.starts_with("Forbidden"), "{create:?}");

    brain.give(space.id, None, "bob", Level::Write).await;
    let updated = brain
        .call(
            "bob",
            "update",
            json!({"article_id": article.id.0, "markdown": "cambiado"}),
        )
        .await
        .expect("a writer updates");
    assert_eq!(updated["updated"], true, "{updated}");
    assert_eq!(updated["revision"], 2);
    let read = brain
        .call("alice", "get", json!({"article_id": article.id.0}))
        .await
        .expect("alice reads");
    assert_eq!(read["markdown"], "cambiado");
    brain.finish().await;
}

#[tokio::test]
async fn trash_is_soft_and_takes_the_article_out_of_sight() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    let article = brain
        .article(space.id, None, "leave", "Vacaciones", "El destino zanzibar.")
        .await;
    brain.give(space.id, None, "bob", Level::Read).await;

    let denied = brain
        .call("bob", "trash", json!({"article_id": article.id.0}))
        .await
        .expect_err("a reader cannot trash");
    assert!(denied.message.starts_with("Forbidden"), "{denied:?}");

    let trashed = brain
        .call("alice", "trash", json!({"article_id": article.id.0}))
        .await
        .expect("the owner trashes");
    assert_eq!(trashed["trashed"], true, "{trashed}");
    let gone = brain
        .call("alice", "get", json!({"article_id": article.id.0}))
        .await
        .expect_err("a trashed article is not served");
    assert_eq!(gone.code, ErrorData::resource_not_found("x", None).code);
    let found = brain
        .call("alice", "search", json!({"query": "zanzibar"}))
        .await
        .expect("a search");
    assert_eq!(found["total"], 0, "{found}");
    brain.finish().await;
}

#[tokio::test]
async fn a_tree_is_created_and_outlined_as_the_person() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;

    let made = brain
        .call(
            "alice",
            "create_tree",
            json!({
                "space": space.id.0,
                "lang": "es",
                "nodes": [{
                    "title": "Guia",
                    "markdown": "raiz",
                    "children": [{"title": "Capitulo uno", "markdown": "uno"}, {"title": "Capitulo dos", "markdown": "dos"}]
                }]
            }),
        )
        .await
        .expect("alice builds a tree");
    assert_eq!(made["created"], 3, "{made}");
    let root = made["nodes"][0]["article_id"].clone();

    let outline = brain
        .call("alice", "outline", json!({"space": space.id.0, "depth": 3}))
        .await
        .expect("alice outlines");
    assert_eq!(outline["total"], 3, "{outline}");
    let none = brain
        .call("bob", "outline", json!({"node": root}))
        .await;
    assert!(none.is_err(), "bob reaches nothing of it: {none:?}");

    let schema = brain
        .call("alice", "metadata_schema", json!({"space": space.id.0}))
        .await
        .expect("the schema");
    assert_eq!(schema["fields"][0]["name"], "status", "{schema}");
    brain.finish().await;
}

#[tokio::test]
async fn move_reorganises_and_metadata_says_who_can_write() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    let first = brain.article(space.id, None, "one", "Uno", "uno").await;
    let second = brain.article(space.id, None, "two", "Dos", "dos").await;

    let moved = brain
        .call(
            "alice",
            "move",
            json!({"article_id": second.id.0, "new_parent_id": first.id.0}),
        )
        .await
        .expect("alice moves");
    assert_eq!(moved["moved"], true, "{moved}");
    let meta = brain
        .call("alice", "metadata", json!({"article_ids": [first.id.0, second.id.0]}))
        .await
        .expect("metadata");
    let documents = meta["documents"].as_array().expect("documents");
    assert_eq!(documents.len(), 2);
    assert!(documents.iter().all(|item| item["can_write"] == true), "{meta}");
    brain.finish().await;
}
