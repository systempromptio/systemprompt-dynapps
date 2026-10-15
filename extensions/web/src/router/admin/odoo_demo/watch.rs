//! The background task that holds the demo's grant open while its suites run
//! and deletes it when they are done.
//!
//! The farm runner executes one run at a time, so three suites can outlast a
//! single lease. The watcher renews the lease on every tick while any suite
//! is unfinished, which keeps the window bounded by [`super::LEASE`] past the
//! last sign of life rather than past the start. [`MAX_WATCH`] caps a run
//! that never reports finished. After the revoke it runs the post-revoke
//! probe ([`super::probe`]): a real `crm_lead_delete` that must be refused.

use std::sync::Arc;

use systemprompt::identifiers::UserId;
use systemprompt::prelude::PgPool;
use systemprompt_web_admin_farm::repositories::evals::find_eval_run;
use systemprompt_web_admin_farm::services::demo::is_eval_run_finished;
use systemprompt_web_admin_farm::types::EvalRunId;
use systemprompt_web_admin_identity::repositories::users::role_grants::{
    TimedGrant, delete_timed_manual_role,
};

use super::{DEMO_ROLE, probe, renew_lease};

const TICK: std::time::Duration = std::time::Duration::from_secs(30);
const MAX_WATCH: std::time::Duration = std::time::Duration::from_hours(4);

pub(super) struct Watch {
    pub pool: Arc<PgPool>,
    pub account: UserId,
    pub actor: UserId,
    pub grant: TimedGrant,
    pub eval_runs: Vec<EvalRunId>,
    pub mcp_url: String,
}

pub(super) fn spawn(watch: Watch) {
    tokio::spawn(async move {
        let started = std::time::Instant::now();
        while started.elapsed() < MAX_WATCH && !all_finished(&watch).await {
            tokio::time::sleep(TICK).await;
            if watch.grant == TimedGrant::Granted
                && let Err(error) = renew_lease(&watch.pool, &watch.account, &watch.actor).await
            {
                tracing::warn!(target: "odoo_demo", %error, "Odoo demo lease not renewed");
            }
        }
        revoke(&watch.pool, &watch.account, &watch.actor, watch.grant).await;
        if watch.grant == TimedGrant::Granted {
            let target = probe::ProbeTarget {
                pool: &watch.pool,
                account: &watch.account,
                actor: &watch.actor,
                mcp_url: watch.mcp_url.clone(),
            };
            probe::run(target).await;
        }
    });
}

async fn all_finished(watch: &Watch) -> bool {
    for id in &watch.eval_runs {
        match find_eval_run(&watch.pool, id).await {
            Ok(Some(run)) if !is_eval_run_finished(&run.status) => return false,
            Ok(_) => {},
            Err(error) => {
                tracing::warn!(target: "odoo_demo", %error, eval_run = id.as_str(),
                    "Odoo demo could not read an eval run; still watching");
                return false;
            },
        }
    }
    true
}

pub(super) async fn revoke(pool: &PgPool, account: &UserId, actor: &UserId, grant: TimedGrant) {
    if grant == TimedGrant::Standing {
        return;
    }
    match delete_timed_manual_role(pool, account, DEMO_ROLE).await {
        Ok(deleted) => tracing::info!(target: "odoo_demo", actor = actor.as_str(),
            account = account.as_str(), role = DEMO_ROLE, deleted,
            "Odoo demo revoked the destructive role"),
        Err(error) => tracing::error!(target: "odoo_demo", %error, account = account.as_str(),
            "Odoo demo grant not revoked; valid_until still bounds it"),
    }
}
