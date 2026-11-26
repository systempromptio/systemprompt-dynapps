//! The writing routes: each needs `write` on what it changes (`403` for a
//! reader, `404` for someone who cannot read it), refuses what the space does
//! not accept, and leaves nothing behind when it refuses.

use axum::http::{Method, StatusCode};
use dynbrain::access::{Level, Subject};
use dynbrain::ids::SpaceId;
use serde_json::{Value, json};

use crate::fixture::user;
use crate::http_support::{Answer, App};

async fn article_count(app: &App, space: SpaceId) -> Value {
    let listed: Answer = app.get(&format!("/spaces/{space}/articles"), "alice").await;
    listed.body["total"].clone()
}

#[tokio::test]
async fn create_makes_an_article_with_its_first_variant() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let made = app
        .post(
            "/articles",
            "alice",
            &json!({"space": space.id.0, "title": "Guía de inicio", "markdown": "Hola",
                    "metadata": {"status": "draft"}}),
        )
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{:?}", made.body);
    assert_eq!(made.body["path"], "handbook/guia-de-inicio");
    assert_eq!(made.body["variant"]["lang"], "es");
    let id = made.body["article_id"].as_i64().expect("an article id");
    let read = app.get(&format!("/articles/{id}"), "alice").await;
    assert_eq!(read.body["title"], "Guía de inicio");
    assert_eq!(read.body["markdown"], "Hola");
    assert_eq!(read.body["metadata"]["status"], "draft");
    let first = app
        .post(
            "/articles",
            "alice",
            &json!({"parent_id": id, "title": "Detalle"}),
        )
        .await;
    let second = app
        .post(
            "/articles",
            "alice",
            &json!({"parent_id": id, "title": "Detalle"}),
        )
        .await;
    assert_eq!(first.body["path"], "handbook/guia-de-inicio/detalle");
    assert_eq!(second.body["path"], "handbook/guia-de-inicio/detalle-2");
    app.finish().await;
}

#[tokio::test]
async fn create_refuses_what_the_space_does_not_accept_and_writes_nothing() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let id = space.id.0;
    for body in [
        json!({"space": id, "title": "x", "metadata": {"status": "bogus"}}),
        json!({"space": id, "title": "x", "metadata": {"nope": 1}}),
        json!({"space": id, "title": "x", "lang": "fr"}),
        json!({"space": id, "title": ""}),
        json!({"title": "x"}),
    ] {
        let refused = app.post("/articles", "alice", &body).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(refused.key(), "invalid", "{body}");
    }
    let malformed = app
        .request(
            Method::POST,
            "/articles",
            Some(&crate::http_support::desktop_token("alice")),
            Some(&json!([1, 2])),
        )
        .await;
    assert_eq!(malformed.status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed.key(), "bad_request");
    assert_eq!(article_count(&app, space.id).await, 0);
    app.finish().await;
}

#[tokio::test]
async fn create_needs_write_on_the_parent() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    app.fixture.write(&root, "es", "Guía", "texto").await;
    let body = json!({"parent_id": root.id.0, "title": "Nota"});
    assert_eq!(
        app.post("/articles", "bob", &body).await.status,
        StatusCode::NOT_FOUND
    );
    let bob = Subject::user(&user("bob"));
    app.fixture
        .give(space.id, Some(root.id), bob.clone(), Level::Read)
        .await;
    let reader = app.post("/articles", "bob", &body).await;
    assert_eq!(reader.status, StatusCode::FORBIDDEN);
    assert_eq!(reader.key(), "forbidden");
    app.fixture
        .give(space.id, Some(root.id), bob, Level::Write)
        .await;
    let writer = app.post("/articles", "bob", &body).await;
    assert_eq!(writer.status, StatusCode::CREATED, "{:?}", writer.body);
    app.finish().await;
}

