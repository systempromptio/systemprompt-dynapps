//! The Access tab and the form that changes it.

use axum::http::StatusCode;

use crate::env::Env;

#[tokio::test]
async fn the_access_tab_lists_what_is_set_and_what_the_article_ends_up_with() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let page = env
        .get(&format!("/brain/articles/{id}?lang=es&tab=access"), true)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has("Access this article ends up with"));
    assert!(page.has("bob"), "bob reads the whole space");
    assert!(page.has("Can read"));
    assert!(page.has("Set on this article"));
    env.finish().await;
}

#[tokio::test]
async fn the_form_gives_a_person_access_and_audits_it() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let saved = env
        .post(
            &format!("/brain/articles/{id}/access"),
            "kind=user&subject=carol&level=write&lang=es",
            true,
        )
        .await;
    assert_eq!(saved.status, StatusCode::SEE_OTHER);
    let page = env
        .get(&format!("/brain/articles/{id}?lang=es&tab=access"), true)
        .await;
    assert!(page.has("carol") && page.has("Can edit"));
    let audit = env
        .sql_text("SELECT action || ':' || subject_id FROM dynbrain_admin_audit")
        .await;
    assert_eq!(audit, "set_access:carol");
    env.finish().await;
}

#[tokio::test]
async fn the_form_removes_an_own_row() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let url = format!("/brain/articles/{id}/access");
    env.post(&url, "kind=user&subject=carol&level=read&lang=es", true)
        .await;
    env.post(&url, "kind=user&subject=carol&level=remove&lang=es", true)
        .await;
    let page = env
        .get(&format!("/brain/articles/{id}?lang=es&tab=access"), true)
        .await;
    assert!(!page.has("carol"), "the row is gone");
    env.finish().await;
}

#[tokio::test]
async fn the_form_refuses_a_kind_it_does_not_know() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let id = env.invoicing.id.0;
    let page = env
        .post(
            &format!("/brain/articles/{id}/access"),
            "kind=robot&subject=carol&level=read&lang=es",
            true,
        )
        .await;
    assert_eq!(page.status, StatusCode::BAD_REQUEST);
    env.finish().await;
}
