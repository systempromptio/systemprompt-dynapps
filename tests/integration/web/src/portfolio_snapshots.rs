//! Source/reference parity is required before removing the old event-query
//! implementation.

use crate::feedback_ingestion::{drain, seed};
use crate::tempdb::TempDb;
use chrono::Utc;
use systemprompt::analytics::resource_metrics::aggregate;
use systemprompt::analytics::snapshots::FeedbackSnapshotsRepository;
use systemprompt::identifiers::{TaskId, UserId};
use systemprompt::marketplace::inventory::ConfiguredInventoryEntry;
use systemprompt::marketplace::managed::ManagedRepository;
use systemprompt::models::feedback::inventory::InventoryAvailability;
use systemprompt_web_admin::repositories::analysis::portfolio::list_portfolio_facts;
use systemprompt_web_admin::repositories::analysis::portfolio_snapshots::{
    PortfolioFilter, PortfolioSnapshotsRepository,
};

#[tokio::test]
async fn legacy_additive_reference_matches_retained_snapshots_and_inventory_keeps_unused_entries() {
    let db = TempDb::create()
        .await
        .expect("database required for portfolio acceptance");
    let pool = db.pool.as_ref();
    seed(pool).await;
    drain(pool).await;
    let owner = UserId::new("publisher");
    let now = Utc::now();
    let snapshots = FeedbackSnapshotsRepository::new(pool.clone());
    while snapshots
        .process(&owner, &TaskId::generate(), now)
        .await
        .unwrap()
        > 0
    {}
    snapshots.refresh(&owner, &[], now).await.unwrap();
    let retained = snapshots.snapshot(&owner, None, 30).await.unwrap().unwrap();
    let start = retained.from_day.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = retained.to_day.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let facts = list_portfolio_facts(pool, start, end).await.unwrap();
    let legacy = aggregate(facts.iter().map(|row| &row.fact));
    assert_eq!(legacy.invocations as i64, retained.metrics.invocations);
    assert_eq!(legacy.requests as i64, retained.metrics.requests);
    assert_eq!(
        legacy.failed_requests as i64,
        retained.metrics.failed_requests
    );
    // Legacy treated the default zero as known pricing. The retained model
    // requires pricing evidence; only the positive failed spend is known.
    assert_eq!(legacy.priced_requests, 2);
    assert_eq!(retained.metrics.priced_requests, 1);
    assert_eq!(
        retained.metrics.requests - retained.metrics.priced_requests,
        1
    );
    assert_eq!(
        legacy.related_cost_microdollars,
        retained.spend_by_currency.get("USD").copied()
    );
    assert_eq!(retained.distinct_users, Some(0));
    assert!(
        legacy.users > 0,
        "legacy claimed identities without device authentication"
    );
    let managed = ManagedRepository::new(pool.clone());
    managed
        .reconcile_inventory(
            &owner,
            &[
                ConfiguredInventoryEntry {
                    kind: "skill".to_owned(),
                    resource_key: "unused".to_owned(),
                    relative_root: "skills/unused".to_owned(),
                    availability: InventoryAvailability::Available,
                    diagnostic: None,
                },
                ConfiguredInventoryEntry {
                    kind: "skill".to_owned(),
                    resource_key: "disabled".to_owned(),
                    relative_root: "skills/disabled".to_owned(),
                    availability: InventoryAvailability::Unavailable,
                    diagnostic: Some("disabled".to_owned()),
                },
            ],
        )
        .await
        .unwrap();
    let portfolio = PortfolioSnapshotsRepository::new(pool.clone());
    let page = portfolio
        .list(
            &owner,
            &PortfolioFilter {
                after: None,
                kind: Some("skill"),
                search: "",
                window: 30,
                limit: 1,
            },
        )
        .await
        .unwrap();
    assert_eq!(page.len(), 1);
    assert!(page[0].snapshot.is_none());
    let next = portfolio
        .list(
            &owner,
            &PortfolioFilter {
                after: Some(&page[0].entry.entry_id),
                kind: Some("skill"),
                search: "",
                window: 30,
                limit: 1,
            },
        )
        .await
        .unwrap();
    assert_eq!(next.len(), 1);
    assert_ne!(page[0].entry.entry_id, next[0].entry.entry_id);
    db.cleanup().await;
}