#[tokio::test]
async fn a_top_level_article_needs_write_on_the_space() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let body = json!({"space": space.id.0, "title": "Portada"});
    let unseen = app.post("/articles", "carol", &body).await;
    assert_eq!(unseen.status, StatusCode::NOT_FOUND);
    app.fixture
        .give(space.id, None, Subject::user(&user("bob")), Level::Read)
        .await;
    let reader = app.post("/articles", "bob", &body).await;
    assert_eq!(reader.status, StatusCode::FORBIDDEN);
    app.fixture
        .give(space.id, None, Subject::role("editor"), Level::Write)
        .await;
    let editor = app.post("/articles", "carol", &body).await;
    assert_eq!(editor.status, StatusCode::CREATED, "{:?}", editor.body);
    let admin = app.post("/articles", "root", &body).await;
    assert_eq!(admin.status, StatusCode::CREATED, "{:?}", admin.body);
    app.finish().await;
}

#[tokio::test]
async fn create_tree_builds_nested_articles_in_order() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let made = app
        .post(
            "/articles/tree",
            "alice",
            &json!({"space": space.id.0, "nodes": [
                {"title": "Alpha", "markdown": "a", "children": [
                    {"title": "Alpha one"}, {"title": "Alpha two"}]},
                {"title": "Beta"}]}),
        )
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{:?}", made.body);
    assert_eq!(made.body["created"], 4);
    assert_eq!(
        made.body["nodes"][0]["children"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(
        made.body["nodes"][0]["children"][1]["path"],
        "handbook/alpha/alpha-two"
    );
    let outline = app
        .get(&format!("/spaces/{}/outline?depth=-1", space.id), "alice")
        .await;
    let titles: Vec<&str> = outline.body["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .filter_map(|node| node["title"].as_str())
        .collect();
    assert_eq!(titles, ["Alpha", "Alpha one", "Alpha two", "Beta"]);
    app.finish().await;
}

#[tokio::test]
async fn create_tree_refuses_a_bad_node_before_creating_anything() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let id = space.id.0;
    let too_many: Vec<Value> = (0..201)
        .map(|n| json!({"title": format!("n{n}")}))
        .collect();
    for body in [
        json!({"space": id, "nodes": [{"title": "ok"},
            {"title": "bad", "metadata": {"status": "bogus"}}]}),
        json!({"space": id, "nodes": []}),
        json!({"space": id, "nodes": too_many}),
    ] {
        let refused = app.post("/articles/tree", "alice", &body).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST);
        assert_eq!(refused.key(), "invalid");
    }
    assert_eq!(article_count(&app, space.id).await, 0);
    app.finish().await;
}

#[tokio::test]
async fn update_edits_the_variant_bumps_the_revision_and_adds_languages() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    app.fixture.write(&root, "es", "Guía", "texto").await;
    let path = format!("/articles/{}", root.id);
    let edited = app
        .patch(
            &path,
            "alice",
            &json!({"title": "Nueva guía", "markdown": "nuevo",
                    "front_matter": {"description": "resumen", "icon": "x"},
                    "metadata": {"status": "decided"}}),
        )
        .await;
    assert_eq!(edited.status, StatusCode::OK, "{:?}", edited.body);
    assert_eq!(edited.body["revision"], 2);
    let read = app.get(&path, "alice").await;
    assert_eq!(read.body["title"], "Nueva guía");
    assert_eq!(read.body["markdown"], "nuevo");
    assert_eq!(read.body["description"], "resumen");
    assert_eq!(read.body["front_matter"]["icon"], "x");
    assert_eq!(read.body["metadata"]["status"], "decided");
    app.patch(&path, "alice", &json!({"metadata": {"status": null}}))
        .await;
    let cleared = app.get(&path, "alice").await;
    assert!(cleared.body["metadata"]["status"].is_null());
    assert_eq!(cleared.body["title"], "Nueva guía");
    let english = app
        .patch(
            &path,
            "alice",
            &json!({"lang": "en", "title": "Guide", "markdown": "body"}),
        )
        .await;
    assert_eq!(english.body["variant"]["lang"], "en");
    assert_eq!(english.body["variant"]["is_default"], false);
    assert_eq!(english.body["revision"], 1);
    let served = app.get(&format!("{path}?lang=en"), "alice").await;
    assert_eq!(served.body["title"], "Guide");
    assert_eq!(app.get(&path, "alice").await.body["title"], "Nueva guía");
    app.finish().await;
}

