//! The export dialog lists datasets in registry order, and every dataset
//! registers itself with its own `order` from the crate that owns its data. A
//! dataset added without a registration, or two registrations renumbered, show
//! up here as a mismatch against the list the dialog has always shown.

use systemprompt_web_admin as _;
use systemprompt_web_admin_catalog as _;
use systemprompt_web_admin_core::export::model::DataSet;
use systemprompt_web_admin_core::export::registry;
use systemprompt_web_admin_governance as _;

const EXPECTED_DIALOG_ORDER: &[&str] = &[
    "requests",
    "sessions",
    "traces",
    "tools",
    "tools-breakdown",
    "artifacts",
    "artifacts-breakdown",
    "contexts",
    "people",
    "users",
    "groups",
    "projects",
    "history",
    "conversations",
    "analytics-models",
    "analytics-skills",
    "analytics-tools",
    "analytics-tool-servers",
    "mcp-servers",
    "mcp-calls",
    "mcp-tools",
    "analytics-sessions",
    "analytics-cost-providers",
    "analytics-cost-days",
    "analytics-cost-containers",
    "analysis-skills",
    "analysis-skill-conversations",
    "analysis-conversations",
    "analysis-conversation-breakdown",
    "analysis-conversation-turns",
    "analysis-skill-runs",
    "analysis-kit-release-impact",
    "analysis-marketplaces",
    "analysis-versions",
    "analysis-plugin-eval",
    "analysis-plugin-eval-tools",
    "governance-decisions",
    "governance-findings",
    "governance-secrets",
    "report-customer-users",
    "report-customer-projects",
    "report-customer-models",
    "report-internal-providers",
    "report-internal-models",
];

#[test]
fn registry_lists_every_dataset_in_dialog_order() {
    let ids: Vec<&'static str> = registry::all().iter().map(|r| r.dataset.id()).collect();
    assert_eq!(ids, EXPECTED_DIALOG_ORDER);
}

#[test]
fn registry_orders_are_unique() {
    let orders: Vec<u16> = registry::all().iter().map(|r| r.order).collect();
    let mut deduped = orders.clone();
    deduped.dedup();
    assert_eq!(orders, deduped);
}

#[test]
fn find_resolves_every_listed_id() {
    for id in EXPECTED_DIALOG_ORDER {
        let found = registry::find(id).map(DataSet::id);
        assert_eq!(found, Some(*id));
    }
}
