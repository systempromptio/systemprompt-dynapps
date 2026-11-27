//! The reading routes: what each shows is what the person reaches, and
//! what they do not reach is `404`.

use axum::http::StatusCode;
use dynbrain::access::{Level, Subject};
use dynbrain::brain::Brain;
use dynbrain::brain::args::{GetArgs, ListArgs, Object};
use dynbrain::brain::fault::BrainFault;
use dynbrain::brain::native::NativeBrain;
use dynbrain::model::{Article, Space};
use serde_json::json;

use crate::fixture::{content, user};
use crate::http_support::App;

async fn handbook(app: &App) -> (Space, Article, Article) {
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    app.fixture
        .write(&root, "es", "Guía", "Cuerpo de la guía")
        .await;
    let child = app
        .fixture
        .article(space.id, Some(root.id), "install")
        .await;
    app.fixture.write(&child, "es", "Instalar", "Pasos").await;
    (space, root, child)
}

#[tokio::test]
async fn spaces_lists_what_the_person_reaches() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, _, _) = handbook(&app).await;
    let alice = app.get("/spaces", "alice").await;
    assert_eq!(alice.status, StatusCode::OK);
    assert_eq!(alice.body["total"], 1);
    assert_eq!(alice.body["spaces"][0]["space"], space.id.0);
    assert_eq!(alice.body["spaces"][0]["slug"], "handbook");
    assert_eq!(alice.body["spaces"][0]["level"], "manage");
    assert_eq!(alice.body["spaces"][0]["article_count"], 2);
    assert_eq!(app.get("/spaces", "bob").await.body["total"], 0);
    app.fixture
        .give(space.id, None, Subject::user(&user("bob")), Level::Read)
        .await;
    let bob = app.get("/spaces", "bob").await;
    assert_eq!(bob.body["total"], 1);
    assert_eq!(bob.body["spaces"][0]["level"], "read");
    app.finish().await;
}

#[tokio::test]
async fn an_article_is_served_with_the_fields_of_the_docs_get_tool() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, root, child) = handbook(&app).await;
    let answer = app.get(&format!("/articles/{}", root.id), "alice").await;
    assert_eq!(answer.status, StatusCode::OK, "{:?}", answer.body);
    let body = &answer.body;
    assert_eq!(body["found"], true);
    assert_eq!(body["article_id"], root.id.0);
    assert_eq!(body["space"], space.id.0);
    assert_eq!(body["path"], "handbook/guide");
    assert_eq!(body["title"], "Guía");
    assert_eq!(body["markdown"], "Cuerpo de la guía");
    assert_eq!(body["lang"], "es");
    assert_eq!(body["variant"]["lang"], "es");
    assert_eq!(body["variant"]["is_default"], true);
    assert_eq!(body["can_write"], true);
    assert_eq!(body["revision"], 1);
    assert_eq!(body["inherit_access"], true);
    assert_eq!(body["children"][0]["article_id"], child.id.0);
    assert_eq!(body["children"][0]["title"], "Instalar");
    app.finish().await;
}

#[tokio::test]
async fn the_language_asked_for_is_served_and_a_missing_one_falls_back() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, root, _) = handbook(&app).await;
    app.fixture.write(&root, "en", "Guide", "Guide body").await;
    let path = format!("/articles/{}", root.id);
    let english = app.get(&format!("{path}?lang=en"), "alice").await;
    assert_eq!(english.body["title"], "Guide");
    assert_eq!(english.body["variant"]["is_default"], false);
    let missing = app.get(&format!("{path}?lang=fr"), "alice").await;
    assert_eq!(missing.body["title"], "Guía");
    assert_eq!(missing.body["variant"]["lang"], "es");
    app.finish().await;
}

#[tokio::test]
async fn what_the_person_does_not_reach_is_not_found() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, root, _) = handbook(&app).await;
    let hidden = app.get(&format!("/articles/{}", root.id), "bob").await;
    assert_eq!(hidden.status, StatusCode::NOT_FOUND);
    assert_eq!(hidden.key(), "not_found");
    let absent = app.get("/articles/987654", "alice").await;
    assert_eq!(absent.status, StatusCode::NOT_FOUND);
    app.finish().await;
}

