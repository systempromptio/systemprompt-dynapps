//! "My routines" with templates: a linked task carries its template's name,
//! and the page's delete refuses a task of a mandatory template only while
//! the person still sees that template, as the desktop's `DELETE` does.

use axum::http::StatusCode;
use dynapps_test_common::{db_or_skip, empty_services_view};
use dynwork_routines::store;

use crate::page_support::{get_page, pages, post_form};
use crate::support::{router, seed_user};
use crate::template_support::{
    activate, body, draft, drop_rules, get, linked, publish, publish_draft, rule, seed_admin,
};

#[tokio::test]
async fn a_person_who_lost_access_deletes_a_mandatory_task_from_the_page() {
    let db = db_or_skip!();
    let desktop = router(&db.pool);
    let admin = seed_admin(&db.pool, "admin").await;
    let alice = seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish_draft(
        &db.pool,
        &admin,
        draft("daily-check", &["sales"], true, body("Daily check")),
    )
    .await;
    assert_eq!(
        get(&desktop, "/templates", "alice", None).await.status,
        StatusCode::OK
    );
    let made = store::list_live_routines(&db.pool, &alice)
        .await
        .expect("list");
    assert_eq!(made.len(), 1, "the mandatory task was made");
    let delete = format!("/routines/{}/delete", made[0].id);
    let pages = pages(&db.pool, &alice, false, empty_services_view());

    let refused = post_form(&pages, &delete, &[]).await;
    assert!(
        refused.redirected("/admin/routines", "err"),
        "{:?}",
        refused.location
    );
    assert_eq!(
        store::list_live_routines(&db.pool, &alice)
            .await
            .expect("list")
            .len(),
        1,
        "kept while alice sees the template"
    );

    drop_rules(&db.pool, "sales").await;
    let deleted = post_form(&pages, &delete, &[]).await;
    assert!(
        deleted.redirected("/admin/routines", "ok"),
        "{:?}",
        deleted.location
    );
    assert!(
        store::list_live_routines(&db.pool, &alice)
            .await
            .expect("list")
            .is_empty(),
        "without access the mandatory task is hers to delete"
    );
}

#[tokio::test]
async fn a_linked_task_shows_its_template_on_my_routines() {
    let db = db_or_skip!();
    let desktop = router(&db.pool);
    let admin = seed_admin(&db.pool, "admin").await;
    let alice = seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "role", "user", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let (status, task) = activate(
        &desktop,
        "alice",
        "task-1",
        linked("My sales check", "sales-review", 1),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{task}");
    let pages = pages(&db.pool, &alice, false, empty_services_view());

    let page = get_page(&pages, "/routines").await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.text);
    assert!(page.has("My sales check"));
    assert!(page.has("Template · Sales review"));
}
