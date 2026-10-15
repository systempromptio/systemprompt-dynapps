//! The console's Odoo demo: re-seed, elevate, run, revoke.
//!
//! One admin action drives three domains, which is why it sits at the
//! composition root rather than in any one of them:
//!
//! 1. platform re-seeds the demo-* Odoo records as the farm's service account
//!    ([`odoo_demo::reseed_odoo_demo`]);
//! 2. identity grants that account `odoo_destructive` for [`LEASE`], recorded
//!    with the acting admin as `granted_by`, so the governed delete and
//!    anonymise tools pass for the demo's runs and for nothing after it;
//! 3. farm starts the three Odoo suites, `triggered_via = demo`;
//! 4. [`watch`] renews the lease while any suite is still running and deletes
//!    the grant when the last one finishes, then attempts a `crm_lead_delete`
//!    as the account and records that it was refused ([`probe`]). If the server
//!    dies first, the lease's `valid_until` is the safety net: the govern hook
//!    stops seeing the role at read time, and the hourly sweep deletes the row.

mod handlers;
mod probe;
mod watch;

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use systemprompt::identifiers::UserId;
use systemprompt::prelude::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_farm::repositories::farms::find_farm;
use systemprompt_web_admin_farm::services::demo::{
    ODOO_DEMO_CASE, odoo_demo_farm, odoo_demo_suites,
};
use systemprompt_web_admin_farm::services::evals::{EvalLaunch, start_eval_run};
use systemprompt_web_admin_farm::state::FarmState;
use systemprompt_web_admin_farm::types::TriggeredVia;
use systemprompt_web_admin_identity::repositories::users::role_grants::{
    TimedGrant, set_timed_manual_role,
};
use systemprompt_web_admin_platform::odoo_demo::{SeedSummary, reseed_odoo_demo};
use systemprompt_web_admin_platform::service_accounts::ServiceAccounts;

pub(super) use handlers::{odoo_demo_action, odoo_demo_api};

pub(super) const DEMO_ROLE: &str = "odoo_destructive";
const LEASE: Duration = Duration::minutes(30);
const POST_REVOKE_CHECK: &str = "after the revoke, crm_lead_delete is attempted as the service account and must be refused; \
     the verdict lands in the activity feed and the Odoo audit's governance row";

#[derive(Debug, Serialize)]
pub(super) struct DemoStarted {
    pub service_account: UserId,
    pub role: &'static str,
    pub valid_until: DateTime<Utc>,
    pub standing_grant: bool,
    pub seed: SeedSummary,
    pub eval_runs: Vec<String>,
    pub post_revoke_check: &'static str,
}

async fn renew_lease(
    pool: &PgPool,
    account: &UserId,
    actor: &UserId,
) -> AdminResult<(TimedGrant, DateTime<Utc>)> {
    let until = Utc::now() + LEASE;
    let grant = set_timed_manual_role(pool, account, DEMO_ROLE, actor, until).await?;
    Ok((grant, until))
}

// Why: the admin SSR router's state is the read pool; every write here goes
// through the farm state's pool, which is the write pool.
pub(super) async fn start_odoo_demo(
    accounts: &ServiceAccounts,
    farm: &Arc<FarmState>,
    actor: &UserId,
) -> AdminResult<DemoStarted> {
    if !watch::claim_in_flight() {
        return Err(AdminError::Conflict(
            "an Odoo demo is already running; it shares one grant, so wait for its revoke".into(),
        ));
    }
    let started = launch_odoo_demo(accounts, farm, actor).await;
    if started.is_err() {
        watch::release_in_flight();
    }
    started
}

async fn launch_odoo_demo(
    accounts: &ServiceAccounts,
    farm: &Arc<FarmState>,
    actor: &UserId,
) -> AdminResult<DemoStarted> {
    let pool = &farm.pool;
    let farm_id = odoo_demo_farm();
    let account = find_farm(pool, &farm_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("farm {}", farm_id.as_str())))?
        .run_as_user_id
        .ok_or_else(|| {
            AdminError::Unprocessable(format!("farm {} names no run_as account", farm_id.as_str()))
        })?;
    let seed = reseed_odoo_demo(pool, accounts, &account).await?;
    let (grant, valid_until) = renew_lease(pool, &account, actor).await?;
    tracing::info!(target: "odoo_demo", actor = actor.as_str(), account = account.as_str(),
        role = DEMO_ROLE, %valid_until, standing = grant == TimedGrant::Standing,
        "Odoo demo granted the destructive role");
    let mut eval_runs = Vec::new();
    for suite in odoo_demo_suites() {
        let launch = EvalLaunch {
            farm_id: farm_id.clone(),
            suite_id: suite,
            user_id: actor.clone(),
            triggered_via: TriggeredVia::Demo,
            case_filter: Some(vec![ODOO_DEMO_CASE.to_owned()]),
        };
        let judge = farm.judge.judge_for(None, actor);
        let started = start_eval_run(Arc::clone(farm), judge, launch).await;
        match started {
            Ok(run) => eval_runs.push(run.id),
            Err(error) => {
                watch::revoke(pool, &account, actor, grant).await;
                return Err(AdminError::from(error));
            },
        }
    }
    let ids = eval_runs.iter().map(|id| id.as_str().to_owned()).collect();
    watch::spawn(watch::Watch {
        pool: Arc::clone(pool),
        account: account.clone(),
        actor: actor.clone(),
        grant,
        eval_runs,
        mcp_url: format!("{}/api/v1/mcp/odoo/mcp", farm.gateway.internal_base),
    });
    Ok(DemoStarted {
        service_account: account,
        role: DEMO_ROLE,
        valid_until,
        standing_grant: grant == TimedGrant::Standing,
        seed,
        eval_runs: ids,
        post_revoke_check: POST_REVOKE_CHECK,
    })
}