#[tokio::test]
async fn the_children_listed_are_the_ones_the_person_reaches() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, root, _) = handbook(&app).await;
    let hidden = app.fixture.article(space.id, Some(root.id), "update").await;
    app.fixture
        .write(&hidden, "es", "Actualizar", "Pasos")
        .await;
    let bob = Subject::user(&user("bob"));
    app.fixture
        .give(space.id, Some(root.id), bob.clone(), Level::Read)
        .await;
    app.fixture
        .give(space.id, Some(hidden.id), bob, Level::NoAccess)
        .await;
    let path = format!("/articles/{}", root.id);
    let seen = app.get(&path, "bob").await;
    assert_eq!(seen.body["children"].as_array().map(Vec::len), Some(1));
    assert_eq!(seen.body["can_write"], false);
    let owner = app.get(&path, "alice").await;
    assert_eq!(owner.body["children"].as_array().map(Vec::len), Some(2));
    app.finish().await;
}

#[tokio::test]
async fn the_outline_is_cut_at_max_nodes_and_says_so() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, root, _) = handbook(&app).await;
    for slug in ["a", "b", "c", "d"] {
        let child = app.fixture.article(space.id, Some(root.id), slug).await;
        app.fixture.write(&child, "es", slug, "texto").await;
    }
    let path = format!("/spaces/{}/outline", space.id);
    let cut = app.get(&format!("{path}?max_nodes=3"), "alice").await;
    assert_eq!(cut.status, StatusCode::OK, "{:?}", cut.body);
    assert_eq!(cut.body["total"], 6);
    assert_eq!(cut.body["shown"], 3);
    assert_eq!(cut.body["truncated"], true);
    assert_eq!(cut.body["has_more"], true);
    assert!(cut.body["hint"].is_string());
    assert_eq!(cut.body["anchor"]["name"], "Space handbook");
    let whole = app.get(&path, "alice").await;
    assert_eq!(whole.body["total"], 6);
    assert_eq!(whole.body["truncated"], false);
    assert!(whole.body["hint"].is_null());
    app.finish().await;
}

#[tokio::test]
async fn the_outline_follows_depth_and_counts_children() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, _, child) = handbook(&app).await;
    let grand = app
        .fixture
        .article(space.id, Some(child.id), "docker")
        .await;
    app.fixture.write(&grand, "es", "Docker", "texto").await;
    let path = format!("/spaces/{}/outline", space.id);
    let shallow = app.get(&path, "alice").await;
    assert_eq!(shallow.body["total"], 2);
    assert_eq!(shallow.body["nodes"][0]["title"], "Guía");
    assert_eq!(shallow.body["nodes"][0]["depth"], 1);
    assert_eq!(shallow.body["nodes"][0]["child_count"], 1);
    assert_eq!(shallow.body["nodes"][1]["has_children"], true);
    let whole = app.get(&format!("{path}?depth=-1"), "alice").await;
    assert_eq!(whole.body["total"], 3);
    assert_eq!(whole.body["nodes"][2]["depth"], 3);
    let invalid = app.get(&format!("{path}?depth=0"), "alice").await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid.key(), "invalid");
    app.finish().await;
}

#[tokio::test]
async fn the_outline_shows_only_what_the_person_reaches() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, root, child) = handbook(&app).await;
    app.fixture
        .give(
            space.id,
            Some(child.id),
            Subject::user(&user("bob")),
            Level::Read,
        )
        .await;
    let outline = app
        .get(&format!("/spaces/{}/outline", space.id), "bob")
        .await;
    assert_eq!(outline.status, StatusCode::OK, "{:?}", outline.body);
    assert_eq!(outline.body["total"], 1);
    assert_eq!(outline.body["nodes"][0]["id"], child.id.0);
    assert!(outline.body["nodes"][0]["parent_id"].is_null());
    let node = app
        .get(
            &format!("/spaces/{}/outline?node={}", space.id, root.id),
            "bob",
        )
        .await;
    assert_eq!(node.status, StatusCode::NOT_FOUND);
    app.finish().await;
}

#[tokio::test]
async fn the_flat_list_gives_paths() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, _, _) = handbook(&app).await;
    let listed = app
        .get(&format!("/spaces/{}/articles", space.id), "alice")
        .await;
    assert_eq!(listed.status, StatusCode::OK, "{:?}", listed.body);
    assert_eq!(listed.body["total"], 2);
    assert_eq!(listed.body["documents"][0]["path"], "handbook/guide");
    assert_eq!(
        listed.body["documents"][1]["path"],
        "handbook/guide/install"
    );
    assert_eq!(listed.body["documents"][1]["title"], "Instalar");
    app.finish().await;
}

