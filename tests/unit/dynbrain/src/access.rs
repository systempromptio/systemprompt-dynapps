//! The inheritance rule of `dynbrain::access`, resolved by `closure`.

use dynbrain::access::{
    ADMIN_ROLE, AccessEntry, Grant, Level, Subject, SubjectKind, TreeNode, closure, subtree,
};
use dynbrain::ids::ArticleId;
use systemprompt::identifiers::UserId;

fn node(id: i64, parent: Option<i64>) -> TreeNode {
    TreeNode {
        id: ArticleId(id),
        parent: parent.map(ArticleId),
    }
}

fn grant(article: i64, subject: &Subject, level: Level, inherit: bool) -> Grant {
    Grant {
        article: ArticleId(article),
        subject: subject.clone(),
        level,
        inherit,
    }
}

fn level_of(entries: &[AccessEntry], article: i64, subject: &Subject) -> Option<Level> {
    entries
        .iter()
        .find(|entry| entry.article == ArticleId(article) && &entry.subject == subject)
        .map(|entry| entry.level)
}

fn subjects_of(entries: &[AccessEntry], article: i64) -> Vec<Subject> {
    entries
        .iter()
        .filter(|entry| entry.article == ArticleId(article))
        .map(|entry| entry.subject.clone())
        .collect()
}

fn owner() -> UserId {
    UserId::new("owner")
}

// root 1 -> child 2 -> grandchild 3, and a second root 4.
fn tree() -> Vec<TreeNode> {
    vec![
        node(1, None),
        node(2, Some(1)),
        node(3, Some(2)),
        node(4, None),
    ]
}

#[test]
fn an_ungranted_article_is_reached_only_by_the_owner_and_administrators() {
    let entries = closure(&tree(), &[], Some(&owner()));
    for article in 1..=4 {
        assert_eq!(
            subjects_of(&entries, article),
            vec![Subject::user(&owner()), Subject::role(ADMIN_ROLE)],
            "article {article}"
        );
        assert_eq!(
            level_of(&entries, article, &Subject::user(&owner())),
            Some(Level::Manage)
        );
    }
    assert_eq!(
        level_of(&entries, 1, &Subject::user(&UserId::new("stranger"))),
        None
    );
}

#[test]
fn without_an_owner_only_administrators_reach_an_ungranted_article() {
    let entries = closure(&tree(), &[], None);
    assert_eq!(subjects_of(&entries, 1), vec![Subject::role(ADMIN_ROLE)]);
}

#[test]
fn a_grant_on_a_root_reaches_every_descendant() {
    let reader = Subject::user(&UserId::new("reader"));
    let entries = closure(
        &tree(),
        &[grant(1, &reader, Level::Read, true)],
        Some(&owner()),
    );
    for article in 1..=3 {
        assert_eq!(level_of(&entries, article, &reader), Some(Level::Read));
    }
    assert_eq!(
        level_of(&entries, 4, &reader),
        None,
        "another root is untouched"
    );
}

#[test]
fn an_article_with_its_own_grants_replaces_its_parents() {
    let reader = Subject::user(&UserId::new("reader"));
    let team = Subject::group("team");
    let grants = [
        grant(1, &reader, Level::Read, true),
        grant(2, &team, Level::Write, true),
    ];
    let entries = closure(&tree(), &grants, Some(&owner()));
    assert_eq!(level_of(&entries, 1, &reader), Some(Level::Read));
    assert_eq!(level_of(&entries, 2, &reader), None);
    assert_eq!(level_of(&entries, 2, &team), Some(Level::Write));
    assert_eq!(
        level_of(&entries, 3, &team),
        Some(Level::Write),
        "the child's grant flows on"
    );
    assert_eq!(level_of(&entries, 3, &reader), None);
}

