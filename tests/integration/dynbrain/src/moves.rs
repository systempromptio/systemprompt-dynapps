//! Moving an article rewrites the access closure of its subtree.

use dynbrain::BrainError;
use dynbrain::access::{Level, Subject};
use dynbrain::model::ArticleMove;

use crate::fixture::{Fixture, user};
use crate::permissions::on;

#[tokio::test]
async fn moving_drops_what_the_subtree_inherited_from_its_old_place() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let sales = f.article(space.id, None, "sales").await;
    let people = f.article(space.id, None, "people").await;
    let policy = f.article(space.id, Some(sales.id), "policy").await;
    let annex = f.article(space.id, Some(policy.id), "annex").await;
    for (article, name) in [(sales.id, "bob"), (people.id, "erin")] {
        f.repo
            .set_grant(
                &on(
                    space.id,
                    Some(article),
                    Subject::user(&user(name)),
                    Level::Read,
                ),
                &user("alice"),
            )
            .await
            .expect("grant");
    }
    assert_eq!(
        f.repo
            .find_level(annex.id, &user("bob"))
            .await
            .expect("level"),
        Some(Level::Read)
    );
    let moved = f
        .repo
        .move_article(
            policy.id,
            ArticleMove {
                parent: Some(people.id),
                position: None,
            },
        )
        .await
        .expect("move");
    assert_eq!(moved.parent, Some(people.id));
    for article in [policy.id, annex.id] {
        assert_eq!(
            f.repo
                .find_level(article, &user("bob"))
                .await
                .expect("level"),
            None,
            "bob read it only through its old place"
        );
        assert_eq!(
            f.repo
                .find_level(article, &user("erin"))
                .await
                .expect("level"),
            Some(Level::Read)
        );
    }
    let to_top = f
        .repo
        .move_article(
            policy.id,
            ArticleMove {
                parent: None,
                position: Some(5),
            },
        )
        .await
        .expect("move to the top level");
    assert_eq!((to_top.parent, to_top.position), (None, 5));
    assert_eq!(
        f.repo
            .find_level(annex.id, &user("erin"))
            .await
            .expect("level"),
        None
    );
    f.finish().await;
}

#[tokio::test]
async fn a_move_below_itself_or_into_another_space_is_refused() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let other = f.space("manuals").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    let elsewhere = f.article(other.id, None, "elsewhere").await;
    for parent in [root.id, child.id] {
        let below = f
            .repo
            .move_article(
                root.id,
                ArticleMove {
                    parent: Some(parent),
                    position: None,
                },
            )
            .await;
        assert!(matches!(below, Err(BrainError::MoveIntoOwnSubtree(_))));
    }
    let across = f
        .repo
        .move_article(
            child.id,
            ArticleMove {
                parent: Some(elsewhere.id),
                position: None,
            },
        )
        .await;
    assert!(matches!(across, Err(BrainError::MoveAcrossSpaces(_))));
    let unchanged = f
        .repo
        .find_article(child.id)
        .await
        .expect("find")
        .expect("kept");
    assert_eq!(unchanged.parent, Some(root.id));
    f.finish().await;
}
