//! The pure halves of the Skills table: the health verdict, the retained
//! drill-down link, marketplace → plugin grouping with distinct-skill rollups
//! and the version chip each group carries, snapshot figures, the measured
//! p95 over the histogram bound, the unresolved names an admin has to bind,
//! and audience union across marketplaces.

use std::collections::{HashMap, HashSet};

use systemprompt::analytics::snapshots::{FeedbackSnapshot, LatencyHistogram, SnapshotMetrics};
use systemprompt::identifiers::{
    InventoryEntryId, ManagedResourceId, MarketplaceId, PluginId, UserId,
};
use systemprompt::marketplace::inventory::InventoryEntry;
use systemprompt::models::feedback::inventory::{InventoryAvailability, InventoryOrigin};
use systemprompt_web_admin::repositories::analysis::inventory_index::{
    InventoryIndex, MarketplaceAudience, MarketplaceRef,
};
use systemprompt_web_admin::repositories::analysis::marketplace_versions::SkillLiveRow;
use systemprompt_web_admin::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use systemprompt_web_admin::repositories::analysis::snapshot_figures::SnapshotFigures;
use systemprompt_web_admin::repositories::sync::marketplace_versions_db::MarketplaceVersionRow;
use systemprompt_web_admin::test_support::{
    Health, HealthInputs, InventoryRow, InventoryRowInputs, RowVersion, UNPLACED, collisions,
    group_rows, health_of, in_marketplace, kpis, live_for, p95_cell, short_hash,
    tally_marketplaces, version_href,
};

fn inputs() -> HealthInputs {
    HealthInputs {
        availability: InventoryAvailability::Available,
        bound: true,
        measured: true,
        invocations: 5,
        requests: 20,
        failed_requests: 0,
        users_with_access: 3,
    }
}

#[test]
fn healthy_skill_is_ok() {
    assert_eq!(health_of(&inputs()), (Health::Ok, "healthy"));
}

#[test]
fn availability_faults_are_errors_before_anything_else() {
    for (availability, reason) in [
        (InventoryAvailability::Conflicting, "conflicting name"),
        (InventoryAvailability::Unavailable, "unavailable"),
        (InventoryAvailability::Withdrawn, "withdrawn"),
    ] {
        let verdict = health_of(&HealthInputs {
            availability,
            invocations: 0,
            ..inputs()
        });
        assert_eq!(verdict, (Health::Err, reason));
    }
}

#[test]
fn more_than_ten_percent_failed_requests_is_an_error() {
    let verdict = health_of(&HealthInputs {
        failed_requests: 3,
        ..inputs()
    });
    assert_eq!(verdict, (Health::Err, "failing requests"));
    let verdict = health_of(&HealthInputs {
        failed_requests: 2,
        ..inputs()
    });
    assert_eq!(verdict.0, Health::Ok);
}

#[test]
fn unreachable_unbound_pending_and_quiet_skills_in_priority_order() {
    let no_audience = health_of(&HealthInputs {
        users_with_access: 0,
        invocations: 0,
        bound: false,
        ..inputs()
    });
    assert_eq!(no_audience, (Health::Warn, "no audience"));
    let unbound = health_of(&HealthInputs {
        bound: false,
        measured: false,
        invocations: 0,
        ..inputs()
    });
    assert_eq!(unbound, (Health::Warn, "unbound"));
    let pending = health_of(&HealthInputs {
        measured: false,
        invocations: 0,
        ..inputs()
    });
    assert_eq!(pending, (Health::Muted, "snapshot pending"));
    let quiet = health_of(&HealthInputs {
        invocations: 0,
        ..inputs()
    });
    assert_eq!(quiet, (Health::Warn, "no attributed use"));
}

fn snapshot(
    scope: Option<&str>,
    invocations: i64,
    requests: i64,
    cost_micros: i128,
) -> FeedbackSnapshot {
    FeedbackSnapshot {
        resource_id: scope.map(ManagedResourceId::new),
        generation: 1,
        fact_generation: 1,
        from_day: chrono::NaiveDate::from_ymd_opt(2026, 8, 16).unwrap_or_default(),
        to_day: chrono::NaiveDate::from_ymd_opt(2026, 9, 16).unwrap_or_default(),
        generated_at: chrono::Utc::now(),
        metrics: SnapshotMetrics {
            invocations,
            verified_invocations: invocations,
            requests,
            failed_requests: 0,
            priced_requests: requests,
            latency_measured_requests: requests,
            token_measured_requests: requests,
            input_tokens: 1_000,
            output_tokens: 200,
            assessed_conversations: 0,
            assessment_conversations: 0,
            failed_assessments: 0,
        },
        spend_by_currency: [("USD".to_owned(), cost_micros)].into(),
        distinct_users: Some(2),
        distinct_sessions: Some(3),
        histogram: LatencyHistogram::default(),
        related_spend_non_additive: true,
        historical_identity_available: true,
        suppressed_days: 0,
    }
}

