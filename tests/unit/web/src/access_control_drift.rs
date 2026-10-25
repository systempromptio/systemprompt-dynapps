//! The drift engine: code versus database, classified.

use systemprompt::governance::authz::{Access, RegisteredEntities, SERVICES_SOURCE};
use systemprompt_web_admin_core::repositories::access_rules::declared::{
    DeclaredInputs, DeclaredSet, build_declared_set,
};
use systemprompt_web_admin_core::repositories::access_rules::drift::{
    BandRuleRow, EntityDefaultRow, EntityState, OrphanOrigin, compute_drift,
};
use systemprompt_web_admin_core::repositories::access_rules::rules_yaml_loader::parse_rules_doc;

const CONSOLE: &str = "console:ana";

fn declared() -> DeclaredSet {
    let doc = parse_rules_doc(
        "entities:\n  - entity: mcp_server/dynbrain\n    default: closed\n    why: pilot\n    allow:\n      role: [admin]\n      group: [india-devs]\n",
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

#[expect(
    clippy::too_many_arguments,
    reason = "a test row is spelled out in full"
)]
fn row(
    entity_id: &str,
    rule_type: &str,
    value: &str,
    access: Access,
    why: &str,
    source: &str,
) -> BandRuleRow {
    BandRuleRow {
        id: format!("{entity_id}-{rule_type}-{value}"),
        entity_type: "mcp_server".to_owned(),
        entity_id: entity_id.to_owned(),
        rule_type: rule_type.to_owned(),
        rule_value: value.to_owned(),
        access,
        justification: Some(why.to_owned()),
        source: source.to_owned(),
        valid_until: None,
    }
}

fn entity(entity_id: &str, open: bool) -> EntityDefaultRow {
    EntityDefaultRow {
        entity_type: "mcp_server".to_owned(),
        entity_id: entity_id.to_owned(),
        default_included: open,
        source: SERVICES_SOURCE.to_owned(),
    }
}

#[test]
fn identical_state_is_clean() {
    let drift = compute_drift(
        &declared(),
        &[
            row(
                "dynbrain",
                "role",
                "admin",
                Access::Allow,
                "pilot",
                SERVICES_SOURCE,
            ),
            row(
                "dynbrain",
                "group",
                "india-devs",
                Access::Allow,
                "pilot",
                SERVICES_SOURCE,
            ),
        ],
        &[entity("dynbrain", false)],
    );
    assert!(drift.is_clean(), "{drift:?}");
}

#[test]
fn missing_changed_and_orphan_rows_are_each_classified() {
    let drift = compute_drift(
        &declared(),
        &[
            row(
                "dynbrain",
                "role",
                "admin",
                Access::Deny,
                "pilot",
                SERVICES_SOURCE,
            ),
            row("dynbrain", "group", "uk", Access::Allow, "hand", CONSOLE),
            row(
                "dynbrain",
                "project",
                "storefront",
                Access::Allow,
                "old",
                SERVICES_SOURCE,
            ),
            row("github", "group", "uk", Access::Allow, "hand", CONSOLE),
        ],
        &[entity("dynbrain", true)],
    );
    let counts = drift.counts();
    assert_eq!(counts.missing_in_db, 1, "india-devs is declared and absent");
    assert_eq!(counts.changed, 1, "admin flipped to deny");
    assert_eq!(counts.only_in_db, 3);
    assert_eq!(counts.only_in_db_dashboard, 2);
    assert_eq!(counts.only_in_db_code, 1);
    assert_eq!(counts.only_in_db_bundle, 0);
    assert_eq!(
        counts.only_in_db_retire, 2,
        "both dynbrain orphans go: one is governed, one code wrote"
    );
    assert_eq!(
        counts.only_in_db_console_retire, 1,
        "the console row on dynbrain"
    );
    assert_eq!(
        counts.only_in_db_kept, 1,
        "github is not declared and the console wrote it, so it stays"
    );
    assert_eq!(counts.default_changed, 1, "declared closed, database open");
    let github = drift
        .only_in_db
        .iter()
        .find(|o| o.row.entity_id == "github")
        .expect("github orphan");
    assert!(!github.governed);
    assert!(!github.retire);
    assert_eq!(github.origin, OrphanOrigin::Console);
    assert!(drift.touches("mcp_server", "dynbrain"));
    assert!(drift.touches("mcp_server", "github"));
    assert!(!drift.touches("mcp_server", "dynapps-odoo"));
}