#[test]
fn a_grant_that_does_not_inherit_stops_at_its_article() {
    let reader = Subject::user(&UserId::new("reader"));
    let editors = Subject::role("editor");
    let grants = [
        grant(1, &reader, Level::Read, false),
        grant(1, &editors, Level::Write, true),
    ];
    let entries = closure(&tree(), &grants, Some(&owner()));
    assert_eq!(level_of(&entries, 1, &reader), Some(Level::Read));
    assert_eq!(level_of(&entries, 2, &reader), None, "inheritance is cut");
    assert_eq!(level_of(&entries, 3, &reader), None);
    assert_eq!(level_of(&entries, 2, &editors), Some(Level::Write));
    assert_eq!(level_of(&entries, 3, &editors), Some(Level::Write));
}

#[test]
fn a_cut_leaves_the_descendants_to_the_owner_and_administrators() {
    let reader = Subject::user(&UserId::new("reader"));
    let entries = closure(
        &tree(),
        &[grant(1, &reader, Level::Manage, false)],
        Some(&owner()),
    );
    assert_eq!(
        subjects_of(&entries, 2),
        vec![Subject::user(&owner()), Subject::role(ADMIN_ROLE)]
    );
}

#[test]
fn a_subject_reached_twice_keeps_its_highest_level() {
    let entries = closure(
        &tree(),
        &[grant(1, &Subject::user(&owner()), Level::Read, true)],
        Some(&owner()),
    );
    assert_eq!(
        level_of(&entries, 2, &Subject::user(&owner())),
        Some(Level::Manage)
    );
    let admins = Subject::role(ADMIN_ROLE);
    let entries = closure(&tree(), &[grant(1, &admins, Level::Read, true)], None);
    assert_eq!(level_of(&entries, 3, &admins), Some(Level::Manage));
}

#[test]
fn moving_an_article_changes_what_it_inherits() {
    let reader = Subject::user(&UserId::new("reader"));
    let writer = Subject::user(&UserId::new("writer"));
    let grants = [
        grant(1, &reader, Level::Read, true),
        grant(4, &writer, Level::Write, true),
    ];
    let before = closure(&tree(), &grants, None);
    assert_eq!(level_of(&before, 3, &reader), Some(Level::Read));
    let moved = vec![
        node(1, None),
        node(2, Some(4)),
        node(3, Some(2)),
        node(4, None),
    ];
    let after = closure(&moved, &grants, None);
    assert_eq!(level_of(&after, 2, &reader), None);
    assert_eq!(level_of(&after, 3, &reader), None);
    assert_eq!(level_of(&after, 3, &writer), Some(Level::Write));
}

#[test]
fn subtree_lists_the_root_and_every_descendant() {
    let nodes = tree();
    assert_eq!(
        subtree(&nodes, ArticleId(1)),
        vec![ArticleId(1), ArticleId(2), ArticleId(3)]
    );
    assert_eq!(subtree(&nodes, ArticleId(3)), vec![ArticleId(3)]);
    assert_eq!(subtree(&nodes, ArticleId(4)), vec![ArticleId(4)]);
}

#[test]
fn levels_are_ordered_and_round_trip_through_the_stored_value() {
    assert!(Level::Read < Level::Write && Level::Write < Level::Manage);
    for level in [Level::Read, Level::Write, Level::Manage] {
        assert_eq!(Level::from_db(level.as_db()), Some(level));
    }
    assert_eq!(Level::from_db(0), None);
    assert_eq!(Level::from_db(4), None);
}

#[test]
fn subject_kinds_round_trip_and_unknown_ones_are_refused() {
    for kind in [SubjectKind::User, SubjectKind::Role, SubjectKind::Group] {
        assert_eq!(SubjectKind::parse(kind.as_db()), Some(kind));
    }
    assert_eq!(SubjectKind::parse("company"), None);
    assert!(!Subject::role("").is_valid());
    assert!(!Subject::group(&"g".repeat(256)).is_valid());
    assert!(Subject::group("team").is_valid());
}
