//! The Brain's contract without a database: how a fault answers over HTTP, how
//! a store error becomes a fault, and the field names the operations take and
//! return (the ones of the `docs_*` tools).

use dynbrain::BrainError;
use dynbrain::access::{Level, SubjectKind};
use dynbrain::brain::args::{
    GrantInput, InheritArgs, ListArgs, Position, SearchArgs, TreeArgs, UpdateArgs,
};
use dynbrain::brain::fault::BrainFault;
use dynbrain::brain::views::{GrantView, Grants, Moved, Trashed};
use dynbrain::ids::{ArticleId, SpaceId};
use dynbrain::router::error::HttpFault;
use serde_json::json;
use systemprompt::traits::FileStorageError;

#[test]
fn a_fault_answers_with_its_status_and_a_stable_key() {
    let cases = [
        (BrainFault::NotFound, 404, "not_found"),
        (BrainFault::Forbidden, 403, "forbidden"),
        (BrainFault::Invalid("no"), 400, "invalid"),
        (
            BrainFault::Refused(BrainError::Invalid("no")),
            400,
            "invalid",
        ),
        (BrainFault::Unavailable("later"), 503, "unavailable"),
        (BrainFault::TooLarge(10), 413, "too_large"),
        (
            BrainFault::Internal(BrainError::Invalid("boom")),
            500,
            "internal",
        ),
    ];
    for (fault, status, key) in cases {
        let http = HttpFault::from(fault);
        assert_eq!(http.status().as_u16(), status, "{http}");
        assert_eq!(http.key(), key, "{http}");
    }
    assert_eq!(HttpFault::NotFound.status().as_u16(), 404);
    assert_eq!(HttpFault::BadRequest("no").key(), "bad_request");
}

#[test]
fn a_store_error_becomes_the_fault_a_caller_can_act_on() {
    let missing = BrainFault::from(BrainError::ArticleNotFound(ArticleId(1)));
    assert!(matches!(missing, BrainFault::NotFound));
    let space = BrainFault::from(BrainError::SpaceNotFound(SpaceId(1)));
    assert!(matches!(space, BrainFault::NotFound));
    let cycle = BrainFault::from(BrainError::MoveIntoOwnSubtree(ArticleId(1)));
    assert!(matches!(cycle, BrainFault::Refused(_)));
    let trashed = BrainFault::from(BrainError::Trashed(ArticleId(1)));
    assert!(matches!(trashed, BrainFault::Refused(_)));
    assert_eq!(trashed.to_string(), "article 1 is in the trash");
    let storage = BrainFault::from(BrainError::Storage(FileStorageError::NotFound(
        "dynbrain/uploads/x".to_owned(),
    )));
    assert!(matches!(storage, BrainFault::Internal(_)));
}

#[test]
fn the_arguments_use_the_field_names_of_the_docs_tools() {
    let search: SearchArgs = serde_json::from_value(json!({
        "query": "backups", "limit": 5, "lang": "es", "space": 3, "where": {"status": "decided"}
    }))
    .expect("search arguments");
    assert_eq!(search.query, "backups");
    assert_eq!(search.space, Some(SpaceId(3)));
    assert!(search.filter.is_some());
    let list: ListArgs =
        serde_json::from_value(json!({"space": 7, "offset": 2})).expect("list arguments");
    assert_eq!(list.space, SpaceId(7));
    assert_eq!(list.offset, Some(2));
    let update: UpdateArgs =
        serde_json::from_value(json!({"title": "T", "lang": "en"})).expect("update arguments");
    assert_eq!(update.title.as_deref(), Some("T"));
    assert_eq!(update.article_id, ArticleId(0), "the route fills the id in");
    let tree: TreeArgs = serde_json::from_value(json!({
        "space": 1, "nodes": [{"title": "A", "children": [{"title": "B"}]}]
    }))
    .expect("tree arguments");
    assert_eq!(tree.nodes[0].children[0].title, "B");
}

#[test]
fn grants_are_written_with_the_levels_the_dialog_offers() {
    for (word, level) in [
        ("none", Level::NoAccess),
        ("read", Level::Read),
        ("write", Level::Write),
        ("manage", Level::Manage),
    ] {
        let input: GrantInput =
            serde_json::from_value(json!({"kind": "group", "id": "team", "level": word}))
                .expect("a grant");
        assert_eq!(input.level, level);
        assert_eq!(input.kind, SubjectKind::Group);
    }
    let unknown =
        serde_json::from_value::<GrantInput>(json!({"kind": "user", "id": "a", "level": "owner"}));
    assert!(unknown.is_err());
    let inherit: InheritArgs = serde_json::from_value(json!({})).expect("defaults");
    assert!(!inherit.inherit && !inherit.copy_inherited);
    let position: Position = serde_json::from_value(json!("first")).expect("a position");
    assert_eq!(position, Position::First);
}

#[test]
fn the_answers_use_the_field_names_of_the_docs_tools() {
    let moved = Moved {
        article_id: ArticleId(5),
        new_parent_id: None,
        applied: true,
    };
    assert_eq!(
        serde_json::to_value(moved).expect("serialize"),
        json!({"article_id": 5, "new_parent_id": null, "moved": true})
    );
    let trashed = Trashed {
        article_id: ArticleId(5),
        applied: true,
    };
    assert_eq!(
        serde_json::to_value(trashed).expect("serialize"),
        json!({"article_id": 5, "trashed": true})
    );
    let grants = Grants {
        own: vec![GrantView {
            kind: SubjectKind::Role,
            id: "editor".to_owned(),
            level: Level::NoAccess,
        }],
        access: Vec::new(),
        inherit_access: Some(true),
    };
    assert_eq!(
        serde_json::to_value(grants).expect("serialize"),
        json!({
            "grants": [{"kind": "role", "id": "editor", "level": "none"}],
            "access": [],
            "inherit_access": true
        })
    );
}
