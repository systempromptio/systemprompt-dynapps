//! Whole-page SSR serialization keeps exact display values beyond JSON Value's
//! integer range.
use super::super::rows::{PortfolioResourceRow, PortfolioRowInputs};
use super::super::{PortfolioPage, groups};
use super::HtmlPortfolioPage;
use crate::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use chrono::Utc;
use systemprompt::analytics::snapshots::{
    FeedbackSnapshot, LatencyHistogram, SnapshotHealth, SnapshotMetrics, SnapshotRangeJob,
};
use systemprompt::identifiers::{InventoryEntryId, ManagedResourceId, TaskId};
use systemprompt::marketplace::inventory::{
    InstallationCoverage, InstallationCoverageStatus, InventoryEntry, InventoryStatus,
};
use systemprompt::models::feedback::inventory::{InventoryAvailability, InventoryOrigin};

fn retained_snapshot() -> FeedbackSnapshot {
    let now = Utc::now();
    let wide = i128::from(u64::MAX) + 777;
    let snapshot = FeedbackSnapshot {
        resource_id: Some(ManagedResourceId::new("wide-resource")),
        generation: 19,
        fact_generation: 17,
        from_day: now.date_naive() - chrono::Duration::days(29),
        to_day: now.date_naive(),
        generated_at: now,
        metrics: SnapshotMetrics {
            invocations: 2,
            verified_invocations: 1,
            requests: 1,
            priced_requests: 1,
            token_measured_requests: 1,
            input_tokens: wide,
            output_tokens: wide + 1,
            ..SnapshotMetrics::default()
        },
        spend_by_currency: [("USD".to_owned(), wide + 2)].into(),
        distinct_users: Some(1),
        distinct_sessions: Some(1),
        histogram: LatencyHistogram::default(),
        related_spend_non_additive: true,
        suppressed_days: 0,
        historical_identity_available: true,
    };
    snapshot
}

fn resource(snapshot: &FeedbackSnapshot) -> PortfolioResourceRow {
    PortfolioResourceRow::build(PortfolioRowInputs {
        item: PortfolioEntry {
            entry: InventoryEntry {
                entry_id: InventoryEntryId::new("wide-entry"),
                kind: "skill".to_owned(),
                resource_key: "wide-resource".to_owned(),
                origin: InventoryOrigin::Configured,
                configured_key: Some("wide-resource".to_owned()),
                resource_id: snapshot.resource_id.clone(),
                source_id: None,
                availability: InventoryAvailability::Available,
                latest_revision_id: None,
                published_revision_id: None,
                diagnostic: None,
            },
            snapshot: Some(snapshot.clone()),
            installation: Some(InstallationCoverage {
                eligible_devices: 5,
                current_acknowledged_devices: 2,
                current_verified_devices: 1,
                acknowledged_installations: 2,
                unverifiable_installations: 1,
                legacy_receipts: 0,
            }),
        },
        plugin: None,
        marketplaces: vec![],
        users_with_access: None,
        managed: &[],
        window: ("a", "b"),
    })
}

