//! Who reaches an article, read back from the access closure.

use dynbrain::access::{ADMIN_ROLE, Grant, Level, Subject};
use dynbrain::ids::ArticleId;

use crate::fixture::{Fixture, user};

fn grant(article: ArticleId, subject: Subject, level: Level, inherit: bool) -> Grant {
    Grant {
        article,
        subject,
        level,
        inherit,
    }
}

async fn level(f: &Fixture, article: ArticleId, name: &str) -> Option<Level> {
    f.repo
        .find_level(article, &user(name))
        .await
        .expect("read the level")
}

#[tokio::test]
async fn nobody_without_a_grant_reaches_an_article_but_its_owner_and_administrators() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    for article in [root.id, child.id] {
        assert_eq!(level(&f, article, "bob").await, None);
        assert_eq!(level(&f, article, "carol").await, None);
        assert_eq!(level(&f, article, "erin").await, None);
        assert_eq!(level(&f, article, "alice").await, Some(Level::Manage));
        assert_eq!(level(&f, article, "root").await, Some(Level::Manage));
    }
    assert_eq!(level(&f, root.id, "nobody").await, None, "an unknown user");
    let subjects: Vec<Subject> = f
        .repo
        .list_access(child.id)
        .await
        .expect("closure")
        .into_iter()
        .map(|entry| entry.subject)
        .collect();
    assert_eq!(
        subjects,
        [Subject::role(ADMIN_ROLE), Subject::user(&user("alice"))]
    );
    f.finish().await;
}

#[tokio::test]
async fn a_grant_reaches_the_subtree_and_revoking_it_takes_it_back() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    let bob = Subject::user(&user("bob"));
    f.repo
        .set_grant(
            &grant(root.id, bob.clone(), Level::Read, true),
            &user("alice"),
        )
        .await
        .expect("grant");
    assert_eq!(level(&f, child.id, "bob").await, Some(Level::Read));
    let grandchild = f.article(space.id, Some(child.id), "grandchild").await;
    assert_eq!(
        level(&f, grandchild.id, "bob").await,
        Some(Level::Read),
        "a new article inherits at creation"
    );
    f.repo
        .set_grant(
            &grant(root.id, bob.clone(), Level::Write, true),
            &user("alice"),
        )
        .await
        .expect("raise");
    assert_eq!(level(&f, grandchild.id, "bob").await, Some(Level::Write));
    assert_eq!(f.repo.list_grants(root.id).await.expect("grants").len(), 1);
    assert!(f.repo.revoke_grant(root.id, &bob).await.expect("revoke"));
    assert!(
        !f.repo
            .revoke_grant(root.id, &bob)
            .await
            .expect("revoke again")
    );
    for article in [root.id, child.id, grandchild.id] {
        assert_eq!(level(&f, article, "bob").await, None);
    }
    f.finish().await;
}

#[tokio::test]
async fn inherit_false_cuts_inheritance() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    f.repo
        .set_grant(
            &grant(root.id, Subject::user(&user("bob")), Level::Read, false),
            &user("alice"),
        )
        .await
        .expect("grant");
    assert_eq!(level(&f, root.id, "bob").await, Some(Level::Read));
    assert_eq!(level(&f, child.id, "bob").await, None);
    f.finish().await;
}

#[tokio::test]
async fn own_grants_replace_the_inherited_ones() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    f.repo
        .set_grant(
            &grant(root.id, Subject::user(&user("bob")), Level::Read, true),
            &user("alice"),
        )
        .await
        .expect("grant on the root");
    f.repo
        .set_grant(
            &grant(child.id, Subject::user(&user("erin")), Level::Write, true),
            &user("alice"),
        )
        .await
        .expect("grant on the child");
    assert_eq!(level(&f, child.id, "bob").await, None);
    assert_eq!(level(&f, child.id, "erin").await, Some(Level::Write));
    assert_eq!(level(&f, root.id, "erin").await, None);
    f.finish().await;
}

#[tokio::test]
async fn a_group_grant_reaches_its_live_members_only() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    f.repo
        .set_grant(
            &grant(root.id, Subject::group("team"), Level::Write, true),
            &user("alice"),
        )
        .await
        .expect("grant");
    assert_eq!(level(&f, child.id, "erin").await, Some(Level::Write));
    assert_eq!(
        level(&f, child.id, "frank").await,
        None,
        "revoked membership"
    );
    assert_eq!(level(&f, child.id, "bob").await, None, "not a member");
    f.finish().await;
}

#[tokio::test]
async fn a_role_grant_reaches_the_users_holding_the_role() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    f.repo
        .set_grant(
            &grant(root.id, Subject::role("editor"), Level::Write, true),
            &user("alice"),
        )
        .await
        .expect("grant");
    assert_eq!(level(&f, root.id, "carol").await, Some(Level::Write));
    assert_eq!(level(&f, root.id, "bob").await, None);
    f.finish().await;
}

#[tokio::test]
async fn an_inactive_account_reaches_nothing() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    for subject in [Subject::role("editor"), Subject::user(&user("dave"))] {
        f.repo
            .set_grant(
                &grant(root.id, subject, Level::Manage, true),
                &user("alice"),
            )
            .await
            .expect("grant");
    }
    assert_eq!(level(&f, root.id, "dave").await, None);
    assert_eq!(level(&f, root.id, "carol").await, Some(Level::Manage));
    f.finish().await;
}