#[test]
fn code_written_rows_on_a_dropped_entity_are_retired_but_bundle_rows_are_kept() {
    let drift = compute_drift(
        &declared(),
        &[
            row(
                "dynbrain",
                "role",
                "admin",
                Access::Allow,
                "pilot",
                SERVICES_SOURCE,
            ),
            row(
                "dynbrain",
                "group",
                "india-devs",
                Access::Allow,
                "pilot",
                SERVICES_SOURCE,
            ),
            row(
                "github",
                "group",
                "uk",
                Access::Allow,
                "gone",
                SERVICES_SOURCE,
            ),
            row(
                "github",
                "role",
                "admin",
                Access::Allow,
                "kit",
                "bundle:sfnext",
            ),
        ],
        &[entity("dynbrain", false)],
    );
    let counts = drift.counts();
    assert_eq!(counts.only_in_db, 2);
    assert_eq!(counts.only_in_db_code, 1);
    assert_eq!(counts.only_in_db_bundle, 1);
    assert_eq!(
        counts.only_in_db_retire, 1,
        "code removed github; code takes its row away"
    );
    assert_eq!(
        counts.only_in_db_kept, 1,
        "the bundle's row is not this file's to delete"
    );
    let by_source = |source: &str| {
        drift
            .only_in_db
            .iter()
            .find(|o| o.row.source == source)
            .expect("orphan")
    };
    assert_eq!(by_source(SERVICES_SOURCE).origin, OrphanOrigin::Code);
    assert!(by_source(SERVICES_SOURCE).retire);
    assert_eq!(by_source("bundle:sfnext").origin, OrphanOrigin::Bundle);
    assert!(!by_source("bundle:sfnext").retire);
}

#[test]
fn a_different_justification_alone_is_drift_and_user_rows_are_invisible() {
    let drift = compute_drift(
        &declared(),
        &[
            row(
                "dynbrain",
                "role",
                "admin",
                Access::Allow,
                "something else",
                SERVICES_SOURCE,
            ),
            row(
                "dynbrain",
                "group",
                "india-devs",
                Access::Allow,
                "pilot",
                SERVICES_SOURCE,
            ),
            row("dynbrain", "user", "someone", Access::Deny, "", CONSOLE),
        ],
        &[entity("dynbrain", false)],
    );
    let counts = drift.counts();
    assert_eq!(counts.changed, 1);
    assert_eq!(counts.only_in_db, 0, "the user band never appears");
}

#[test]
fn an_uncatalogued_declared_entity_is_reported() {
    let drift = compute_drift(&declared(), &[], &[]);
    assert_eq!(drift.counts().entities_missing, 1);
    assert_eq!(drift.counts().missing_in_db, 2);
}

// Why: "Unknown" is the page's word for "could not compare", and only that.
// A readable file against an empty database is drift, so an operator is
// never shown Unknown for a database that simply has nothing in it yet.
#[test]
fn unknown_means_the_declaration_could_not_be_compared_and_nothing_else() {
    let empty_db = compute_drift(&declared(), &[], &[]);
    assert_eq!(
        EntityState::for_entity(Some(&empty_db), "mcp_server", "dynbrain"),
        EntityState::Drift
    );
    assert_eq!(
        EntityState::for_entity(None, "mcp_server", "dynbrain"),
        EntityState::Unknown
    );
    let in_step = compute_drift(
        &declared(),
        &[
            row(
                "dynbrain",
                "role",
                "admin",
                Access::Allow,
                "pilot",
                SERVICES_SOURCE,
            ),
            row(
                "dynbrain",
                "group",
                "india-devs",
                Access::Allow,
                "pilot",
                SERVICES_SOURCE,
            ),
        ],
        &[entity("dynbrain", false)],
    );
    assert_eq!(
        EntityState::for_entity(Some(&in_step), "mcp_server", "dynbrain"),
        EntityState::InSync
    );
    assert_eq!(EntityState::Unknown.label(), "Unknown");
    assert_eq!(EntityState::Drift.tone(), "warn");
}
