//! `repositories::users::role_grants` — one time-boxed manual role, as the
//! console's Odoo demo grants `odoo_destructive` to the farm service account —
//! and the read the govern hook takes roles through.
//!
//! The demo must never erase a grant an admin made by hand, and the govern
//! hook must stop seeing a lapsed grant before the hourly sweep rewrites
//! `users.roles`; these tests pin both.

use chrono::{Duration, Utc};
use systemprompt_web_admin_core::authz::account::account_scope_and_roles;
use systemprompt_web_admin_identity::repositories::users::role_grants::{
    TimedGrant, delete_timed_manual_role, set_timed_manual_role,
};
use systemprompt_web_admin_identity::repositories::users::roles::{
    list_manual_roles, set_manual_roles,
};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

const ROLE: &str = "odoo_destructive";

#[tokio::test]
async fn a_timed_grant_is_added_beside_existing_manual_roles_and_revoked_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let account = insert_user(&db.pool, &unique("svc"), &unclaimed_email("svc")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;
    set_manual_roles(&db.pool, &account, &["developer".to_owned()], &admin, None)
        .await
        .expect("standing developer grant");

    let grant = set_timed_manual_role(
        &db.pool,
        &account,
        ROLE,
        &admin,
        Utc::now() + Duration::minutes(30),
    )
    .await
    .expect("timed grant");
    assert_eq!(grant, TimedGrant::Granted);
    let (_, roles) = account_scope_and_roles(&db.pool, &account)
        .await
        .expect("roles");
    assert!(roles.contains(&ROLE.to_owned()) && roles.contains(&"developer".to_owned()));

    assert!(
        delete_timed_manual_role(&db.pool, &account, ROLE)
            .await
            .expect("revoke")
    );
    let (_, roles) = account_scope_and_roles(&db.pool, &account)
        .await
        .expect("roles");
    assert!(
        !roles.contains(&ROLE.to_owned()),
        "revoked role must leave the effective set"
    );
    assert_eq!(
        list_manual_roles(&db.pool, &account).await.expect("manual"),
        ["developer".to_owned()],
        "the demo must not touch a grant made by hand"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_standing_grant_wins_over_a_timed_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let account = insert_user(&db.pool, &unique("svc"), &unclaimed_email("svc")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;
    set_manual_roles(&db.pool, &account, &[ROLE.to_owned()], &admin, None)
        .await
        .expect("standing grant");

    let grant = set_timed_manual_role(
        &db.pool,
        &account,
        ROLE,
        &admin,
        Utc::now() + Duration::minutes(30),
    )
    .await
    .expect("timed grant");
    assert_eq!(grant, TimedGrant::Standing);
    assert!(
        !delete_timed_manual_role(&db.pool, &account, ROLE)
            .await
            .expect("revoke")
    );
    let (_, roles) = account_scope_and_roles(&db.pool, &account)
        .await
        .expect("roles");
    assert!(
        roles.contains(&ROLE.to_owned()),
        "a standing grant survives the demo"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_govern_read_drops_a_lapsed_grant_before_the_sweep_runs() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let account = insert_user(&db.pool, &unique("svc"), &unclaimed_email("svc")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;
    set_timed_manual_role(
        &db.pool,
        &account,
        ROLE,
        &admin,
        Utc::now() + Duration::minutes(30),
    )
    .await
    .expect("timed grant");
    sqlx::query(
        "UPDATE user_manual_roles SET valid_until = NOW() - INTERVAL '1 minute' WHERE user_id = $1",
    )
    .bind(account.as_str())
    .execute(&*db.pool)
    .await
    .expect("lapse the grant");
    let stored: Vec<String> = sqlx::query_scalar("SELECT roles FROM users WHERE id = $1")
        .bind(account.as_str())
        .fetch_one(&*db.pool)
        .await
        .expect("stored roles");
    assert!(
        stored.contains(&ROLE.to_owned()),
        "users.roles still holds it until the sweep"
    );

    let (_, roles) = account_scope_and_roles(&db.pool, &account)
        .await
        .expect("roles");
    assert!(
        !roles.contains(&ROLE.to_owned()),
        "the govern hook must not see a lapsed grant"
    );
    db.cleanup().await;
}
