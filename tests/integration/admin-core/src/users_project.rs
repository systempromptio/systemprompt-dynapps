//! `repositories::users::project` — the side row AD FS writes on every
//! sign-in, and the per-request roles+project lookup that reads it.

use systemprompt_web_admin::repositories::users::project::{find_project, set_project};
use systemprompt_web_admin::repositories::users::queries::find_user_roles_project;
use systemprompt_web_admin::types::Project;

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn set_project_upserts_the_profile_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("upsert")).await;

    set_project(&db.pool, &user, Some(Project::Commerce))
        .await
        .expect("insert");
    set_project(&db.pool, &user, Some(Project::Core))
        .await
        .expect("update in place");

    assert_eq!(
        find_project(&db.pool, &user).await.expect("lookup"),
        Some(Project::Core)
    );
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_profile_ext WHERE user_id = $1")
        .bind(user.as_str())
        .fetch_one(&*db.pool)
        .await
        .expect("count");
    assert_eq!(
        rows, 1,
        "one profile row per user, whatever the number of sign-ins"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn set_project_to_none_clears_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("clear")).await;
    set_project(&db.pool, &user, Some(Project::Core))
        .await
        .expect("insert");

    set_project(&db.pool, &user, None).await.expect("clear");

    assert_eq!(find_project(&db.pool, &user).await.expect("lookup"), None);
    db.cleanup().await;
}

#[tokio::test]
async fn find_user_roles_project_reads_null_for_a_user_with_no_profile_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("norow")).await;

    let (roles, project) = find_user_roles_project(&db.pool, &user)
        .await
        .expect("lookup")
        .expect("the user exists");

    assert_eq!(roles, vec!["user".to_owned()]);
    assert_eq!(project, None);
    db.cleanup().await;
}
