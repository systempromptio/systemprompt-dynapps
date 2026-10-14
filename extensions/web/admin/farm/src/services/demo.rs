//! The Odoo demo the console runs: which farm, which suites, which case.
//!
//! Only the `live_run` case is run: the `skill_used` case would execute the
//! destructive agent a second time over records the first run already
//! cleaned. The orchestration (seed, grant, watch, revoke) spans the
//! platform, identity and farm domains, so it lives at the composition root;
//! this module holds the part that is the farm's to know.

use crate::types::{EvalRunStatus, EvalSuiteId, FarmId};

pub const ODOO_DEMO_FARM: &str = "dynapps-agent-farm";
pub const ODOO_DEMO_AGENTS: [&str; 3] = [
    "odoo_lead_triage",
    "odoo_contact_dedup",
    "odoo_gdpr_erasure",
];
pub const ODOO_DEMO_CASE: &str = "live_run";

#[must_use]
pub fn odoo_demo_farm() -> FarmId {
    FarmId::new(ODOO_DEMO_FARM)
}

#[must_use]
pub fn odoo_demo_suites() -> Vec<EvalSuiteId> {
    ODOO_DEMO_AGENTS
        .iter()
        .map(|agent| EvalSuiteId::new(format!("{ODOO_DEMO_FARM}.{agent}")))
        .collect()
}

// Why: an unknown status counts as finished, so a watcher holding a grant
// open never waits forever on a status a later core version introduces.
#[must_use]
pub fn is_eval_run_finished(status: &str) -> bool {
    !matches!(
        EvalRunStatus::parse(status),
        Some(EvalRunStatus::Queued | EvalRunStatus::Running)
    )
}
