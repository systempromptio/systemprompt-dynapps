//! Core's SSO state binding as this installation's routes drive it: the row
//! a `start` writes is hashed at rest, expires, and is consumed once.
//!
//! The provider config lives in a process-wide first-write-wins slot, so
//! these tests depend on nextest running each test in its own process.

use dynapps_test_common::FakeOidcIdp;

use crate::fixtures::unique;
use crate::sso_harness::oidc::DOMAIN;
use crate::sso_harness::{OIDC_TENANT, SsoApp, oidc};
use crate::tempdb::TempDb;

async fn unconsumed(pool: &sqlx::PgPool, provider: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM sso_state_bindings WHERE provider_id = $1 AND consumed_at IS NULL",
    )
    .bind(provider)
    .fetch_one(pool)
    .await
    .expect("read sso_state_bindings")
}

#[tokio::test(flavor = "multi_thread")]
async fn sso_state_is_hashed_at_rest_and_consumed_once() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = FakeOidcIdp::start(OIDC_TENANT).await;
    let app = SsoApp::build(&db.pool, Some(oidc::config(&idp)));
    let started = oidc::start_default(&app).await;

    let stored: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT state_hash, nonce FROM sso_state_bindings WHERE provider_id = 'entra-oidc'",
    )
    .fetch_all(&*db.pool)
    .await
    .expect("read state rows");
    assert_eq!(stored.len(), 1, "one row per start");
    assert_ne!(stored[0].0, started.state, "the raw state is never stored");
    assert_eq!(stored[0].1.as_deref(), Some(started.nonce.as_str()));

    let spec = oidc::token_for(
        &started,
        &format!("{}@{DOMAIN}", unique("once")),
        &unique("sub"),
    );
    oidc::call_back(&app, &idp, &started, spec)
        .await
        .assert_signed_in("/admin");
    assert_eq!(
        unconsumed(&db.pool, "entra-oidc").await,
        0,
        "the row is consumed"
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn sso_state_past_its_expiry_is_not_honoured() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = FakeOidcIdp::start(OIDC_TENANT).await;
    let app = SsoApp::build(&db.pool, Some(oidc::config(&idp)));
    let started = oidc::start_default(&app).await;
    sqlx::query("UPDATE sso_state_bindings SET expires_at = now() - interval '1 second'")
        .execute(&*db.pool)
        .await
        .expect("expire the state");

    let spec = oidc::token_for(
        &started,
        &format!("{}@{DOMAIN}", unique("late")),
        &unique("sub"),
    );
    oidc::call_back(&app, &idp, &started, spec)
        .await
        .assert_refused("correlation_lost");
    assert!(
        !idp.requests().iter().any(|r| r.path == "token"),
        "an expired state never reaches the token endpoint"
    );
    db.cleanup().await;
}
