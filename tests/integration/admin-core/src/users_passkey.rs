//! `repositories::users::passkey` — passkey self-registration, which mirrors
//! the SSO provisioning rules minus the federated mapping.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::users::passkey;

use crate::fixtures::{
    OrgSpec, insert_member, insert_org, insert_plan, insert_user, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn passkey_find_user_by_email_returns_none_for_a_stranger() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let found = passkey::find_user_by_email(&db.pool, &unclaimed_email("stranger"))
        .await
        .expect("lookup succeeds");

    assert!(found.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn passkey_find_user_by_email_reports_no_credential_yet() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("passkey");
    let user = insert_user(&db.pool, &unique("user"), &email).await;

    let found = passkey::find_user_by_email(&db.pool, &email.to_uppercase())
        .await
        .expect("lookup succeeds")
        .expect("the account exists");

    assert_eq!(found.id, user);
    assert!(
        !found.has_passkey,
        "a fresh account has no webauthn credential"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn passkey_count_webauthn_credentials_starts_at_zero() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("count")).await;

    let count = passkey::count_webauthn_credentials(&db.pool, &user)
        .await
        .expect("count succeeds");

    assert_eq!(count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn passkey_count_webauthn_credentials_is_zero_for_an_unknown_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let count = passkey::count_webauthn_credentials(&db.pool, &UserId::new(unique("absent")))
        .await
        .expect("count succeeds");

    assert_eq!(
        count, 0,
        "get_-shaped counts report zero rather than erroring on an unknown subject"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn passkey_insert_setup_token_stores_a_credential_link_token() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("token")).await;
    let expires = chrono::Utc::now() + chrono::Duration::minutes(15);

    passkey::insert_setup_token(&db.pool, &user, "hash-of-token", expires)
        .await
        .expect("insert succeeds");

    let purpose: String =
        sqlx::query_scalar("SELECT purpose FROM webauthn_setup_tokens WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("the token row exists");
    assert_eq!(purpose, "credential_link");
    db.cleanup().await;
}

#[tokio::test]
async fn passkey_insert_passkey_user_provisions_like_the_sso_door() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("selfreg");

    let user_id = passkey::insert_passkey_user(&db.pool, &email, "Self Registered")
        .await
        .expect("self-registration succeeds on an unclaimed domain");

    let (name, display, verified): (String, Option<String>, bool) =
        sqlx::query_as("SELECT name, display_name, email_verified FROM users WHERE id = $1")
            .bind(user_id.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("the new user exists");
    assert_eq!(
        name, email,
        "name carries the email to dodge the uniqueness constraint"
    );
    assert_eq!(display.as_deref(), Some("Self Registered"));
    assert!(
        verified,
        "the domain allowlist is the gate, so the address is trusted"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn passkey_insert_passkey_user_refuses_a_full_plan() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let domain = format!("{}.example", uuid::Uuid::new_v4().simple());
    let plan_id = unique("plan");
    insert_plan(&db.pool, &plan_id, Some(1), None, 0).await;
    let org_id = unique("org");
    let mut spec = OrgSpec::active(&org_id, &org_id);
    spec.plan_id = Some(&plan_id);
    spec.email_domains = vec![domain.clone()];
    insert_org(&db.pool, &spec).await;
    let sitting = insert_user(&db.pool, &unique("user"), &format!("first@{domain}")).await;
    insert_member(&db.pool, &sitting, &org_id, "member").await;

    let refused = passkey::insert_passkey_user(&db.pool, &format!("second@{domain}"), "Second")
        .await
        .expect_err("the seat limit is enforced on this door too");

    assert!(refused.to_string().contains("seat limit reached"));
    db.cleanup().await;
}
