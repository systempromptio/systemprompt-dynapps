//! Consent state and grants are isolated by principal and provider.

use crate::fixtures::{insert_user, unique, user_id};
use crate::tempdb::TempDb;
use systemprompt_web_admin::repositories::users::connector_credentials::{
    self as repo, EncryptedGrant,
};

#[tokio::test]
async fn consent_is_single_use_and_bound_to_the_user_and_provider() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("connector-alice");
    let bob = unique("connector-bob");
    insert_user(&db.pool, &alice).await;
    insert_user(&db.pool, &bob).await;
    let grant = EncryptedGrant {
        ciphertext: vec![7; 32],
        nonce: vec![1; 12],
    };
    repo::save_state(&db.pool, &user_id(&alice), "github", "test-consent", &grant)
        .await
        .unwrap();
    assert!(
        repo::consume_state(&db.pool, &user_id(&bob), "github", "test-consent")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::consume_state(&db.pool, &user_id(&alice), "atlassian", "test-consent")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::consume_state(&db.pool, &user_id(&alice), "github", "test-consent")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        repo::consume_state(&db.pool, &user_id(&alice), "github", "test-consent")
            .await
            .unwrap()
            .is_none()
    );
    let mut tx = db.pool.begin().await.unwrap();
    repo::store(&mut tx, &user_id(&alice), "github", &grant)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut tx = db.pool.begin().await.unwrap();
    assert!(
        repo::lock(&mut tx, &user_id(&bob), "github")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::lock(&mut tx, &user_id(&alice), "atlassian")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::lock(&mut tx, &user_id(&alice), "github")
            .await
            .unwrap()
            .is_some()
    );
    tx.commit().await.unwrap();
    db.cleanup().await;
}

#[tokio::test]
async fn account_revisions_survive_disconnect_and_are_isolated_per_user() {
    use systemprompt_web_admin::repositories::users::connector_accounts as accounts;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("connection-state-alice");
    let bob = unique("connection-state-bob");
    insert_user(&db.pool, &alice).await;
    insert_user(&db.pool, &bob).await;
    let user = user_id(&alice);
    let mut tx = db.pool.begin().await.unwrap();
    let mut row = accounts::get_locked_account(&mut tx, &user, "github")
        .await
        .unwrap();
    let initial = row.revision;
    row.status = "connected".into();
    row.account_id = Some("verified-github-id".into());
    accounts::update_account(&mut tx, &user, &row)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let connected = accounts::list_accounts(&db.pool, &user).await.unwrap();
    assert!(connected[0].revision > initial);
    assert!(
        accounts::list_accounts(&db.pool, &user_id(&bob))
            .await
            .unwrap()
            .is_empty()
    );
    let mut tx = db.pool.begin().await.unwrap();
    let mut row = accounts::get_locked_account(&mut tx, &user, "github")
        .await
        .unwrap();
    row.status = "not_connected".into();
    row.generation += 1;
    accounts::update_account(&mut tx, &user, &row)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let disconnected = accounts::list_accounts(&db.pool, &user).await.unwrap();
    assert_eq!(disconnected[0].generation, 1);
    assert!(disconnected[0].revision > connected[0].revision);
    db.cleanup().await;
}

#[tokio::test]
async fn refresh_and_disconnect_serialize_on_the_account_generation() {
    use std::time::Duration;
    use systemprompt_web_admin::repositories::users::connector_accounts as accounts;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("connection-lock-alice");
    insert_user(&db.pool, &alice).await;
    let user = user_id(&alice);
    let mut first = db.pool.begin().await.unwrap();
    let mut row = accounts::get_locked_account(&mut first, &user, "atlassian")
        .await
        .unwrap();
    row.generation += 1;
    accounts::update_account(&mut first, &user, &row)
        .await
        .unwrap();
    let mut second = db.pool.begin().await.unwrap();
    assert!(
        tokio::time::timeout(
            Duration::from_millis(50),
            accounts::get_locked_account(&mut second, &user, "atlassian")
        )
        .await
        .is_err()
    );
    first.commit().await.unwrap();
    // Cancellation may abort the waiting PostgreSQL statement; start a fresh
    // transaction to prove the committed generation is what a callback sees.
    second.rollback().await.unwrap();
    let mut second = db.pool.begin().await.unwrap();
    let observed = accounts::get_locked_account(&mut second, &user, "atlassian")
        .await
        .unwrap();
    assert_eq!(observed.generation, 1);
    second.commit().await.unwrap();
    db.cleanup().await;
}
