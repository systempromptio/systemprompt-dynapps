//! `repositories::users::queries::find_identity_envelope` — the single read
//! behind `GET /api/public/bridge/whoami`.
//!
//! The endpoint's whole reason to exist is that a caller's identity is spread
//! across five tables, so the assertions here are about the joins: that an
//! account with no SSO mapping and no organization still resolves, that the
//! federated and organization halves attach when they exist, and that a user
//! holding two IdP mappings reports the one they last signed in with.

use systemprompt_web_admin::repositories::users::queries::find_identity_envelope;

use crate::fixtures::{
    OrgSpec, insert_federated_identity, insert_member, insert_org, insert_user, insert_user_full,
    set_department, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

const ISSUER: &str = "https://login.adfs.test/adfs";

#[tokio::test]
async fn a_local_account_resolves_with_no_federated_or_organization_half() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("local")).await;

    let envelope = find_identity_envelope(&db.pool, &user)
        .await
        .expect("query succeeds")
        .expect("the user exists");

    assert_eq!(envelope.user_id, user);
    assert!(envelope.idp_issuer.is_none(), "never signed in via an IdP");
    assert!(envelope.organization_id.is_none(), "no seat");
    assert_eq!(
        envelope.department, "Default",
        "an unset department reads as Default, not as a null the card would drop"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_sso_account_carries_its_issuer_department_and_seat() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("sso");
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &email,
        Some("Federated Person"),
        &["admin".to_owned(), "user".to_owned()],
        "active",
    )
    .await;
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &user).await;
    set_department(&db.pool, &user, "Engineering").await;
    let org_id = unique("org");
    insert_org(&db.pool, &OrgSpec::active(&org_id, &unique("slug"))).await;
    insert_member(&db.pool, &user, &org_id, "owner").await;

    let envelope = find_identity_envelope(&db.pool, &user)
        .await
        .expect("query succeeds")
        .expect("the user exists");

    assert_eq!(envelope.idp_issuer.as_deref(), Some(ISSUER));
    assert_eq!(envelope.external_sub.as_deref(), Some(sub.as_str()));
    assert_eq!(envelope.department, "Engineering");
    assert_eq!(envelope.organization_id.as_deref(), Some(org_id.as_str()));
    assert_eq!(envelope.org_role.as_deref(), Some("owner"));
    assert_eq!(envelope.display_name.as_deref(), Some("Federated Person"));
    assert!(envelope.roles.contains(&"admin".to_owned()));
    assert!(
        envelope.linked_at.is_some() && envelope.last_seen_at.is_some(),
        "both federated timestamps are NOT NULL columns"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn two_mappings_report_the_one_most_recently_signed_in_with() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("two")).await;
    let old_sub = unique("sub-old");
    let new_sub = unique("sub-new");
    insert_federated_identity(&db.pool, "https://legacy.idp.test", &old_sub, &user).await;
    insert_federated_identity(&db.pool, ISSUER, &new_sub, &user).await;
    // Why: both rows default `last_seen_at` to now within the same statement
    // batch, so the ordering under test has to be made unambiguous rather than
    // raced against clock resolution.
    sqlx::query(
        "UPDATE federated_identities SET last_seen_at = NOW() - INTERVAL '1 day'
         WHERE issuer = $1 AND external_sub = $2",
    )
    .bind("https://legacy.idp.test")
    .bind(&old_sub)
    .execute(db.pool.as_ref())
    .await
    .expect("age the legacy mapping");

    let envelope = find_identity_envelope(&db.pool, &user)
        .await
        .expect("query succeeds")
        .expect("the user exists");

    assert_eq!(
        envelope.idp_issuer.as_deref(),
        Some(ISSUER),
        "the LATERAL join takes the newest mapping, and takes exactly one"
    );
    db.cleanup().await;
}
