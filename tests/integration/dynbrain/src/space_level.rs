//! The level a person has on a space itself, read from its grants.

use dynbrain::access::{Level, Subject};

use crate::fixture::{Fixture, user};

async fn level(f: &Fixture, space: dynbrain::ids::SpaceId, name: &str) -> Option<Level> {
    f.repo
        .find_space_level(space, &user(name))
        .await
        .expect("read the space level")
}

#[tokio::test]
async fn the_owner_and_administrators_manage_and_the_rest_follow_the_space_grants() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let article = f.article(space.id, None, "guide").await;
    f.give(space.id, None, Subject::role("editor"), Level::Write)
        .await;
    f.give(space.id, None, Subject::group("team"), Level::Read)
        .await;
    f.give(space.id, None, Subject::user(&user("bob")), Level::NoAccess)
        .await;
    f.give(
        space.id,
        Some(article.id),
        Subject::user(&user("frank")),
        Level::Manage,
    )
    .await;
    assert_eq!(level(&f, space.id, "alice").await, Some(Level::Manage));
    assert_eq!(level(&f, space.id, "root").await, Some(Level::Manage));
    assert_eq!(level(&f, space.id, "carol").await, Some(Level::Write));
    assert_eq!(level(&f, space.id, "erin").await, Some(Level::Read));
    assert_eq!(level(&f, space.id, "bob").await, Some(Level::NoAccess));
    assert_eq!(
        level(&f, space.id, "frank").await,
        None,
        "an article grant is not a space grant"
    );
    assert_eq!(
        level(&f, space.id, "dave").await,
        None,
        "a suspended account"
    );
    assert_eq!(level(&f, space.id, "nobody").await, None);
    f.finish().await;
}