fn entry(key: &str, invocations: i64) -> PortfolioEntry {
    let bound = invocations >= 0;
    PortfolioEntry {
        entry: InventoryEntry {
            entry_id: InventoryEntryId::new(format!("configured:skill:{key}")),
            kind: "skill".into(),
            resource_key: key.into(),
            origin: InventoryOrigin::Configured,
            configured_key: Some(key.into()),
            resource_id: bound.then(|| ManagedResourceId::new(format!("res-{key}"))),
            source_id: None,
            availability: InventoryAvailability::Available,
            latest_revision_id: None,
            published_revision_id: None,
            diagnostic: None,
        },
        snapshot: bound.then(|| {
            snapshot(
                Some(&format!("res-{key}")),
                invocations,
                invocations * 2,
                i128::from(invocations) * 1_000_000,
            )
        }),
        installation: None,
    }
}

fn row(
    key: &str,
    plugin: (&str, &str),
    marketplaces: &[(&str, &str)],
    invocations: i64,
) -> InventoryRow {
    InventoryRow::build(InventoryRowInputs {
        item: entry(key, invocations),
        plugin: Some(plugin),
        marketplaces: marketplaces
            .iter()
            .map(|(id, name)| MarketplaceRef {
                id: (*id).to_owned(),
                name: (*name).to_owned(),
            })
            .collect(),
        users_with_access: 4,
        managed: &[],
        live: None,
        window: ("2026-08-16", "2026-09-16"),
    })
    .unwrap_or_else(|error| panic!("row build failed: {error}"))
}

#[test]
fn snapshot_figures_read_the_retained_metrics_verbatim() {
    let figures = SnapshotFigures::of(&snapshot(None, 4, 8, 2_500_000))
        .unwrap_or_else(|error| panic!("figures failed: {error}"))
        .unwrap_or_default();
    assert!(figures.measured);
    assert_eq!(figures.invocations, 4);
    assert_eq!(figures.attributed_pct, Some(100));
    assert_eq!(figures.requests, 8);
    assert_eq!(figures.cost_microdollars, 2_500_000);
    assert_eq!(figures.input_tokens, 1_000);
    assert_eq!(figures.distinct_users, Some(2));
    assert!(!SnapshotFigures::default().measured);
}

#[test]
fn a_suppressed_snapshot_is_treated_as_absent() {
    let mut suppressed = snapshot(None, 4, 8, 1);
    suppressed.suppressed_days = 2;
    let figures = SnapshotFigures::of(&suppressed).unwrap_or_else(|error| panic!("{error}"));
    assert!(figures.is_none());
}

#[test]
fn an_unbound_entry_is_unmeasured_not_zero() {
    let unmeasured = row("silent", ("a", "A"), &[("m", "M")], -1);
    assert!(!unmeasured.figures.measured);
    assert_eq!(unmeasured.usage_status, "unbound");
    assert_eq!(unmeasured.cost, "—");
    assert_eq!(unmeasured.health, Health::Warn);
    assert_eq!(
        unmeasured.drilldown_href,
        "/admin/analysis/skills?skill=silent&start=2026-08-16&end=2026-09-16",
        "an unbound entry still links to its conversations, by skill identity"
    );
}

#[test]
fn a_bound_entry_reads_its_own_snapshot() {
    let measured = row("busy", ("a", "A"), &[("m", "M")], 3);
    assert!(measured.figures.measured);
    assert_eq!(measured.usage_status, "measured");
    assert_eq!(measured.cost, "$3.00");
    assert_eq!(measured.health, Health::Ok);
    assert_eq!(
        measured.drilldown_href,
        "/admin/analysis/skills?resource=res-busy&start=2026-08-16&end=2026-09-16",
        "a bound entry links by its revision-keyed managed resource"
    );
}

#[test]
fn rows_group_by_marketplace_then_plugin_and_repeat_across_audiences() {
    let rows = vec![
        row(
            "who_am_i",
            ("astound-commons", "Commons"),
            &[("commons", "Commons"), ("india", "India")],
            3,
        ),
        row(
            "code_review",
            ("astound-dev", "Dev"),
            &[("commons", "Commons")],
            0,
        ),
        row("orphan", ("x", "X"), &[], 1),
    ];
    let groups = group_rows(rows, |id| if id == "commons" { 10 } else { 2 }, &[]);
    let names: Vec<&str> = groups.iter().map(|g| g.marketplace_name.as_str()).collect();
    assert_eq!(names, ["Commons", "India", "Not in any marketplace"]);
    let commons = &groups[0];
    assert_eq!(commons.users_with_access, 10);
    assert_eq!(commons.plugins.len(), 2);
    assert_eq!(commons.rollup.skills, 2);
    assert_eq!(commons.rollup.measured, 2);
    assert_eq!(commons.rollup.used, 1);
    assert_eq!(commons.rollup.invocations, 3);
    assert_eq!(commons.rollup.cost, "$3.00");
    assert_eq!(groups[1].rollup.skills, 1);
    assert_eq!(groups[2].marketplace_id.as_str(), UNPLACED);
}

