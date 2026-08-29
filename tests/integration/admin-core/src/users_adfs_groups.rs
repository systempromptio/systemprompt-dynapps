//! `repositories::users::adfs_groups` — the AD group side table the ADFS
//! callback writes and the `adfs_group` dimension reads.

use systemprompt_security::authz::SubjectAttributeProvider;
use systemprompt_web_admin::authz::adfs_group::AdfsGroupAttributeProvider;
use systemprompt_web_admin::repositories::users::adfs_groups::{list_groups, replace_groups};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_groups_is_empty_before_any_sso_login() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("nogroups")).await;

    let groups = list_groups(&db.pool, &user).await.expect("list groups");

    assert!(groups.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn replace_groups_reflects_the_latest_token_not_the_union() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("groups")).await;

    replace_groups(
        &db.pool,
        &user,
        &[
            "Systemprompt-Users".to_owned(),
            "Systemprompt-Sales".to_owned(),
        ],
    )
    .await
    .expect("first login");
    replace_groups(&db.pool, &user, &["Systemprompt-Users".to_owned()])
        .await
        .expect("second login, one group fewer");

    let groups = list_groups(&db.pool, &user).await.expect("list groups");

    assert_eq!(
        groups,
        vec!["Systemprompt-Users".to_owned()],
        "a group the user has left is gone at the next login"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_adfs_group_dimension_resolves_the_recorded_groups() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("dim")).await;
    replace_groups(&db.pool, &user, &["Systemprompt-Engineering".to_owned()])
        .await
        .expect("record groups");
    let provider = AdfsGroupAttributeProvider::new(std::sync::Arc::new((*db.pool).clone()));

    let values = provider.values_for(&user).await;

    assert_eq!(values, vec!["Systemprompt-Engineering".to_owned()]);
    let dim = provider.dimension();
    assert_eq!(dim.rule_type.as_str(), "adfs_group");
    assert!(
        dim.precedence > 100 && dim.precedence < 200,
        "must out-rank the role band and defer to department, got {}",
        dim.precedence
    );
    db.cleanup().await;
}
