//! The `group` notification audience: core's fabric expands a `group:<id>`
//! subject only through the resolver this extension registers, so an empty
//! or wrong answer here is an announcement that silently reaches nobody or
//! the wrong people.

use std::sync::Arc;

use systemprompt::extension::notifications::{Membership, NotificationAudienceResolver};
use systemprompt::models::notifications::AudienceSubject;
use systemprompt_web_admin_identity::notifications::GroupAudienceResolver;

use crate::fixtures::{
    insert_group, insert_group_member, insert_user, unclaimed_email, unique, unique_group,
};
use crate::tempdb::TempDb;

fn group_subject(id: &str) -> AudienceSubject {
    AudienceSubject::Dimension {
        kind: "group".to_owned(),
        value: id.to_owned(),
    }
}

#[tokio::test]
async fn a_group_expands_to_its_active_members_only() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("naud-m")).await;
    let lapsed = insert_user(&db.pool, &unique("user"), &unclaimed_email("naud-l")).await;
    let outsider = insert_user(&db.pool, &unique("user"), &unclaimed_email("naud-o")).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Engineering").await;
    insert_group_member(&db.pool, &group, &member, "manual").await;
    insert_group_member(&db.pool, &group, &lapsed, "manual").await;
    sqlx::query("UPDATE users SET status = 'inactive' WHERE id = $1")
        .bind(lapsed.as_str())
        .execute(&*db.pool)
        .await
        .expect("deactivate");
    let resolver = GroupAudienceResolver::new(Some(Arc::new((*db.pool).clone())));
    let subject = group_subject(group.as_str());

    let members = resolver.members(&subject, 10).await.expect("expand group");

    assert_eq!(members, vec![member.clone()]);
    assert_eq!(
        resolver.is_member(&subject, &member).await.expect("member"),
        Membership::Member
    );
    assert_eq!(
        resolver.is_member(&subject, &lapsed).await.expect("lapsed"),
        Membership::NotMember
    );
    assert_eq!(
        resolver
            .is_member(&subject, &outsider)
            .await
            .expect("outsider"),
        Membership::NotMember
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_expansion_honours_the_limit_core_asks_for() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Engineering").await;
    for n in 0..3 {
        let user = insert_user(
            &db.pool,
            &unique("user"),
            &unclaimed_email(&format!("naud-cap{n}")),
        )
        .await;
        insert_group_member(&db.pool, &group, &user, "manual").await;
    }
    let resolver = GroupAudienceResolver::new(Some(Arc::new((*db.pool).clone())));

    let members = resolver
        .members(&group_subject(group.as_str()), 2)
        .await
        .expect("expand group");

    assert_eq!(
        members.len(),
        2,
        "core detects an oversize group from limit + 1 rows"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_resolver_claims_only_group_subjects() {
    let resolver = GroupAudienceResolver::new(None);

    assert!(resolver.handles(&group_subject("uk")));
    assert!(!resolver.handles(&AudienceSubject::Role("admin".to_owned())));
    assert!(!resolver.handles(&AudienceSubject::Everyone));
    assert!(!resolver.handles(&AudienceSubject::Dimension {
        kind: "project".to_owned(),
        value: "uk".to_owned(),
    }));
}

#[tokio::test]
async fn without_a_database_the_group_is_withheld_not_empty() {
    let resolver = GroupAudienceResolver::new(None);

    let expanded = resolver.members(&group_subject("uk"), 10).await;

    assert!(
        expanded.is_err(),
        "an empty audience would read as a delivered announcement"
    );
}