#[test]
fn rollups_count_a_skill_once_per_marketplace_even_when_two_plugins_carry_it() {
    let rows = vec![
        row("shared", ("a", "A"), &[("m", "M")], 2),
        row("shared", ("b", "B"), &[("m", "M")], 2),
    ];
    let groups = group_rows(rows, |_| 1, &[]);
    assert_eq!(groups[0].plugins.len(), 2);
    assert_eq!(groups[0].rollup.skills, 1);
    assert_eq!(groups[0].plugins[0].rollup.skills, 1);
}

#[test]
fn kpis_count_distinct_skills_and_take_totals_from_the_organisation_snapshot() {
    let rows = [
        row("shared", ("a", "A"), &[("m", "M")], 1),
        row("shared", ("b", "B"), &[("m", "M")], 1),
        row("quiet", ("a", "A"), &[("m", "M")], 0),
    ];
    let totals = SnapshotFigures::of(&snapshot(None, 10, 154, 1_533_315))
        .unwrap_or_else(|error| panic!("figures failed: {error}"))
        .unwrap_or_default();
    let k = kpis(&rows, Some(&totals));
    assert_eq!(k.skills, 2);
    assert_eq!(k.measured, 2);
    assert_eq!(k.used, 1);
    assert_eq!(k.unused, 1);
    assert_eq!(k.invocations, 10);
    assert_eq!(k.requests, 154);
    assert_eq!(k.users, 2);
    assert_eq!(k.cost, "$1.53");
    assert_eq!(kpis(&rows, None).cost, "—");
}

#[test]
fn audience_unions_users_across_every_marketplace_reaching_a_skill() {
    let mut index = InventoryIndex::default();
    index.plugins_by_skill.insert(
        "who_am_i".into(),
        vec![
            std::sync::Arc::new(("p1".into(), "P1".into())),
            std::sync::Arc::new(("p2".into(), "P2".into())),
        ],
    );
    index.marketplaces_by_plugin.insert(
        "p1".into(),
        vec![MarketplaceRef {
            id: "m1".into(),
            name: "M1".into(),
        }],
    );
    index.marketplaces_by_plugin.insert(
        "p2".into(),
        vec![MarketplaceRef {
            id: "m2".into(),
            name: "M2".into(),
        }],
    );
    let mut users_by_marketplace: HashMap<String, HashSet<UserId>> = HashMap::new();
    users_by_marketplace.insert("m1".into(), [UserId::new("u1"), UserId::new("u2")].into());
    users_by_marketplace.insert("m2".into(), [UserId::new("u2"), UserId::new("u3")].into());
    let audience = MarketplaceAudience {
        users_by_marketplace,
    };
    assert_eq!(audience.users_reaching_skill(&index, "who_am_i"), 3);
    assert_eq!(audience.users_reaching_skill(&index, "unknown"), 0);
    assert_eq!(audience.users_reaching_marketplace("m1"), 2);
}

fn version(marketplace: &str, hash: &str) -> MarketplaceVersionRow {
    MarketplaceVersionRow {
        marketplace_id: MarketplaceId::new(marketplace),
        content_hash: hash.to_owned(),
        source: "base".to_owned(),
        source_hash: None,
        manifest: None,
        plugin_count: 1,
        skill_count: 1,
        first_seen_at: chrono::Utc::now(),
        last_seen_at: chrono::Utc::now(),
        effective_until: None,
        origin: "manifest".to_owned(),
    }
}

#[test]
fn every_row_and_caption_carries_the_version_of_the_marketplace_it_sits_under() {
    let rows = vec![row(
        "who_am_i",
        ("astound-commons", "Commons"),
        &[("commons", "Commons"), ("india", "India")],
        3,
    )];
    let versions = [
        version("commons", "aaaaaaaaaaaabbbbbbbbbbbbcccccccccccc"),
        version("india", "dddddddddddd111111111111222222222222"),
    ];
    let groups = group_rows(rows, |_| 1, &versions);
    let chip = |g: &systemprompt_web_admin::test_support::MarketplaceGroup| {
        g.version.clone().map(|v| v.short_hash)
    };
    assert_eq!(chip(&groups[0]).as_deref(), Some("aaaaaaaaaaaa"));
    assert_eq!(chip(&groups[1]).as_deref(), Some("dddddddddddd"));
    let placed = &groups[1].plugins[0].rows[0];
    assert_eq!(
        placed.version,
        Some(RowVersion {
            marketplace_id: MarketplaceId::new("india"),
            content_hash: "dddddddddddd111111111111222222222222".into(),
            short_hash: "dddddddddddd".into(),
            href: "/admin/analysis/versions/india#v-dddddddddddd".into(),
        }),
        "a row under a marketplace names that marketplace's version, not another's"
    );
    assert_eq!(short_hash("abc"), "abc");
    assert_eq!(
        version_href(&MarketplaceId::new("kit one"), "ffffffffffffffff"),
        "/admin/analysis/versions/kit%20one#v-ffffffffffff"
    );
}