fn page() -> PortfolioPage {
    let now = Utc::now();
    let snapshot = retained_snapshot();
    let resource = resource(&snapshot);
    let mut totals = snapshot.clone();
    totals.resource_id = None;
    let kpis = groups::kpis(std::slice::from_ref(&resource), Some(&totals));
    PortfolioPage {
        page: "analysis-portfolio",
        title: "Resource effectiveness",
        kind: "skills".to_owned(),
        groups: groups::group_rows(vec![resource]),
        kpis,
        marketplaces: vec![],
        user_groups: vec![],
        filter_marketplace: String::new(),
        filter_group: String::new(),
        start: snapshot.from_day.to_string(),
        end: snapshot.to_day.to_string(),
        search: String::new(),
        days: 30,
        generated_at: now.to_rfc3339(),
        totals: Some(totals),
        health: SnapshotHealth {
            generation: 19,
            fact_generation: 17,
            generated_at: Some(now),
            last_error: None,
            pending_changes: 2,
            pending_producer_changes: 1,
            facts_generation: 20,
            pending_jobs: 0,
        },
        inventory: InventoryStatus {
            generation: 3,
            observed_at: Some(now),
            entries: 51,
            last_error: None,
        },
        installations: InstallationCoverageStatus {
            generation: 4,
            observed_at: Some(now),
        },
        pending_jobs: 0,
        range_jobs: vec![SnapshotRangeJob {
            operation_id: TaskId::new("wide-range"),
            state: "completed".to_owned(),
            result: Some(snapshot),
            diagnostic: None,
        }],
        unknown_usage: Some(1),
        unbound: 0,
        conflicting: 0,
        operation_key: "wide-page".to_owned(),
        next_cursor: Some(InventoryEntryId::new("next-entry")),
        next_href: Some("/admin/analysis/resources?after=next-entry".to_owned()),
        metrics_notice: "Retained snapshots".to_owned(),
    }
}

#[test]
fn whole_page_html_preserves_wide_amounts_tokens_metadata_and_operation_status() {
    let page = page();
    let wide = i128::from(u64::MAX) + 777;
    let html = serde_json::to_value(HtmlPortfolioPage(&page))
        .expect("HTML page never serializes raw i128 aggregates");
    let row = &html["groups"][0]["plugins"][0]["rows"][0];
    assert_eq!(row["tokens_in"], wide.to_string());
    assert_eq!(row["tokens_out"], (wide + 1).to_string());
    assert_eq!(row["cost"], format!("USD {} micro-units", wide + 2));
    assert_eq!(html["kpis"]["cost"], row["cost"]);
    assert_eq!(row["snapshot"]["generation"], 19);
    assert_eq!(row["snapshot"]["fact_generation"], 17);
    assert_eq!(row["snapshot"]["from_day"], page.start);
    assert_eq!(html["totals"]["to_day"], page.end);
    assert_eq!(row["installation"]["eligible_devices"], 5);
    assert_eq!(html["health"]["pending_changes"], 2);
    assert_eq!(html["range_jobs"][0]["operation_id"], "wide-range");
    assert_eq!(html["range_jobs"][0]["state"], "completed");
    assert_eq!(html["range_jobs"][0]["result"]["generation"], 19);
    assert_eq!(html["next_cursor"], "next-entry");
    for metadata in [
        &row["snapshot"],
        &html["totals"],
        &html["range_jobs"][0]["result"],
    ] {
        assert!(metadata.get("metrics").is_none());
        assert!(metadata.get("spend_by_currency").is_none());
    }
    let original_json =
        serde_json::to_string(&page).expect("raw JSON serializer supports full i128 numbers");
    assert!(original_json.contains(&format!("\"input_tokens\":{wide}")));
    assert!(original_json.contains(&format!("\"USD\":{}", wide + 2)));
    assert_eq!(
        page.totals.as_ref().unwrap().metrics.input_tokens,
        wide,
        "HTML borrowing does not mutate the JSON response model"
    );
}

#[test]
fn html_preserves_absence_and_failed_custom_range_diagnostics() {
    let mut page = page();
    page.totals = None;
    page.groups[0].plugins[0].rows[0].snapshot = None;
    page.range_jobs[0].result = None;
    page.range_jobs[0].state = "failed".to_owned();
    page.range_jobs[0].diagnostic = Some("Range data is unavailable".to_owned());
    let html = serde_json::to_value(HtmlPortfolioPage(&page)).unwrap();
    assert!(html["totals"].is_null());
    assert!(html["groups"][0]["plugins"][0]["rows"][0]["snapshot"].is_null());
    assert!(html["range_jobs"][0]["result"].is_null());
    assert_eq!(
        html["range_jobs"][0]["diagnostic"],
        "Range data is unavailable"
    );
}
