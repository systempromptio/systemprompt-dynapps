//! `repositories::scope::membership::list_managed_member_ids` — the
//! managed-members list a `project_manager`'s history scope is built from.
//!
//! These tests pin that the list is the live members of the manager's own
//! live projects: another project's members, a revoked or expired membership
//! on either side, never appear.

use systemprompt_web_admin_core::repositories::scope::membership::list_managed_member_ids;

use crate::fixtures::{
    insert_project, insert_project_member, insert_user, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn a_manager_gets_the_live_members_of_their_projects_only() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pm = insert_user(&db.pool, &unique("pm"), &unclaimed_email("pm")).await;
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("mem")).await;
    let revoked = insert_user(&db.pool, &unique("user"), &unclaimed_email("rev")).await;
    let expired = insert_user(&db.pool, &unique("user"), &unclaimed_email("exp")).await;
    let outsider = insert_user(&db.pool, &unique("user"), &unclaimed_email("out")).await;

    let mine = unique("proj");
    let theirs = unique("proj");
    insert_project(&db.pool, &mine, "Mine").await;
    insert_project(&db.pool, &theirs, "Theirs").await;
    for user in [&pm, &member, &revoked, &expired] {
        insert_project_member(&db.pool, &mine, user, "manual").await;
    }
    insert_project_member(&db.pool, &theirs, &outsider, "manual").await;
    sqlx::query("UPDATE project_members SET revoked_at = NOW() WHERE user_id = $1")
        .bind(revoked.as_str())
        .execute(&*db.pool)
        .await
        .expect("revoke");
    sqlx::query(
        "UPDATE project_members SET valid_until = NOW() - INTERVAL '1 minute' WHERE user_id = $1",
    )
    .bind(expired.as_str())
    .execute(&*db.pool)
    .await
    .expect("expire");

    let mut ids: Vec<String> = list_managed_member_ids(&db.pool, &pm)
        .await
        .expect("list")
        .into_iter()
        .map(|id| id.as_str().to_owned())
        .collect();
    ids.sort();
    let mut expected = vec![pm.as_str().to_owned(), member.as_str().to_owned()];
    expected.sort();
    assert_eq!(ids, expected);
    db.cleanup().await;
}

#[tokio::test]
async fn a_manager_whose_own_membership_lapsed_manages_nobody() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pm = insert_user(&db.pool, &unique("pm"), &unclaimed_email("pmx")).await;
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("memx")).await;
    let project = unique("proj");
    insert_project(&db.pool, &project, "Lapsed").await;
    insert_project_member(&db.pool, &project, &pm, "manual").await;
    insert_project_member(&db.pool, &project, &member, "manual").await;
    sqlx::query("UPDATE project_members SET revoked_at = NOW() WHERE user_id = $1")
        .bind(pm.as_str())
        .execute(&*db.pool)
        .await
        .expect("revoke");

    let ids = list_managed_member_ids(&db.pool, &pm).await.expect("list");
    assert!(ids.is_empty());
    db.cleanup().await;
}