#[tokio::test]
async fn update_needs_something_to_change_and_write_access() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    app.fixture.write(&root, "es", "Guía", "texto").await;
    let path = format!("/articles/{}", root.id);
    let empty = app.patch(&path, "alice", &json!({})).await;
    assert_eq!(empty.status, StatusCode::BAD_REQUEST);
    let change = json!({"title": "Otra"});
    assert_eq!(
        app.patch(&path, "bob", &change).await.status,
        StatusCode::NOT_FOUND
    );
    app.fixture
        .give(
            space.id,
            Some(root.id),
            Subject::user(&user("bob")),
            Level::Read,
        )
        .await;
    assert_eq!(
        app.patch(&path, "bob", &change).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(app.get(&path, "alice").await.body["title"], "Guía");
    app.finish().await;
}

#[tokio::test]
async fn move_reparents_orders_and_refuses_cycles() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let a = app.fixture.article(space.id, None, "a").await;
    let c = app.fixture.article(space.id, None, "c").await;
    let a1 = app.fixture.article(space.id, Some(a.id), "a1").await;
    app.fixture.write(&a, "es", "A", "texto").await;
    app.fixture.write(&c, "es", "C", "texto").await;
    app.fixture.write(&a1, "es", "A1", "texto").await;
    let into = json!({"new_parent_id": a.id.0});
    let moved = app
        .post(&format!("/articles/{}/move", c.id), "alice", &into)
        .await;
    assert_eq!(moved.status, StatusCode::OK, "{:?}", moved.body);
    assert_eq!(moved.body["new_parent_id"], a.id.0);
    let first = json!({"new_parent_id": a.id.0, "position": "first"});
    app.post(&format!("/articles/{}/move", c.id), "alice", &first)
        .await;
    let parent = app.get(&format!("/articles/{}", a.id), "alice").await;
    assert_eq!(parent.body["children"][0]["title"], "C");
    assert_eq!(parent.body["children"][1]["title"], "A1");
    let cycle = json!({"new_parent_id": a1.id.0});
    let refused = app
        .post(&format!("/articles/{}/move", a.id), "alice", &cycle)
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(refused.key(), "invalid");
    let top = app
        .post(&format!("/articles/{}/move", c.id), "alice", &json!({}))
        .await;
    assert_eq!(top.status, StatusCode::OK, "{:?}", top.body);
    assert!(top.body["new_parent_id"].is_null());
    app.finish().await;
}

#[tokio::test]
async fn trash_takes_the_subtree_out_and_needs_write() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    let child = app
        .fixture
        .article(space.id, Some(root.id), "install")
        .await;
    app.fixture.write(&root, "es", "Guía", "texto").await;
    app.fixture.write(&child, "es", "Instalar", "texto").await;
    app.fixture
        .give(
            space.id,
            Some(root.id),
            Subject::user(&user("bob")),
            Level::Read,
        )
        .await;
    let path = format!("/articles/{}/trash", root.id);
    let none = json!({});
    assert_eq!(
        app.post(&path, "bob", &none).await.status,
        StatusCode::FORBIDDEN
    );
    let trashed = app.post(&path, "alice", &none).await;
    assert_eq!(trashed.status, StatusCode::OK, "{:?}", trashed.body);
    assert_eq!(trashed.body["trashed"], true);
    for id in [root.id, child.id] {
        let gone = app.get(&format!("/articles/{id}"), "alice").await;
        assert_eq!(gone.status, StatusCode::NOT_FOUND);
    }
    assert_eq!(article_count(&app, space.id).await, 0);
    app.finish().await;
}

#[tokio::test]
async fn search_and_answer_are_unavailable_until_the_retrieval_phases() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let search = app.post("/search", "alice", &json!({"query": "x"})).await;
    assert_eq!(search.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(search.key(), "unavailable");
    let answer = app
        .post("/answer", "alice", &json!({"question": "x"}))
        .await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    let anonymous = app
        .request(Method::POST, "/search", None, Some(&json!({"query": "x"})))
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    app.finish().await;
}
