//! `repositories::users` read models — the user index, its filter options, and
//! the per-user side tables the detail page renders.

use systemprompt_web_admin_core::repositories::scope::SubjectScope;
use systemprompt_web_admin_core::repositories::share_token;
use systemprompt_web_admin_identity::repositories::users;

use crate::fixtures::{
    group_scope, insert_user, insert_user_full, project_scope, set_project, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_users_includes_a_freshly_created_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("listed")).await;

    let listed = systemprompt_web_admin_core::repositories::principal::listing::list_users(
        &db.pool,
        &SubjectScope::All,
    )
    .await
    .expect("listing succeeds");

    let row = listed
        .iter()
        .find(|u| u.user_id == user)
        .expect("the new user appears in the index");
    assert!(row.is_active);
    assert_eq!(row.total_events, 0, "a user with no traffic has no events");
    db.cleanup().await;
}

#[tokio::test]
async fn list_users_excludes_anonymous_identities() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let anon = insert_user_full(
        &db.pool,
        &unique("anon"),
        &unclaimed_email("anon"),
        None,
        &["anonymous".to_owned()],
        "active",
    )
    .await;

    let listed = systemprompt_web_admin_core::repositories::principal::listing::list_users(
        &db.pool,
        &SubjectScope::All,
    )
    .await
    .expect("listing succeeds");

    assert!(
        !listed.iter().any(|u| u.user_id == anon),
        "the anonymous role is filtered out of the user index"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_distinct_roles_surfaces_a_new_role_and_drops_machine_roles() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let role = unique("auditor");
    insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("roled"),
        None,
        std::slice::from_ref(&role),
        "active",
    )
    .await;
    insert_user_full(
        &db.pool,
        &unique("svc"),
        &unclaimed_email("svc"),
        None,
        &["service".to_owned()],
        "active",
    )
    .await;

    let roles = users::queries::list_distinct_roles(&db.pool)
        .await
        .expect("listing succeeds");

    assert!(roles.contains(&role));
    assert!(
        !roles.contains(&"service".to_owned()),
        "machine roles are filtered out of the operator-facing list"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_share_token_version_returns_none_without_a_profile_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("share")).await;

    let version = share_token::find_share_token_version(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert_eq!(
        version, None,
        "absence of a profile row is not an error for find_"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_share_token_version_starts_at_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("share2")).await;
    // The profile row is what carries the version; provisioning writes it,
    // the bare user fixture does not.
    sqlx::query("INSERT INTO user_profile_ext (user_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(user.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("profile row");

    let version = share_token::find_share_token_version(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert_eq!(version, Some(1));
    db.cleanup().await;
}

#[tokio::test]
async fn find_user_settings_returns_none_when_the_user_has_none() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("settings")).await;

    let settings = users::user_settings::find_user_settings(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert!(settings.is_none());
    db.cleanup().await;
}

// Why: a non-admin is pinned to the groups and projects they belong to, so
// the scope is the whole split between Commerce and Core on the users page.
// It is worth a test of its own.
#[tokio::test]
async fn list_users_narrows_to_a_groups_and_a_projects_members() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let ours = insert_user(&db.pool, &unique("user"), &unclaimed_email("ours")).await;
    let theirs = insert_user(&db.pool, &unique("user"), &unclaimed_email("theirs")).await;
    set_project(&db.pool, &ours, Some("commerce")).await;
    set_project(&db.pool, &theirs, Some("core")).await;

    let scoped = systemprompt_web_admin_core::repositories::principal::listing::list_users(
        &db.pool,
        &project_scope(&db.pool, "commerce").await,
    )
    .await
    .expect("listing succeeds");
    let ids: Vec<&str> = scoped.iter().map(|u| u.user_id.as_str()).collect();
    assert!(
        ids.contains(&ours.as_str()),
        "the project's own member is listed"
    );
    assert!(
        !ids.contains(&theirs.as_str()),
        "the other project's member must not be listed"
    );

    let by_group = systemprompt_web_admin_core::repositories::principal::listing::list_users(
        &db.pool,
        &group_scope(&db.pool, "commerce").await,
    )
    .await
    .expect("listing succeeds");
    let group_ids: Vec<&str> = by_group.iter().map(|u| u.user_id.as_str()).collect();
    assert!(
        group_ids.contains(&ours.as_str()) && !group_ids.contains(&theirs.as_str()),
        "the group half of the split scopes the same way the project half does"
    );

    let all = systemprompt_web_admin_core::repositories::principal::listing::list_users(
        &db.pool,
        &SubjectScope::All,
    )
    .await
    .expect("listing succeeds");
    let all_ids: Vec<&str> = all.iter().map(|u| u.user_id.as_str()).collect();
    assert!(all_ids.contains(&ours.as_str()));
    assert!(all_ids.contains(&theirs.as_str()));
    db.cleanup().await;
}

// Why: an empty user list is what a caller with no group resolves to. It must
// match nobody rather than falling through to every user, which would make
// the widest scope the one reached by having the least attachment.
#[tokio::test]
async fn list_users_scoped_to_nothing_lists_nobody() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    set_project(&db.pool, &member, Some("core")).await;

    let listed = systemprompt_web_admin_core::repositories::principal::listing::list_users(
        &db.pool,
        &SubjectScope::Users(Vec::new()),
    )
    .await
    .expect("listing succeeds");
    assert!(listed.is_empty(), "the empty scope must match no project");
    db.cleanup().await;
}
