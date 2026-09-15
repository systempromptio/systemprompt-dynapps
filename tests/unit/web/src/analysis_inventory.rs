//! Retained resource metrics, explicit bindings, canonical grouping and unknown
//! evidence.

use chrono::Utc;
use systemprompt::analytics::snapshots::{FeedbackSnapshot, LatencyHistogram, SnapshotMetrics};
use systemprompt::identifiers::{InventoryEntryId, ManagedResourceId, MarketplaceId, PluginId};
use systemprompt::marketplace::inventory::InventoryEntry;
use systemprompt::models::feedback::inventory::{InventoryAvailability, InventoryOrigin};
use systemprompt_web_admin::repositories::analysis::inventory_index::MarketplaceRef;
use systemprompt_web_admin::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use systemprompt_web_admin::test_support::{
    Health, HealthInputs, PortfolioResourceRow, PortfolioRowInputs, group_rows, health_of, kpis,
};

fn inputs() -> HealthInputs {
    HealthInputs {
        availability: InventoryAvailability::Available,
        bound: true,
        invocations: 5,
        linked_requests: 20,
        failed_requests: 0,
        users_with_access: Some(3),
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
fn quiet_unbound_or_unreachable_skills_warn_in_priority_order() {
    let no_audience = health_of(&HealthInputs {
        users_with_access: Some(0),
        invocations: 0,
        bound: false,
        ..inputs()
    });
    assert_eq!(no_audience, (Health::Warn, "no audience"));
    let unused = health_of(&HealthInputs {
        invocations: 0,
        bound: false,
        ..inputs()
    });
    assert_eq!(unused, (Health::Warn, "unused"));
    let unbound = health_of(&HealthInputs {
        bound: false,
        ..inputs()
    });
    assert_eq!(unbound, (Health::Warn, "unbound"));
}

fn entry(key: &str) -> PortfolioEntry {
    PortfolioEntry {
        entry: InventoryEntry {
            entry_id: InventoryEntryId::new(format!("configured:skill:{key}")),
            kind: "skill".into(),
            resource_key: key.into(),
            origin: InventoryOrigin::Configured,
            configured_key: Some(key.into()),
            resource_id: None,
            source_id: None,
            availability: InventoryAvailability::Available,
            latest_revision_id: None,
            published_revision_id: None,
            diagnostic: None,
        },
        snapshot: None,
        installation: None,
    }
}

fn snapshot(key: &str, invocations: i64) -> FeedbackSnapshot {
    let now = Utc::now();
    let mut histogram = LatencyHistogram::default();
    histogram.record(100_000).unwrap();
    histogram.record(900_000).unwrap();
    FeedbackSnapshot {
        resource_id: Some(ManagedResourceId::new(key)),
        generation: 41,
        fact_generation: 39,
        from_day: now.date_naive() - chrono::Duration::days(29),
        to_day: now.date_naive() + chrono::Duration::days(1),
        generated_at: now,
        metrics: SnapshotMetrics {
            invocations,
            verified_invocations: invocations,
            requests: 2,
            failed_requests: 1,
            priced_requests: 1,
            latency_measured_requests: 2,
            token_measured_requests: 1,
            input_tokens: 17,
            output_tokens: 23,
            ..SnapshotMetrics::default()
        },
        spend_by_currency: [("USD".to_owned(), 1_000_000)].into(),
        distinct_users: Some(1),
        distinct_sessions: Some(1),
        histogram,
        related_spend_non_additive: true,
        suppressed_days: 0,
        historical_identity_available: true,
    }
}
fn row(
    key: &str,
    plugin: (&str, &str),
    marketplaces: &[(&str, &str)],
    invocations: i64,
) -> PortfolioResourceRow {
    let mut item = entry(key);
    item.entry.resource_id = Some(ManagedResourceId::new(key));
    item.snapshot = Some(snapshot(key, invocations));
    PortfolioResourceRow::build(PortfolioRowInputs {
        item,
        plugin: Some(plugin),
        marketplaces: marketplaces
            .iter()
            .map(|(id, name)| MarketplaceRef {
                id: MarketplaceId::new(*id),
                name: (*name).to_owned(),
            })
            .collect(),
        users_with_access: Some(4),
        managed: &[],
        window: ("2026-08-16", "2026-09-16"),
    })
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
    let groups = group_rows(rows);
    let names: Vec<&str> = groups.iter().map(|g| g.marketplace_name.as_str()).collect();
    assert_eq!(names, ["Commons", "India", "Not in any marketplace"]);
    let commons = &groups[0];
    assert_eq!(commons.users_with_access, None);
    assert_eq!(commons.plugins.len(), 2);
    assert_eq!(commons.rollup.skills, 2);
    assert_eq!(commons.rollup.used, 1);
    assert_eq!(commons.rollup.invocations, None);
    assert_eq!(
        commons.rollup.cost,
        "Unavailable for current catalog groups"
    );
    assert_eq!(groups[1].rollup.skills, 1);
    assert_eq!(groups[2].marketplace_id, None);
}

#[test]
fn rollups_count_a_skill_once_per_marketplace_even_when_two_plugins_carry_it() {
    let rows = vec![
        row("shared", ("a", "A"), &[("m", "M")], 2),
        row("shared", ("b", "B"), &[("m", "M")], 2),
    ];
    let groups = group_rows(rows);
    assert_eq!(groups[0].plugins.len(), 2);
    assert_eq!(groups[0].rollup.skills, 1);
    assert_eq!(groups[0].plugins[0].rollup.skills, 1);
}

#[test]
fn organization_kpis_use_independent_snapshot_never_sum_overlapping_resource_costs() {
    let a = row("a", ("p", "P"), &[], 3);
    let b = row("b", ("p", "P"), &[], 4);
    let mut total = snapshot("ignored", 7);
    total.resource_id = None;
    let k = kpis(&[a, b], Some(&total));
    assert_eq!(k.invocations, Some(7));
    assert_eq!(k.users, Some(1));
    assert_eq!(k.linked_requests, Some(2));
    assert_eq!(k.cost, "USD 1000000 micro-units");
    assert_eq!(k.avg_latency, "Unavailable");
    let unavailable = kpis(&[], None);
    assert_eq!(unavailable.invocations, None);
    assert_eq!(unavailable.users, None);
}

#[test]
fn missing_unbound_and_suppressed_evidence_never_become_zero_or_unused() {
    for bound in [false, true] {
        let mut item = entry("quiet");
        if bound {
            item.entry.resource_id = Some(ManagedResourceId::new("quiet"));
        }
        let row = PortfolioResourceRow::build(PortfolioRowInputs {
            item,
            plugin: None,
            marketplaces: vec![],
            users_with_access: Some(10),
            managed: &[],
            window: ("a", "b"),
        });
        assert!(row.usage.is_none());
        assert_eq!(row.health, Health::Muted);
        assert_eq!(row.cost, "Unknown");
        assert_eq!(row.tokens_in, "Unknown");
        assert_eq!(row.last_used, None);
        assert_eq!(kpis(&[row], None).unused, 0);
    }
    let mut item = entry("private");
    item.entry.resource_id = Some(ManagedResourceId::new("private"));
    let mut s = snapshot("private", 10);
    s.suppressed_days = 1;
    item.snapshot = Some(s);
    let row = PortfolioResourceRow::build(PortfolioRowInputs {
        item,
        plugin: None,
        marketplaces: vec![],
        users_with_access: Some(10),
        managed: &[],
        window: ("a", "b"),
    });
    assert!(row.usage.is_none());
    assert_eq!(row.usage_status, "suppressed");
    assert_eq!(row.attributed_pct, None);
}

#[test]
fn retained_values_preserve_watermarks_histogram_coverage_and_unavailable_identities() {
    let row = row("known", ("p", "P"), &[], 5);
    let usage = row.usage.as_ref().unwrap();
    assert_eq!(usage.invocations, 5);
    assert_eq!(usage.failed_requests, 1);
    assert_eq!(usage.latency_measured, 2);
    assert_eq!(usage.priced_requests, 1);
    assert_eq!(row.tokens_in, "17");
    assert_eq!(row.snapshot.as_ref().unwrap().generation, 41);
    assert_eq!(row.snapshot.as_ref().unwrap().fact_generation, 39);
    assert_eq!(row.p95, "1.05 s");
}

#[test]
fn same_name_candidates_require_explicit_binding_and_overflow_is_visible() {
    use systemprompt::identifiers::{ManagedSourceId, ResourceRevisionId};
    use systemprompt::marketplace::managed::ResourceSummary;
    let candidates: Vec<_> = (0..51)
        .map(|i| ResourceSummary {
            id: ManagedResourceId::new(format!("resource-{i}")),
            source_id: ManagedSourceId::new(format!("source-{i}")),
            source_name: format!("Source {i}"),
            kind: "skill".to_owned(),
            resource_key: "shared".to_owned(),
            revision_count: 1,
            latest_revision: Some(ResourceRevisionId::new(format!("revision-{i}"))),
        })
        .collect();
    let unbound = PortfolioResourceRow::build(PortfolioRowInputs {
        item: entry("shared"),
        plugin: None,
        marketplaces: vec![],
        users_with_access: Some(4),
        managed: &candidates,
        window: ("a", "b"),
    });
    assert!(unbound.campaign_href.is_none());
    assert_eq!(unbound.bind_candidates.len(), 50);
    assert!(unbound.more_bind_candidates);
    assert!(unbound.usage.is_none());
    let mut item = entry("shared");
    item.entry.resource_id = Some(candidates[1].id.clone());
    let bound = PortfolioResourceRow::build(PortfolioRowInputs {
        item,
        plugin: None,
        marketplaces: vec![],
        users_with_access: Some(4),
        managed: &candidates,
        window: ("a", "b"),
    });
    assert_eq!(
        bound.campaign_href.as_deref(),
        Some("/admin/analysis/campaigns?baseline=resource-1|revision-1")
    );
    assert!(bound.bind_candidates.is_empty());
}

#[test]
fn older_identity_and_unknown_pricing_remain_unavailable_while_counts_survive() {
    let mut item = entry("historical");
    item.entry.resource_id = Some(ManagedResourceId::new("historical"));
    let mut s = snapshot("historical", 5);
    s.distinct_users = None;
    s.distinct_sessions = None;
    s.historical_identity_available = false;
    s.metrics.priced_requests = 0;
    s.spend_by_currency.clear();
    item.snapshot = Some(s);
    let row = PortfolioResourceRow::build(PortfolioRowInputs {
        item,
        plugin: None,
        marketplaces: vec![],
        users_with_access: Some(4),
        managed: &[],
        window: ("a", "b"),
    });
    let usage = row.usage.as_ref().unwrap();
    assert_eq!(usage.invocations, 5);
    assert_eq!(usage.distinct_users, None);
    assert_eq!(usage.conversations, None);
    assert_eq!(row.cost, "Unknown");
}

#[test]
fn unrelated_or_unbound_snapshot_cannot_supply_usage_by_matching_name() {
    for bound in [false, true] {
        let mut item = entry("shared");
        if bound {
            item.entry.resource_id = Some(ManagedResourceId::new("actual"));
        }
        item.snapshot = Some(snapshot("other", 99));
        let row = PortfolioResourceRow::build(PortfolioRowInputs {
            item,
            plugin: None,
            marketplaces: vec![],
            users_with_access: None,
            managed: &[],
            window: ("a", "b"),
        });
        assert!(row.snapshot.is_none());
        assert!(row.usage.is_none());
        assert_eq!(row.cost, "Unknown");
    }
}

#[test]
fn current_inventory_template_renders_unknown_evidence_and_retains_actions_and_cursor() {
    use std::path::Path;
    fn partials(hbs: &mut handlebars::Handlebars<'_>, root: &Path, dir: &Path) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                partials(hbs, root, &path);
            } else if path.extension().is_some_and(|ext| ext == "hbs") {
                let name = path.strip_prefix(root).unwrap().with_extension("");
                hbs.register_partial(
                    name.to_str().unwrap(),
                    std::fs::read_to_string(&path).unwrap(),
                )
                .unwrap();
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("storage/files/admin").is_dir())
        .unwrap();
    let admin = root.join("storage/files/admin");
    let mut hbs = handlebars::Handlebars::new();
    systemprompt_web_admin::templates::helpers::register_helpers(&mut hbs);
    partials(&mut hbs, &admin.join("partials"), &admin.join("partials"));
    let unknown = PortfolioResourceRow::build(PortfolioRowInputs {
        item: entry("unknown-evidence"),
        plugin: None,
        marketplaces: vec![],
        users_with_access: None,
        managed: &[],
        window: ("a", "b"),
    });
    let known = row("retained-evidence", ("p", "P"), &[], 404);
    let groups = group_rows(vec![unknown, known]);
    let html = hbs.render_template(&std::fs::read_to_string(admin.join("templates/analysis-portfolio.hbs")).unwrap(), &serde_json::json!({
        "page": "analysis-portfolio", "title": "Resource effectiveness", "groups": groups,
        "health": { "generation": 41, "fact_generation": 39, "facts_generation": 42, "pending_changes": 3, "pending_producer_changes": 2 },
        "inventory": { "generation": 2, "entries": 53 }, "installations": { "generation": 3 },
        "kpis": kpis(&[], None), "days": 30, "kind": "skills", "operation_key": "test-operation",
        "next_cursor": "entry-50", "next_href": "/admin/analysis/resources?after=entry-50",
        "generated_at": "2026-09-15T10:00:00Z", "pending_jobs": 0,
    })).unwrap();
    let unknown_row = html
        .split("data-skill=\"unknown-evidence\"")
        .nth(1)
        .unwrap()
        .split("</tr>")
        .next()
        .unwrap();
    assert!(unknown_row.contains("Unknown"));
    assert!(!unknown_row.contains("never in window"));
    assert!(unknown_row.contains("Installation coverage unknown"));
    let retained_row = html
        .split("data-skill=\"retained-evidence\"")
        .nth(1)
        .unwrap()
        .split("</tr>")
        .next()
        .unwrap();
    assert!(retained_row.contains("404"));
    assert!(retained_row.contains("2/2 latency measured"));
    assert!(html.contains("data-next-cursor=\"entry-50\""));
    assert!(html.contains("Prepare baselines"));
    assert!(html.contains("pending facts 3"));
    assert!(!html.contains("Live from observed events"));
}

#[test]
fn catalog_plugin_conversation_action_keeps_native_usage_key_without_relabeling_metrics() {
    assert_eq!(
        systemprompt_web_admin::test_support::usage_key(&PluginId::new("plugin"), "my_skill"),
        "plugin:my-skill"
    );
    let row = row("my_skill", ("plugin", "Plugin"), &[], 5);
    assert!(row.drilldown_href.contains("plugin%3Amy-skill"));
    assert_eq!(
        row.snapshot.unwrap().resource_id.unwrap().as_str(),
        "my_skill"
    );
}

#[test]
fn valid_maximum_counts_preserve_failure_threshold_and_attribution_percentage() {
    let mut health = inputs();
    health.invocations = i64::MAX;
    health.linked_requests = i64::MAX;
    health.failed_requests = i64::MAX / 10;
    assert_eq!(health_of(&health), (Health::Ok, "healthy"));
    health.failed_requests += 1;
    assert_eq!(health_of(&health), (Health::Err, "failing requests"));
    for (verified, expected) in [
        (i64::MAX, Some(100)),
        (i64::MAX / 2, Some(49)),
        (0, Some(0)),
    ] {
        let mut item = entry("large");
        item.entry.resource_id = Some(ManagedResourceId::new("large"));
        let mut s = snapshot("large", i64::MAX);
        s.metrics.verified_invocations = verified;
        item.snapshot = Some(s);
        let row = PortfolioResourceRow::build(PortfolioRowInputs {
            item,
            plugin: None,
            marketplaces: vec![],
            users_with_access: None,
            managed: &[],
            window: ("a", "b"),
        });
        assert_eq!(row.attributed_pct, expected);
    }
}

#[tokio::test]
async fn inventory_styles_are_discovered_by_the_actual_admin_bundle_collector() {
    use std::path::Path;
    use systemprompt_web_extension::jobs::internals::{collect_css_files, concatenate_css_files};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("storage/files/css/admin").is_dir())
        .unwrap();
    let files = collect_css_files(&root.join("storage/files/css/admin"))
        .await
        .unwrap();
    let inventory = files
        .iter()
        .find(|p| {
            p.file_name()
                .is_some_and(|n| n == "74-analysis-inventory.css")
        })
        .expect("directory discovery includes the new stylesheet");
    let (bundle, count, failures) = concatenate_css_files(std::slice::from_ref(inventory)).await;
    assert_eq!(count, 1);
    assert_eq!(failures, 0);
    assert!(bundle.contains(".sp-inventory-table"));
}