#[tokio::test]
async fn the_export_is_markdown_in_tree_order() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, _, _) = handbook(&app).await;
    let path = format!("/spaces/{}/export", space.id);
    let full = app.get(&path, "alice").await;
    assert_eq!(full.status, StatusCode::OK, "{:?}", full.body);
    assert_eq!(full.body["mimetype"], "text/markdown");
    assert_eq!(full.body["article_count"], 2);
    let text = full.body["markdown"].as_str().unwrap_or_default();
    assert!(text.starts_with("# Guía\n\nCuerpo de la guía"), "{text}");
    assert!(text.contains("## Instalar\n\nPasos"), "{text}");
    let index = app.get(&format!("{path}?format=index"), "alice").await;
    let lines = index.body["markdown"].as_str().unwrap_or_default();
    assert!(lines.contains("- Guía (`handbook/guide`)"), "{lines}");
    assert!(
        lines.contains("  - Instalar (`handbook/guide/install`)"),
        "{lines}"
    );
    let refused = app.get(&format!("{path}?format=zip"), "alice").await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(app.get(&path, "bob").await.status, StatusCode::NOT_FOUND);
    app.finish().await;
}

#[tokio::test]
async fn metadata_and_schema_show_what_the_person_may_do() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, root, _) = handbook(&app).await;
    app.fixture
        .give(
            space.id,
            Some(root.id),
            Subject::user(&user("bob")),
            Level::Read,
        )
        .await;
    let path = format!("/articles/{}/metadata", root.id);
    let owner = app.get(&path, "alice").await;
    assert_eq!(owner.status, StatusCode::OK, "{:?}", owner.body);
    let item = &owner.body["documents"][0];
    assert_eq!(item["id"], root.id.0);
    assert_eq!(item["can_write"], true);
    assert_eq!(item["is_synced"], false);
    assert_eq!(item["child_count"], 1);
    assert_eq!(item["variant_langs"][0], "es");
    assert_eq!(
        app.get(&path, "bob").await.body["documents"][0]["can_write"],
        false
    );
    let schema_path = format!("/spaces/{}/schema", space.id);
    let schema = app.get(&schema_path, "alice").await;
    assert_eq!(schema.body["fields"][0]["name"], "status");
    assert_eq!(schema.body["fields"][0]["type"], "selection");
    assert_eq!(
        app.get(&schema_path, "carol").await.status,
        StatusCode::NOT_FOUND
    );
    app.finish().await;
}

#[tokio::test]
async fn a_path_finds_an_article_and_a_property_filters_the_list() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, _, child) = handbook(&app).await;
    let mut decided = content("Instalar", "Pasos");
    decided
        .properties
        .insert("status".to_owned(), json!("decided"));
    app.fixture
        .repo
        .write_variant(child.id, "es", &decided, &user("alice"))
        .await
        .expect("write the property");
    let brain = NativeBrain::new(app.fixture.repo.clone());
    let alice = user("alice");
    let found = brain
        .get(
            &alice,
            &GetArgs {
                path: Some("handbook/guide/install".to_owned()),
                ..GetArgs::default()
            },
        )
        .await
        .expect("get by path");
    assert_eq!(found.article_id, child.id);
    let both = GetArgs {
        article_id: Some(child.id),
        path: Some("handbook/guide".to_owned()),
        ..GetArgs::default()
    };
    assert!(matches!(
        brain.get(&alice, &both).await,
        Err(BrainFault::Invalid(_))
    ));
    let missing = GetArgs {
        path: Some("handbook/nothing".to_owned()),
        ..GetArgs::default()
    };
    assert!(matches!(
        brain.get(&alice, &missing).await,
        Err(BrainFault::NotFound)
    ));
    let filter: Object = json!({"status": "decided"})
        .as_object()
        .cloned()
        .expect("an object");
    let listed = brain
        .list(
            &alice,
            &ListArgs {
                space: space.id,
                filter: Some(filter),
                ..ListArgs::default()
            },
        )
        .await
        .expect("filtered list");
    assert_eq!(listed.total, 1);
    assert_eq!(listed.documents[0].article_id, child.id);
    app.finish().await;
}
