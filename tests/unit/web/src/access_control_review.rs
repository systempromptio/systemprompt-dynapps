//! The access review: drift regrouped by entity, one kind per entity, and a
//! "keep the database" decision that holds only for the diff it was taken on.

use chrono::Utc;
use systemprompt::governance::authz::{Access, RegisteredEntities, SERVICES_SOURCE};
use systemprompt_web_admin_core::repositories::access_rules::declared::{
    DeclaredInputs, DeclaredSet, build_declared_set,
};
use systemprompt_web_admin_core::repositories::access_rules::drift::{
    BandRuleRow, EntityDefaultRow, compute_drift,
};
use systemprompt_web_admin_core::repositories::access_rules::review::{
    EntityReview, ReviewKind, review_entities,
};
use systemprompt_web_admin_core::repositories::access_rules::rules_yaml_loader::parse_rules_doc;
use systemprompt_web_admin_core::repositories::sync::attention::split_reviews;
use systemprompt_web_admin_core::repositories::sync::history::KeptReview;

const CONSOLE: &str = "console:ana";

fn declared() -> DeclaredSet {
    let doc = parse_rules_doc(
        "entities:\n  - entity: mcp_server/dynbrain\n    default: closed\n    why: pilot\n    allow:\n      role: [admin]\n      group: [india-devs]\n  - entity: mcp_server/dynapps-odoo\n    default: closed\n    why: pilot\n    allow:\n      role: [admin]\n",
    )
    .expect("parses");
    build_declared_set(
        &doc,
        &DeclaredInputs {
            gateway_routes: &[],
            marketplace_ids: &[],
            registered: &RegisteredEntities::default(),
        },
    )
    .expect("projects")
}

fn row(entity_id: &str, rule_type: &str, value: &str, access: Access, source: &str) -> BandRuleRow {
    BandRuleRow {
        id: format!("{entity_id}-{rule_type}-{value}"),
        entity_type: "mcp_server".to_owned(),
        entity_id: entity_id.to_owned(),
        rule_type: rule_type.to_owned(),
        rule_value: value.to_owned(),
        access,
        justification: Some("pilot".to_owned()),
        source: source.to_owned(),
        valid_until: None,
    }
}

fn entity(entity_id: &str) -> EntityDefaultRow {
    EntityDefaultRow {
        entity_type: "mcp_server".to_owned(),
        entity_id: entity_id.to_owned(),
        default_included: false,
        source: SERVICES_SOURCE.to_owned(),
    }
}

fn find<'a>(reviews: &'a [EntityReview], id: &str) -> &'a EntityReview {
    reviews
        .iter()
        .find(|r| r.entity_id == id)
        .unwrap_or_else(|| panic!("no review for {id}: {reviews:?}"))
}

#[test]
fn an_entity_never_applied_is_new_in_code_and_not_live() {
    let reviews = review_entities(&compute_drift(&declared(), &[], &[]));
    let odoo = find(&reviews, "dynapps-odoo");
    assert_eq!(odoo.kind, ReviewKind::NewInCode);
    assert!(odoo.not_live);
    assert!(odoo.applies);
    assert_eq!(odoo.key, "mcp_server/dynapps-odoo");
}

#[test]
fn a_live_entity_missing_a_declared_rule_reads_as_removed_in_console() {
    let rules = [row(
        "dynbrain",
        "role",
        "admin",
        Access::Allow,
        SERVICES_SOURCE,
    )];
    let entities = [entity("dynbrain"), entity("dynapps-odoo")];
    let reviews = review_entities(&compute_drift(&declared(), &rules, &entities));
    let dynbrain = find(&reviews, "dynbrain");
    assert_eq!(dynbrain.kind, ReviewKind::RemovedInConsole);
    assert!(!dynbrain.not_live);
    assert_eq!(dynbrain.lines.len(), 1, "{dynbrain:?}");
    assert_eq!(dynbrain.lines[0].subject, "india-devs");
}

#[test]
fn a_disagreeing_row_is_changed_in_code_and_a_console_extra_is_only_in_console() {
    let rules = [
        row("dynbrain", "role", "admin", Access::Deny, SERVICES_SOURCE),
        row(
            "dynbrain",
            "group",
            "india-devs",
            Access::Allow,
            SERVICES_SOURCE,
        ),
        row(
            "dynapps-odoo",
            "role",
            "admin",
            Access::Allow,
            SERVICES_SOURCE,
        ),
        row("agent-farm", "group", "uk", Access::Allow, CONSOLE),
    ];
    let entities = [
        entity("dynbrain"),
        entity("dynapps-odoo"),
        entity("agent-farm"),
    ];
    let reviews = review_entities(&compute_drift(&declared(), &rules, &entities));
    assert_eq!(find(&reviews, "dynbrain").kind, ReviewKind::ChangedInCode);
    let farm = find(&reviews, "agent-farm");
    assert_eq!(farm.kind, ReviewKind::OnlyInConsole);
    assert!(
        !farm.applies,
        "code does not govern agent-farm, so applying it moves nothing"
    );
    assert!(
        reviews.iter().all(|r| r.entity_id != "dynapps-odoo"),
        "dynapps-odoo agrees"
    );
}

#[test]
fn a_bundle_row_on_an_ungoverned_entity_is_never_reviewed() {
    let rules = [
        row("dynbrain", "role", "admin", Access::Allow, SERVICES_SOURCE),
        row(
            "dynbrain",
            "group",
            "india-devs",
            Access::Allow,
            SERVICES_SOURCE,
        ),
        row(
            "dynapps-odoo",
            "role",
            "admin",
            Access::Allow,
            SERVICES_SOURCE,
        ),
        row("kitsrv", "group", "uk", Access::Allow, "bundle:sfnext"),
    ];
    let entities = [entity("dynbrain"), entity("dynapps-odoo")];
    let reviews = review_entities(&compute_drift(&declared(), &rules, &entities));
    assert!(reviews.is_empty(), "{reviews:?}");
}

#[test]
fn a_kept_decision_holds_only_for_the_diff_it_was_taken_on() {
    let reviews = review_entities(&compute_drift(&declared(), &[], &[]));
    let odoo = find(&reviews, "dynapps-odoo").clone();
    let kept = |fingerprint: &str| KeptReview {
        key: odoo.key.clone(),
        fingerprint: fingerprint.to_owned(),
        reason: "retiring it".to_owned(),
        display_name: "Ed".to_owned(),
        created_at: Utc::now(),
    };

    let held = split_reviews(reviews.clone(), &[kept(&odoo.fingerprint)]);
    assert!(held.pending.iter().all(|r| r.review.key != odoo.key));
    assert_eq!(held.kept.len(), 1);

    let moved = split_reviews(reviews, &[kept("an-older-diff")]);
    assert!(moved.pending.iter().any(|r| r.review.key == odoo.key));
    assert!(moved.kept.is_empty());
}

#[test]
fn the_fingerprint_is_stable_and_moves_with_the_diff() {
    let a = review_entities(&compute_drift(&declared(), &[], &[]));
    let b = review_entities(&compute_drift(&declared(), &[], &[]));
    assert_eq!(
        find(&a, "dynapps-odoo").fingerprint,
        find(&b, "dynapps-odoo").fingerprint
    );
    let rules = [row("dynapps-odoo", "group", "uk", Access::Allow, CONSOLE)];
    let c = review_entities(&compute_drift(&declared(), &rules, &[]));
    assert_ne!(
        find(&a, "dynapps-odoo").fingerprint,
        find(&c, "dynapps-odoo").fingerprint
    );
}
