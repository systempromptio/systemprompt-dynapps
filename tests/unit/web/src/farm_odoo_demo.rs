//! The console's Odoo demo: which suites it runs, when a run counts as
//! finished, and the demo records the platform seeder restores.
//!
//! The seed plan restates `demo/agent-farm/odoo-seed.sh`; the drift test reads
//! the script so the two cannot quietly disagree about which records exist.

use std::collections::{BTreeMap, BTreeSet};

use systemprompt_web_admin_farm::services::demo::{
    ODOO_DEMO_CASE, ODOO_DEMO_FARM, is_eval_run_finished, odoo_demo_farm, odoo_demo_suites,
};
use systemprompt_web_admin_platform::odoo_demo::plan::{
    DEDUP_LEADS, DEDUP_PARTNERS, DEMO_TAGS, GDPR_DRAFT_REF, GDPR_LEAD_NAME, GDPR_POSTED_REF,
    GDPR_REQUESTER, TAG_DEDUP, TAG_GDPR, TRIAGE_LEADS,
};

use crate::support::repo_root;

#[test]
fn the_demo_runs_the_three_odoo_suites_of_the_configured_farm() {
    let suites: Vec<String> = odoo_demo_suites()
        .iter()
        .map(|s| s.as_str().to_owned())
        .collect();
    assert_eq!(
        suites,
        [
            "dynapps-agent-farm.odoo_lead_triage",
            "dynapps-agent-farm.odoo_contact_dedup",
            "dynapps-agent-farm.odoo_gdpr_erasure",
        ]
    );
    assert_eq!(odoo_demo_farm().as_str(), ODOO_DEMO_FARM);
    let farms = std::fs::read_to_string(repo_root().join("services/web/config/agent-farms.yaml"))
        .expect("agent-farms.yaml");
    assert!(
        farms.contains(&format!("id: {ODOO_DEMO_FARM}")),
        "the demo farm must be one this instance syncs"
    );
    assert_eq!(ODOO_DEMO_CASE, "live_run");
}

#[test]
fn only_queued_and_running_eval_runs_hold_the_grant_open() {
    assert!(!is_eval_run_finished("queued"));
    assert!(!is_eval_run_finished("running"));
    for status in ["completed", "failed", "cancelled", "something-new"] {
        assert!(
            is_eval_run_finished(status),
            "{status} must release the grant"
        );
    }
}

#[test]
fn every_seeded_record_carries_a_demo_tag() {
    let tags: BTreeSet<&str> = DEMO_TAGS.into_iter().collect();
    for lead in TRIAGE_LEADS.iter().chain(DEDUP_LEADS.iter()) {
        assert!(tags.contains(lead.tag), "{} is untagged", lead.name);
    }
    for partner in DEDUP_PARTNERS.iter().chain([&GDPR_REQUESTER]) {
        assert!(tags.contains(partner.tag), "{} is untagged", partner.name);
    }
    assert_eq!(GDPR_REQUESTER.tag, TAG_GDPR);
    assert!(DEDUP_PARTNERS.iter().all(|p| p.tag == TAG_DEDUP));
    assert!(GDPR_DRAFT_REF.starts_with("demo-") && GDPR_POSTED_REF.starts_with("demo-"));
    assert_ne!(GDPR_DRAFT_REF, GDPR_POSTED_REF);
}

#[test]
fn the_dedup_partners_are_three_pairs_sharing_an_email() {
    let mut by_email: BTreeMap<&str, usize> = BTreeMap::new();
    for partner in &DEDUP_PARTNERS {
        *by_email.entry(partner.email).or_default() += 1;
    }
    assert_eq!(by_email.len(), 3);
    assert!(by_email.values().all(|n| *n == 2));
    let names: BTreeSet<&str> = DEDUP_PARTNERS.iter().map(|p| p.name).collect();
    assert_eq!(
        names.len(),
        DEDUP_PARTNERS.len(),
        "upsert keys must be distinct"
    );
}

#[test]
fn the_seed_plan_matches_the_shell_seeder() {
    let script = std::fs::read_to_string(repo_root().join("demo/agent-farm/odoo-seed.sh"))
        .expect("odoo-seed.sh");
    let names = TRIAGE_LEADS
        .iter()
        .chain(DEDUP_LEADS.iter())
        .map(|l| l.name)
        .chain(DEDUP_PARTNERS.iter().map(|p| p.name))
        .chain([
            GDPR_REQUESTER.name,
            GDPR_LEAD_NAME,
            GDPR_DRAFT_REF,
            GDPR_POSTED_REF,
        ]);
    for name in names {
        assert!(script.contains(name), "{name} is not in odoo-seed.sh");
    }
    for tag in DEMO_TAGS {
        assert!(script.contains(tag), "{tag} is not in odoo-seed.sh");
    }
}
