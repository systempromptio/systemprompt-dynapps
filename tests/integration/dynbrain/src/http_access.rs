//! Grants and inheritance over HTTP: they need `manage`; `none` is written and
//! listed as a grant, never as access; spaces are listed for the people their
//! grants reach, whether through them, their roles or their groups.

use axum::http::StatusCode;
use dynbrain::access::{Level, Subject};
use serde_json::{Value, json};

use crate::fixture::user;
use crate::http_support::App;

fn grant(kind: &str, id: &str, level: &str) -> Value {
    json!({"kind": kind, "id": id, "level": level})
}

#[tokio::test]
async fn a_manager_reads_and_replaces_the_grants_of_a_space() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let path = format!("/spaces/{}/grants", space.id);
    let body = json!({"grants": [
        grant("user", "bob", "read"),
        grant("group", "team", "write"),
        grant("role", "editor", "none")]});
    let written = app.put(&path, "alice", &body).await;
    assert_eq!(written.status, StatusCode::OK, "{:?}", written.body);
    let listed = app.get(&path, "alice").await;
    let grants = listed.body["grants"].as_array().expect("grants");
    assert_eq!(grants.len(), 3);
    assert!(grants.contains(&grant("role", "editor", "none")));
    assert!(grants.contains(&grant("group", "team", "write")));
    let replaced = app
        .put(
            &path,
            "alice",
            &json!({"grants": [grant("user", "bob", "read")]}),
        )
        .await;
    assert_eq!(replaced.body["grants"].as_array().map(Vec::len), Some(1));
    assert_eq!(replaced.body["access"].as_array().map(Vec::len), Some(0));
    assert!(replaced.body["inherit_access"].is_null());
    app.finish().await;
}

#[tokio::test]
async fn grants_and_inheritance_need_manage() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    app.fixture.write(&root, "es", "Guía", "texto").await;
    app.fixture
        .give(
            space.id,
            Some(root.id),
            Subject::user(&user("bob")),
            Level::Write,
        )
        .await;
    let article = format!("/articles/{}/grants", root.id);
    let space_path = format!("/spaces/{}/grants", space.id);
    let body = json!({"grants": []});
    assert_eq!(app.get(&article, "bob").await.status, StatusCode::FORBIDDEN);
    assert_eq!(
        app.put(&article, "bob", &body).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.get(&space_path, "bob").await.status,
        StatusCode::FORBIDDEN
    );
    let inherit = format!("/articles/{}/inherit", root.id);
    let cut = json!({"inherit": false});
    let refused = app.put(&inherit, "bob", &cut).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(refused.key(), "forbidden");
    assert_eq!(
        app.get(&article, "carol").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.put(&space_path, "carol", &body).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.put(&inherit, "carol", &cut).await.status,
        StatusCode::NOT_FOUND
    );
    app.finish().await;
}

#[tokio::test]
async fn none_is_listed_as_a_grant_and_never_as_access() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    let other = app.fixture.article(space.id, None, "other").await;
    app.fixture.write(&root, "es", "Guía", "texto").await;
    app.fixture.write(&other, "es", "Otro", "texto").await;
    app.fixture
        .give(space.id, None, Subject::user(&user("bob")), Level::Read)
        .await;
    let path = format!("/articles/{}/grants", root.id);
    let written = app
        .put(
            &path,
            "alice",
            &json!({"grants": [grant("user", "bob", "none")]}),
        )
        .await;
    assert_eq!(written.status, StatusCode::OK, "{:?}", written.body);
    let grants = written.body["grants"].as_array().expect("grants");
    assert!(grants.contains(&grant("user", "bob", "none")));
    let access = written.body["access"].as_array().expect("access");
    assert!(access.iter().all(|entry| entry["id"] != "bob"));
    assert!(access.contains(&grant("user", "alice", "manage")));
    assert_eq!(written.body["inherit_access"], true);
    let hidden = app.get(&format!("/articles/{}", root.id), "bob").await;
    assert_eq!(hidden.status, StatusCode::NOT_FOUND);
    let shown = app.get(&format!("/articles/{}", other.id), "bob").await;
    assert_eq!(shown.status, StatusCode::OK);
    app.finish().await;
}

