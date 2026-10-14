//! Entra ID SAML sign-in for someone who already has a local account but no
//! federated link: core's driver links by verified email when the policy's
//! `link_by_email` allows it and refuses otherwise, and a hand-granted role
//! survives the directory's role write (the `RoleResolver` re-joins
//! `user_manual_roles`), both in `users.roles` and in the minted token.
//!
//! One provider config per process (first-write-wins slot): nextest's
//! process-per-test isolation is required.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use dynapps_test_common::SamlIdpFixture;
use systemprompt_web_admin_identity::repositories::users::roles::set_manual_roles;

use crate::fixtures::{insert_user, unique};
use crate::sso_harness::saml::{self, DOMAIN};
use crate::sso_harness::{SsoApp, federated_user, login_events};
use crate::tempdb::TempDb;

const MANUAL_ROLE: &str = "admin";

// An existing local account with a verified `@sso.test` email, and an app
// whose policy never provisions and links by email when `link_by_email`.
async fn existing_account(db: &TempDb, link_by_email: bool) -> (SsoApp, SamlIdpFixture, String) {
    let idp = SamlIdpFixture::default();
    let mut cfg = saml::config(&idp);
    cfg.policy.auto_provision = false;
    cfg.policy.link_by_email = link_by_email;
    let app = SsoApp::build(&db.pool, Some(cfg), None);
    let email = format!("{}@{DOMAIN}", unique("existing"));
    insert_user(&db.pool, &unique("user"), &email).await;
    (app, idp, email)
}

async fn sign_in(
    app: &SsoApp,
    idp: &SamlIdpFixture,
    email: &str,
    sub: &str,
) -> crate::sso_harness::Reply {
    let started = saml::start_default(app).await;
    let mut spec = saml::response_for(&started, email);
    spec.name_id = sub.to_owned();
    saml::post_back(app, idp, &started, &spec).await
}

fn token_roles(access_cookie: &str) -> Vec<String> {
    let jwt = access_cookie.split_once('=').expect("cookie pair").1;
    let payload = jwt.split('.').nth(1).expect("JWT payload");
    let claims: serde_json::Value =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).expect("base64url payload"))
            .expect("claims JSON");
    serde_json::from_value(claims["roles"].clone()).expect("roles claim")
}

#[tokio::test(flavor = "multi_thread")]
async fn an_existing_account_is_linked_by_email_when_the_policy_allows() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (app, idp, email) = existing_account(&db, true).await;
    let existing: String = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&*db.pool)
        .await
        .expect("seeded user");
    let sub = unique("persistent-sub");

    sign_in(&app, &idp, &email, &sub)
        .await
        .assert_signed_in("/admin");
    assert_eq!(
        federated_user(&db.pool, &sub).await.as_deref(),
        Some(existing.as_str()),
        "the identity links to the existing account, not a new one"
    );
    let accounts: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&*db.pool)
        .await
        .expect("count users");
    assert_eq!(accounts, 1, "no duplicate account");
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_existing_account_is_not_claimed_without_link_by_email() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (app, idp, email) = existing_account(&db, false).await;
    let sub = unique("persistent-sub");

    sign_in(&app, &idp, &email, &sub)
        .await
        .assert_refused("not_provisioned");
    assert!(federated_user(&db.pool, &sub).await.is_none());
    assert_eq!(
        login_events(&db.pool).await,
        vec![("refused".to_owned(), Some("not_provisioned".to_owned()))]
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_manual_role_survives_sign_in_in_the_row_and_the_token() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (app, idp, email) = existing_account(&db, true).await;
    let user = insert_user(
        &db.pool,
        &unique("granter"),
        &format!("{}@elsewhere.test", unique("g")),
    )
    .await;
    let id: String = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&*db.pool)
        .await
        .expect("seeded user");
    let target = systemprompt::identifiers::UserId::new(id.clone());
    set_manual_roles(&db.pool, &target, &[MANUAL_ROLE.to_owned()], &user, None)
        .await
        .expect("grant a manual role");

    let reply = sign_in(&app, &idp, &email, &unique("persistent-sub")).await;
    reply.assert_signed_in("/admin");

    let roles: Vec<String> = sqlx::query_scalar("SELECT roles FROM users WHERE id = $1")
        .bind(&id)
        .fetch_one(&*db.pool)
        .await
        .expect("read roles");
    assert!(
        roles.iter().any(|r| r == MANUAL_ROLE),
        "manual role kept in users.roles: {roles:?}"
    );
    assert!(
        roles.iter().any(|r| r == "user"),
        "directory role applied: {roles:?}"
    );
    let minted = token_roles(&reply.cookie("access_token").expect("access_token"));
    assert!(
        minted.iter().any(|r| r == MANUAL_ROLE),
        "manual role in the JWT: {minted:?}"
    );
    db.cleanup().await;
}
