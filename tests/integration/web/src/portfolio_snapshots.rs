//! Snapshot parity against a test-owned raw reference protects additive totals.

use crate::feedback_ingestion::{drain, seed};
use crate::portfolio_reference::list_portfolio_facts;
use crate::tempdb::TempDb;
use chrono::Utc;
use systemprompt::analytics::resource_metrics::aggregate;
use systemprompt::analytics::snapshots::FeedbackSnapshotsRepository;
use systemprompt::identifiers::{TaskId, UserId};
use systemprompt::marketplace::inventory::ConfiguredInventoryEntry;
use systemprompt::marketplace::managed::ManagedRepository;
use systemprompt::models::feedback::inventory::InventoryAvailability;
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
    let snapshots = FeedbackSnapshotsRepository::new(
        pool.clone(),
        systemprompt::analytics::feedback::FeedbackFactsRepository::new(pool.clone()),
    );
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
    assert!(
        facts.iter().all(|row| row.plugin.is_none()),
        "legacy raw events preserve absent plugin identity"
    );
    assert_eq!(
        facts
            .iter()
            .map(|row| row.skill.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        2
    );
    let legacy = aggregate(facts.iter().map(|row| &row.fact));
    assert_eq!(legacy.invocations as i64, retained.metrics.invocations);
    assert_eq!(legacy.requests as i64, retained.metrics.requests);
    assert_eq!(
        legacy.failed_requests as i64,
        retained.metrics.failed_requests
    );
    // The legacy query NULLIFs default zero cost; both models retain only
    // the positive failed request as priced and preserve the unknown denominator.
    assert_eq!(legacy.priced_requests, 1);
    assert_eq!(
        legacy.priced_requests as i64,
        retained.metrics.priced_requests
    );
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

#[tokio::test]
async fn cursor_pages_keep_canonical_entries_and_bound_candidates_are_owner_scoped() {
    use systemprompt::marketplace::managed::{NewResource, ResourceKind, SourceSpec};
    let db = TempDb::create()
        .await
        .expect("database required for bounded inventory acceptance");
    let pool = db.pool.as_ref();
    seed(pool).await;
    let owner = UserId::new("publisher");
    let repo = ManagedRepository::new(pool.clone());
    let configured: Vec<_> = (0..53)
        .map(|i| ConfiguredInventoryEntry {
            kind: "skill".to_owned(),
            resource_key: format!("page-{i:03}"),
            relative_root: format!("skills/page-{i:03}"),
            availability: if i == 52 {
                InventoryAvailability::Unavailable
            } else {
                InventoryAvailability::Available
            },
            diagnostic: None,
        })
        .collect();
    repo.reconcile_inventory(&owner, &configured).await.unwrap();
    let portfolio = PortfolioSnapshotsRepository::new(pool.clone());
    let first = portfolio
        .list(
            &owner,
            &PortfolioFilter {
                after: None,
                kind: Some("skill"),
                search: "page-",
                window: 30,
                limit: 51,
            },
        )
        .await
        .unwrap();
    assert_eq!(first.len(), 51);
    let second = portfolio
        .list(
            &owner,
            &PortfolioFilter {
                after: Some(&first[49].entry.entry_id),
                kind: Some("skill"),
                search: "page-",
                window: 30,
                limit: 51,
            },
        )
        .await
        .unwrap();
    assert_eq!(second.len(), 3);
    assert_eq!(
        first[50].entry.entry_id, second[0].entry.entry_id,
        "lookahead is not skipped"
    );
    let ids: std::collections::HashSet<_> = first[..50]
        .iter()
        .chain(&second)
        .map(|e| &e.entry.entry_id)
        .collect();
    assert_eq!(ids.len(), 53);
    assert!(
        first.iter().chain(&second).all(|e| e.snapshot.is_none()),
        "absence is retained as unknown"
    );
    // Why: entry ids are content digests, so which page `page-052` lands on
    // is an accident of hashing; what matters is that an unavailable entry is
    // retained in the pages rather than dropped.
    assert_eq!(
        first
            .iter()
            .chain(&second)
            .filter(|e| e.entry.availability == InventoryAvailability::Unavailable)
            .count(),
        1
    );
    for i in 0..3 {
        let source = repo
            .register_source(&owner, &format!("candidate-{i}"), &SourceSpec::Managed)
            .await
            .unwrap();
        repo.bind_resource(
            &owner,
            &NewResource {
                source_id: source,
                upstream_key: "shared".to_owned(),
                kind: ResourceKind::Skill,
                resource_key: "shared".to_owned(),
            },
        )
        .await
        .unwrap();
    }
    let candidates = repo
        .list_resources_for_key(&owner, "skill", "shared", 2)
        .await
        .unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates[0].id.as_str() < candidates[1].id.as_str());
    assert!(
        repo.list_resources_for_key(&UserId::new("consumer"), "skill", "shared", 51)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        repo.list_resources_for_key(&owner, "rule", "shared", 51)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        repo.list_resources_for_key(&owner, "skill", "different", 51)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        repo.list_resources_for_key(&owner, "skill", "shared", 52)
            .await
            .is_err()
    );
    assert!(
        repo.list_resources_for_key(&owner, "skill", &"x".repeat(201), 51)
            .await
            .is_err()
    );
    db.cleanup().await;
}
