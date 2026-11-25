//! Who reaches an article, read back from the access closure.

use dynbrain::access::{ADMIN_ROLE, Grant, Level, Subject};
use dynbrain::ids::{ArticleId, SpaceId};

use crate::fixture::{Fixture, user};

pub(crate) fn on(
    space: SpaceId,
    article: Option<ArticleId>,
    subject: Subject,
    level: Level,
) -> Grant {
    Grant {
        space,
        article,
        subject,
        level,
    }
}

async fn level(f: &Fixture, article: ArticleId, name: &str) -> Option<Level> {
    f.repo
        .find_level(article, &user(name))
        .await
        .expect("read the level")
}

async fn give(f: &Fixture, grant: &Grant) {
    f.repo
        .set_grant(grant, &user("alice"))
        .await
        .expect("write the grant");
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
async fn a_space_grant_reaches_every_article() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    give(
        &f,
        &on(space.id, None, Subject::user(&user("bob")), Level::Read),
    )
    .await;
    let later = f.article(space.id, None, "later").await;
    for article in [root.id, child.id, later.id] {
        assert_eq!(level(&f, article, "bob").await, Some(Level::Read));
    }
    assert_eq!(
        f.repo
            .list_grants(space.id, None)
            .await
            .expect("grants")
            .len(),
        1
    );
    assert!(
        f.repo
            .revoke_grant(space.id, None, &Subject::user(&user("bob")))
            .await
            .expect("revoke")
    );
    assert_eq!(level(&f, child.id, "bob").await, None);
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
    give(&f, &on(space.id, Some(root.id), bob.clone(), Level::Read)).await;
    assert_eq!(level(&f, child.id, "bob").await, Some(Level::Read));
    let grandchild = f.article(space.id, Some(child.id), "grandchild").await;
    assert_eq!(
        level(&f, grandchild.id, "bob").await,
        Some(Level::Read),
        "a new article inherits at creation"
    );
    assert!(
        f.repo
            .revoke_grant(space.id, Some(root.id), &bob)
            .await
            .expect("revoke")
    );
    assert!(
        !f.repo
            .revoke_grant(space.id, Some(root.id), &bob)
            .await
            .expect("revoke again")
    );
    for article in [root.id, child.id, grandchild.id] {
        assert_eq!(level(&f, article, "bob").await, None);
    }
    f.finish().await;
}

#[tokio::test]
async fn a_grant_on_a_child_keeps_the_parents_subjects_and_can_lower_one() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    let bob = Subject::user(&user("bob"));
    give(&f, &on(space.id, Some(root.id), bob.clone(), Level::Manage)).await;
    give(
        &f,
        &on(
            space.id,
            Some(child.id),
            Subject::user(&user("erin")),
            Level::Write,
        ),
    )
    .await;
    assert_eq!(level(&f, child.id, "bob").await, Some(Level::Manage));
    assert_eq!(level(&f, child.id, "erin").await, Some(Level::Write));
    assert_eq!(level(&f, root.id, "erin").await, None);
    give(&f, &on(space.id, Some(child.id), bob, Level::Read)).await;
    assert_eq!(
        level(&f, child.id, "bob").await,
        Some(Level::Read),
        "lowered"
    );
    assert_eq!(level(&f, root.id, "bob").await, Some(Level::Manage));
    f.finish().await;
}

#[tokio::test]
async fn a_none_grant_removes_an_inherited_subject_for_the_subtree() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    let grandchild = f.article(space.id, Some(child.id), "grandchild").await;
    let team = Subject::group("team");
    give(&f, &on(space.id, None, team.clone(), Level::Write)).await;
    give(&f, &on(space.id, Some(child.id), team, Level::NoAccess)).await;
    assert_eq!(level(&f, root.id, "erin").await, Some(Level::Write));
    assert_eq!(level(&f, child.id, "erin").await, None);
    assert_eq!(level(&f, grandchild.id, "erin").await, None);
    let stored: Vec<Level> = f
        .repo
        .list_access(grandchild.id)
        .await
        .expect("closure")
        .into_iter()
        .map(|entry| entry.level)
        .collect();
    assert!(!stored.contains(&Level::NoAccess));
    f.finish().await;
}

#[tokio::test]
async fn turning_inheritance_off_cuts_and_copy_inherited_copies() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let cut = f.article(space.id, Some(root.id), "cut").await;
    let copied = f.article(space.id, Some(root.id), "copied").await;
    let below = f.article(space.id, Some(cut.id), "below").await;
    give(
        &f,
        &on(
            space.id,
            Some(root.id),
            Subject::user(&user("bob")),
            Level::Read,
        ),
    )
    .await;
    let article = f
        .repo
        .set_inherit(cut.id, false, false, &user("alice"))
        .await
        .expect("cut");
    assert!(!article.inherit_access);
    assert_eq!(level(&f, cut.id, "bob").await, None);
    assert_eq!(level(&f, below.id, "bob").await, None);
    assert_eq!(level(&f, cut.id, "alice").await, Some(Level::Manage));
    f.repo
        .set_inherit(copied.id, false, true, &user("alice"))
        .await
        .expect("cut with a copy");
    assert_eq!(level(&f, copied.id, "bob").await, Some(Level::Read));
    let own = f
        .repo
        .list_grants(space.id, Some(copied.id))
        .await
        .expect("own grants");
    assert_eq!(own.len(), 1);
    assert_eq!(own[0].subject, Subject::user(&user("bob")));
    give(
        &f,
        &on(
            space.id,
            Some(root.id),
            Subject::user(&user("erin")),
            Level::Read,
        ),
    )
    .await;
    assert_eq!(
        level(&f, copied.id, "erin").await,
        None,
        "it no longer follows its parent"
    );
    f.repo
        .set_inherit(cut.id, true, false, &user("alice"))
        .await
        .expect("inherit again");
    assert_eq!(level(&f, below.id, "bob").await, Some(Level::Read));
    f.finish().await;
}

#[tokio::test]
async fn a_group_grant_reaches_its_live_members_only() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    give(
        &f,
        &on(
            space.id,
            Some(root.id),
            Subject::group("team"),
            Level::Write,
        ),
    )
    .await;
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
    give(
        &f,
        &on(
            space.id,
            Some(root.id),
            Subject::role("editor"),
            Level::Write,
        ),
    )
    .await;
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
        give(&f, &on(space.id, Some(root.id), subject, Level::Manage)).await;
    }
    assert_eq!(level(&f, root.id, "dave").await, None);
    assert_eq!(level(&f, root.id, "carol").await, Some(Level::Manage));
    f.finish().await;
}

#[tokio::test]
async fn a_grant_on_an_article_of_another_space_is_refused() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let other = f.space("manuals").await;
    let elsewhere = f.article(other.id, None, "elsewhere").await;
    let refused = f
        .repo
        .set_grant(
            &on(
                space.id,
                Some(elsewhere.id),
                Subject::user(&user("bob")),
                Level::Read,
            ),
            &user("alice"),
        )
        .await;
    assert!(refused.is_err());
    assert_eq!(level(&f, elsewhere.id, "bob").await, None);
    f.finish().await;
}