#[tokio::test]
async fn the_inherit_route_cuts_and_copies() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    let copied = app.fixture.article(space.id, Some(root.id), "copied").await;
    let cut = app.fixture.article(space.id, Some(root.id), "cut").await;
    for article in [&root, &copied, &cut] {
        app.fixture.write(article, "es", "Título", "texto").await;
    }
    app.fixture
        .give(
            space.id,
            Some(root.id),
            Subject::user(&user("bob")),
            Level::Read,
        )
        .await;
    let read = |id| format!("/articles/{id}");
    assert_eq!(
        app.get(&read(copied.id), "bob").await.status,
        StatusCode::OK
    );
    let plain = app
        .put(
            &format!("/articles/{}/inherit", cut.id),
            "alice",
            &json!({"inherit": false}),
        )
        .await;
    assert_eq!(plain.status, StatusCode::OK, "{:?}", plain.body);
    assert_eq!(plain.body["inherit_access"], false);
    assert_eq!(
        app.get(&read(cut.id), "bob").await.status,
        StatusCode::NOT_FOUND
    );
    let copy = json!({"inherit": false, "copy_inherited": true});
    let kept = app
        .put(&format!("/articles/{}/inherit", copied.id), "alice", &copy)
        .await;
    assert_eq!(kept.body["article_id"], copied.id.0);
    let grants = app
        .get(&format!("/articles/{}/grants", copied.id), "alice")
        .await;
    assert!(
        grants.body["grants"]
            .as_array()
            .expect("grants")
            .contains(&grant("user", "bob", "read"))
    );
    assert_eq!(grants.body["inherit_access"], false);
    let cleared = json!({"grants": []});
    app.put(&format!("/articles/{}/grants", root.id), "alice", &cleared)
        .await;
    assert_eq!(
        app.get(&read(root.id), "bob").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.get(&read(copied.id), "bob").await.status,
        StatusCode::OK
    );
    app.finish().await;
}

#[tokio::test]
async fn administrators_and_owners_manage_a_space_with_no_articles() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("empty").await;
    let path = format!("/spaces/{}/grants", space.id);
    for who in ["alice", "root"] {
        let answer = app.get(&path, who).await;
        assert_eq!(answer.status, StatusCode::OK, "{who}: {:?}", answer.body);
        assert_eq!(answer.body["grants"].as_array().map(Vec::len), Some(0));
    }
    assert_eq!(app.get(&path, "bob").await.status, StatusCode::NOT_FOUND);
    let listed = app.get("/spaces", "root").await;
    assert_eq!(listed.body["spaces"][0]["slug"], "empty");
    assert_eq!(listed.body["spaces"][0]["level"], "manage");
    assert_eq!(app.get("/spaces", "erin").await.body["total"], 0);
    app.finish().await;
}

#[tokio::test]
async fn a_group_grant_lists_the_space_for_its_live_members_only() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let body = json!({"grants": [grant("group", "team", "read")]});
    let written = app
        .put(&format!("/spaces/{}/grants", space.id), "alice", &body)
        .await;
    assert_eq!(written.status, StatusCode::OK, "{:?}", written.body);
    assert_eq!(app.get("/spaces", "erin").await.body["total"], 1);
    assert_eq!(app.get("/spaces", "frank").await.body["total"], 0);
    app.finish().await;
}

#[tokio::test]
async fn a_suspended_account_reaches_nothing_even_with_a_valid_token() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let root = app.fixture.article(space.id, None, "guide").await;
    app.fixture.write(&root, "es", "Guía", "texto").await;
    let dave = Subject::user(&user("dave"));
    app.fixture
        .give(space.id, None, dave.clone(), Level::Read)
        .await;
    app.fixture
        .give(space.id, Some(root.id), dave, Level::Read)
        .await;
    assert_eq!(app.get("/spaces", "dave").await.body["total"], 0);
    let article = app.get(&format!("/articles/{}", root.id), "dave").await;
    assert_eq!(article.status, StatusCode::NOT_FOUND);
    app.finish().await;
}

#[tokio::test]
async fn a_replacement_with_one_bad_grant_changes_nothing() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let path = format!("/spaces/{}/grants", space.id);
    let first = json!({"grants": [grant("group", "all", "read")]});
    assert_eq!(app.put(&path, "alice", &first).await.status, StatusCode::OK);
    let bad = json!({"grants": [grant("user", "bob", "read"), grant("user", "", "read")]});
    let refused = app.put(&path, "alice", &bad).await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{:?}",
        refused.body
    );
    let listed = app.get(&path, "alice").await;
    let grants = listed.body["grants"].as_array().expect("grants");
    assert!(
        grants.contains(&grant("group", "all", "read")),
        "{grants:?}"
    );
    assert!(
        !grants.contains(&grant("user", "bob", "read")),
        "{grants:?}"
    );
    app.finish().await;
}
