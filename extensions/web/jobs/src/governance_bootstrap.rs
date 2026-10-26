//! Governance bootstrap: check the governance declaration and report how the
//! install's sync planes stand against their files.
//!
//! Two steps:
//! 0. Validate `services/governance/config.yaml`, failing the boot rather than
//!    letting an unparseable file degrade to the built-in defaults unnoticed.
//!    An absent file runs core's built-in chain in warn mode, logged as a
//!    warning, never anything more permissive than `unapplied_planes: warn`,
//!    and warn when the resulting chain enforces nothing. Then report whether a
//!    plugin owns `hooks.judge`, the conversation judge's switch. `warning`
//!    policies are counted apart from `active` ones because an active chain of
//!    warning-only policies enforces nothing. Paths resolve against the
//!    composed services root, the same tree `/admin/sync` reads.
//! 1. Run every sync plane through the one boot contract
//!    (`repositories::sync::boot`). The planes core enforces from the database
//!    (access control, gateway policies) are only compared and the drift
//!    logged: they reach the database through `systemprompt admin config apply`
//!    and `/admin/sync`, never through a restart. The install's own projections
//!    (groups, and the staging copies of the gateway routes and the governance
//!    chain) are seeded from their files when empty.
//!
//! The gateway-route entity catalog is core's: its boot reconcile registers a
//! row for every route the resolved catalog dispatches.
//!
//! Runs once at boot as a `scheduler.bootstrap_jobs` entry; it is not
//! cron-scheduled (`schedule()` is empty).

use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_admin_core::repositories::sync::boot::reconcile_all;
use systemprompt_web_admin_core::repositories::sync::sources_db::record_service_sources;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct GovernanceBootstrapJob;

#[async_trait::async_trait]
impl Job for GovernanceBootstrapJob {
    fn name(&self) -> &'static str {
        "governance_bootstrap"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Check the governance declaration and report the install's sync planes against \
         their files; seed them when empty"
    }

    fn schedule(&self) -> &'static str {
        ""
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(execute_inner(ctx).await?)
    }
}

async fn execute_inner(ctx: &JobContext) -> Result<JobResult, JobError> {
    let start = std::time::Instant::now();

    let db_pool = ctx.get::<DbPool>()?;
    let services_path = composed_services_root()?;

    let governance = check_governance_config(&services_path)?;
    check_judge_config(
        &ctx.get::<systemprompt::manifest::services::ServicesView>()?
            .config(),
    )?;

    let pool = db_pool.write_pool();
    let planes = reconcile_all(&pool)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;

    let sources = record_service_sources(&pool)
        .await
        .map_err(JobError::from)?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        groups_declared = planes.groups.declared,
        groups_seeded = planes.groups.seeded,
        groups_drift = planes.groups.drift_rows,
        gateway_routes_declared = planes.gateway_routes.declared,
        gateway_routes_seeded = planes.gateway_routes.seeded,
        gateway_routes_drift = planes.gateway_routes.drift_rows,
        service_sources = sources.sources,
        service_owned_ids = sources.owned_ids,
        marketplace_versions = sources.versions.marketplaces,
        marketplace_versions_new = sources.versions.new_versions,
        governance_policies_active = governance.active,
        governance_policies_warning = governance.warning,
        duration_ms,
        "governance bootstrap completed"
    );
    Ok(JobResult::success().with_duration(duration_ms))
}

fn composed_services_root() -> Result<std::path::PathBuf, JobError> {
    Ok(systemprompt::loader::services_root::ServicesRootBootstrap::require()?.path)
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct GovernanceStatus {
    pub active: usize,
    pub warning: usize,
}

#[doc(hidden)]
pub fn check_governance_config(
    services_path: &std::path::Path,
) -> Result<GovernanceStatus, JobError> {
    use systemprompt::governance::policy::GovernanceConfig;

    let path = services_path.join("governance/config.yaml");
    let config = match std::fs::read_to_string(&path) {
        Ok(text) => GovernanceConfig::parse(&text)
            .map_err(|e| MarketplaceError::config_file(path.display().to_string(), e))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                path = %path.display(),
                "no governance declaration: running core's built-in policies in warn mode, \
                 the same chain `governance.unapplied_planes: warn` leaves in force"
            );
            GovernanceConfig::defaults()
        },
        Err(e) => return Err(MarketplaceError::config_file(path.display().to_string(), e).into()),
    };
    let active = if config.enabled {
        config.policies.iter().filter(|p| p.enabled).count()
    } else {
        0
    };
    let warning = if config.enabled {
        config
            .policies
            .iter()
            .filter(|p| p.enabled && p.mode.is_warn())
            .count()
    } else {
        0
    };
    if warning > 0 {
        tracing::warn!(
            path = %path.display(),
            active,
            warning,
            "governance policies are in warn mode: they evaluate and audit but refuse              nothing. Read them back with `systemprompt infra logs governance report`."
        );
    }
    if active == 0 {
        tracing::warn!(
            path = %path.display(),
            master_switch = config.enabled,
            "governance is not enforcing: no policy will run on any request"
        );
    }
    Ok(GovernanceStatus { active, warning })
}

#[doc(hidden)]
pub fn check_judge_config(config: &systemprompt::manifest::ServicesConfig) -> Result<(), JobError> {
    let owner = config
        .plugins
        .values()
        .find(|p| p.enabled && p.hooks.judge)
        .map(|p| p.id.as_str());
    let automatic = systemprompt::config::ProfileBootstrap::get()?
        .judge
        .automatic;
    if let Some(owner) = owner {
        tracing::info!(owner, automatic, "conversation judge configured");
    } else {
        tracing::warn!(
            "no enabled plugin sets hooks.judge: true — conversations will not be judged"
        );
    }
    Ok(())
}

systemprompt::traits::submit_job!(&GovernanceBootstrapJob);
