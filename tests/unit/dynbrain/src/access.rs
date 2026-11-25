//! The access rule of `dynbrain::access`, resolved by `closure` and
//! `inherited`.

use dynbrain::access::{
    ADMIN_ROLE, AccessEntry, Grant, Level, Subject, SubjectKind, TreeNode, closure, inherited,
    subtree,
};
use dynbrain::ids::{ArticleId, SpaceId};
use systemprompt::identifiers::UserId;

fn node(id: i64, parent: Option<i64>) -> TreeNode {
    TreeNode {
        id: ArticleId(id),
        parent: parent.map(ArticleId),
        inherit_access: true,
    }
}

fn on_article(article: i64, subject: &Subject, level: Level) -> Grant {
    Grant {
        space: SpaceId(1),
        article: Some(ArticleId(article)),
        subject: subject.clone(),
        level,
    }
}

fn on_space(subject: &Subject, level: Level) -> Grant {
    Grant {
        space: SpaceId(1),
        article: None,
        subject: subject.clone(),
        level,
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

fn person(name: &str) -> Subject {
    Subject::user(&UserId::new(name))
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

fn cut(nodes: &mut [TreeNode], article: i64) {
    for node in nodes
        .iter_mut()
        .filter(|node| node.id == ArticleId(article))
    {
        node.inherit_access = false;
    }
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
    }
    assert_eq!(level_of(&entries, 1, &person("stranger")), None);
}

#[test]
fn without_an_owner_only_administrators_reach_an_ungranted_article() {
    let entries = closure(&tree(), &[], None);
    assert_eq!(subjects_of(&entries, 1), vec![Subject::role(ADMIN_ROLE)]);
}

#[test]
fn a_space_grant_reaches_every_article() {
    let team = Subject::group("team");
    let entries = closure(&tree(), &[on_space(&team, Level::Read)], None);
    for article in 1..=4 {
        assert_eq!(level_of(&entries, article, &team), Some(Level::Read));
    }
}

#[test]
fn a_grant_on_a_root_reaches_every_descendant() {
    let reader = person("reader");
    let entries = closure(&tree(), &[on_article(1, &reader, Level::Read)], None);
    for article in 1..=3 {
        assert_eq!(level_of(&entries, article, &reader), Some(Level::Read));
    }
    assert_eq!(level_of(&entries, 4, &reader), None, "another root");
}

#[test]
fn a_grant_on_a_child_keeps_the_parents_subjects() {
    let reader = person("reader");
    let team = Subject::group("team");
    let grants = [
        on_article(1, &reader, Level::Read),
        on_article(2, &team, Level::Write),
    ];
    let entries = closure(&tree(), &grants, None);
    assert_eq!(level_of(&entries, 2, &reader), Some(Level::Read));
    assert_eq!(level_of(&entries, 2, &team), Some(Level::Write));
    assert_eq!(level_of(&entries, 3, &reader), Some(Level::Read));
    assert_eq!(level_of(&entries, 3, &team), Some(Level::Write));
    assert_eq!(level_of(&entries, 1, &team), None);
}

#[test]
fn an_own_grant_replaces_the_inherited_level_of_its_subject() {
    let editor = person("editor");
    let grants = [
        on_article(1, &editor, Level::Manage),
        on_article(2, &editor, Level::Read),
    ];
    let entries = closure(&tree(), &grants, None);
    assert_eq!(level_of(&entries, 1, &editor), Some(Level::Manage));
    assert_eq!(level_of(&entries, 2, &editor), Some(Level::Read), "lowered");
    assert_eq!(level_of(&entries, 3, &editor), Some(Level::Read));
}

#[test]
fn a_none_grant_removes_a_subject_for_the_subtree_until_granted_again() {
    let reader = person("reader");
    let other = person("other");
    let grants = [
        on_space(&reader, Level::Write),
        on_space(&other, Level::Read),
        on_article(2, &reader, Level::NoAccess),
        on_article(3, &reader, Level::Read),
    ];
    let entries = closure(&tree(), &grants, None);
    assert_eq!(level_of(&entries, 1, &reader), Some(Level::Write));
    assert_eq!(level_of(&entries, 2, &reader), None);
    assert_eq!(
        level_of(&entries, 3, &reader),
        Some(Level::Read),
        "granted again"
    );
    assert_eq!(level_of(&entries, 2, &other), Some(Level::Read));
    assert!(
        entries.iter().all(|entry| entry.level != Level::NoAccess),
        "none never reaches the closure"
    );
}

#[test]
fn turning_inheritance_off_cuts_everything_from_above() {
    let reader = person("reader");
    let team = Subject::group("team");
    let mut nodes = tree();
    cut(&mut nodes, 2);
    cut(&mut nodes, 4);
    let grants = [
        on_space(&team, Level::Read),
        on_article(1, &reader, Level::Read),
        on_article(3, &person("child"), Level::Write),
    ];
    let entries = closure(&nodes, &grants, Some(&owner()));
    assert_eq!(level_of(&entries, 1, &team), Some(Level::Read));
    assert_eq!(
        subjects_of(&entries, 2),
        vec![Subject::user(&owner()), Subject::role(ADMIN_ROLE)]
    );
    assert_eq!(level_of(&entries, 3, &reader), None);
    assert_eq!(level_of(&entries, 3, &person("child")), Some(Level::Write));
    assert_eq!(level_of(&entries, 4, &team), None, "cut from the space");
}

#[test]
fn inherited_is_what_the_parent_or_the_space_resolves_to() {
    let reader = person("reader");
    let team = Subject::group("team");
    let grants = [
        on_space(&team, Level::Read),
        on_article(1, &reader, Level::Write),
        on_article(2, &team, Level::NoAccess),
    ];
    let nodes = tree();
    assert_eq!(
        inherited(&nodes, &grants, ArticleId(1)),
        vec![(team.clone(), Level::Read)]
    );
    assert_eq!(
        inherited(&nodes, &grants, ArticleId(2)),
        vec![(reader.clone(), Level::Write), (team, Level::Read)]
    );
    assert_eq!(
        inherited(&nodes, &grants, ArticleId(3)),
        vec![(reader, Level::Write)]
    );
}

#[test]
fn the_owner_and_administrators_always_manage() {
    let admins = Subject::role(ADMIN_ROLE);
    let grants = [
        on_space(&Subject::user(&owner()), Level::Read),
        on_space(&admins, Level::Read),
        on_article(2, &admins, Level::NoAccess),
    ];
    let entries = closure(&tree(), &grants, Some(&owner()));
    for article in 1..=3 {
        assert_eq!(
            level_of(&entries, article, &Subject::user(&owner())),
            Some(Level::Manage)
        );
        assert_eq!(level_of(&entries, article, &admins), Some(Level::Manage));
    }
}

#[test]
fn moving_an_article_changes_what_it_inherits() {
    let reader = person("reader");
    let writer = person("writer");
    let grants = [
        on_article(1, &reader, Level::Read),
        on_article(4, &writer, Level::Write),
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
    assert!(Level::NoAccess < Level::Read && Level::Read < Level::Write);
    assert!(Level::Write < Level::Manage);
    for level in [Level::NoAccess, Level::Read, Level::Write, Level::Manage] {
        assert_eq!(Level::from_db(level.as_db()), Some(level));
    }
    assert_eq!(Level::NoAccess.as_db(), 0);
    assert_eq!(Level::from_db(-1), None);
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