fn live(plugin: &str, skill: &str, users: i64, p95_micros: Option<i64>) -> SkillLiveRow {
    SkillLiveRow {
        plugin_id: PluginId::new(plugin),
        skill: skill.to_owned(),
        users,
        p50_micros: p95_micros.map(|m| m / 2),
        p95_micros,
        latency_measured: i64::from(p95_micros.is_some()),
    }
}

#[test]
fn live_figures_are_keyed_the_way_facts_key_a_skill() {
    let rows = [live(
        "astound-ba",
        "astound-ba:jira-management",
        8,
        Some(2_400_000),
    )];
    let hit = live_for(&rows, Some("astound-ba"), "jira_management");
    assert_eq!(
        hit.map(|l| l.users),
        Some(8),
        "underscored config ids match dashed fact keys"
    );
    assert!(live_for(&rows, Some("other"), "jira_management").is_none());
    assert!(live_for(&rows, None, "jira_management").is_none());
}

#[test]
fn p95_is_measured_when_request_latency_exists_and_a_bound_otherwise() {
    let measured = live("p", "p:s", 1, Some(2_400_000));
    assert_eq!(
        p95_cell(Some(&measured), Some(16_777_215)),
        ("2.40 s".to_owned(), true)
    );
    let unmeasured = live("p", "p:s", 1, None);
    let (text, real) = p95_cell(Some(&unmeasured), Some(16_777_215));
    assert!(!real);
    assert!(
        text.starts_with("≤ "),
        "a histogram bound is marked as a bound: {text}"
    );
    assert_eq!(p95_cell(None, None), ("—".to_owned(), false));
}

#[test]
fn a_row_prefers_the_live_user_count_and_falls_back_to_the_snapshot() {
    let with_live = InventoryRow::build(InventoryRowInputs {
        item: entry("busy", 3),
        plugin: Some(("a", "A")),
        marketplaces: Vec::new(),
        users_with_access: 4,
        managed: &[],
        live: Some(&live("a", "a:busy", 7, None)),
        window: ("2026-08-16", "2026-09-16"),
    })
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(with_live.users, Some(7));
    let without = row("busy", ("a", "A"), &[], 3);
    assert_eq!(
        without.users,
        Some(2),
        "the snapshot's distinct users stand in"
    );
}

#[test]
fn chip_tallies_count_a_skill_once_per_marketplace_and_gather_the_unplaced() {
    let rows = [
        row("shared", ("a", "A"), &[("m", "M"), ("n", "N")], 5),
        row("shared", ("b", "B"), &[("m", "M")], 5),
        row("quiet", ("a", "A"), &[("m", "M")], 0),
        row("orphan", ("a", "A"), &[], 7),
    ];
    let tallies = tally_marketplaces(&rows);
    let named: Vec<(&str, usize, i64)> = tallies
        .iter()
        .map(|t| (t.id.as_str(), t.skills, t.invocations))
        .collect();
    assert_eq!(
        named,
        [("m", 2, 5), ("n", 1, 5), (UNPLACED, 1, 7)],
        "tallies are in name order, count distinct skills, and place rows in no marketplace under {UNPLACED}"
    );
}

#[test]
fn the_single_marketplace_view_can_still_reach_rows_in_no_marketplace() {
    let placed = row("a", ("p", "P"), &[("m", "M")], 1);
    let orphan = row("b", ("p", "P"), &[], 1);
    assert!(in_marketplace(&placed, "m"));
    assert!(!in_marketplace(&placed, "n"));
    assert!(!in_marketplace(&placed, UNPLACED));
    assert!(in_marketplace(&orphan, UNPLACED));
    assert!(!in_marketplace(&orphan, "m"));
}

#[test]
fn collisions_are_listed_once_per_entry_not_once_per_row() {
    let rows = [
        row("shared", ("a", "A"), &[("m", "M")], -1),
        row("shared", ("b", "B"), &[("m", "M")], -1),
        row("measured", ("a", "A"), &[("m", "M")], 3),
    ];
    assert!(
        collisions(&rows).is_empty(),
        "an unbound entry with no candidate to bind to is nothing an admin can resolve"
    );
}
