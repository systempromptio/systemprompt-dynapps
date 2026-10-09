//! Governance bootstrap: project the committed access-control baseline into the
//! authz tables.
//!
//! Three steps, in dependency order:
//! 0. Validate `services/governance/config.yaml`, failing the boot rather than
//!    letting an unparseable file degrade to the built-in defaults unnoticed,
//!    and warn when the resulting chain enforces nothing. Then report whether a
//!    plugin owns `hooks.judge`, the conversation judge's switch. `warning`
//!    policies are counted apart from `active` ones because an active chain of
//!    warning-only policies enforces nothing. Paths resolve against the
//!    composed services root, the same tree `/admin/sync` reads.
//! 1. Reconcile the services gateway-route entities into
//!    `access_control_entities` (so the FK on `access_control_rules` is
//!    satisfied and a `gateway_route` `entity_match` glob has routes to expand
//!    over), deleting catalog rows no configured route claims. An empty route
//!    set leaves the catalog untouched: reconciling against it would cascade
//!    away every route grant, and a tree with no gateway is legitimate. The
//!    catalog only inserts unseen routes (with the `rules.yaml` default) and
//!    deletes stale ones; it never rewrites an existing row's access columns.
//! 2. Run every sync plane through the one boot contract
//!    (`repositories::sync::boot`): groups, then access control, then gateway
//!    policies. A plane whose projection is empty is seeded from its file;
//!    every other plane is compared and the drift logged, nothing written.
//!    Reconciling code and database is an administrator's act on `/admin/sync`,
//!    never a side effect of a restart. Step 1 must run first because the
//!    `gateway_route/*` glob expands over its catalog.
//!
//! Runs once at boot as a `scheduler.bootstrap_jobs` entry so authorization is
//! correct at app start; it is not cron-scheduled (`schedule()` is empty). The
//! CLI `admin config` reconcile path re-materialises the catalog after a live
//! gateway/provider edit, so no recurring cadence is needed. The catalog ids
//! are deterministic, so re-runs are idempotent.

use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt::security::authz::{EntityKind, RegisteredEntities};

use crate::error::JobError;
use systemprompt_web_admin_core::repositories::access_rules::declared_load::{
    load_declared_set, marketplace_ids_from_services,
};
use systemprompt_web_admin_core::repositories::gateway_catalog::{
    dispatchable_route_ids, registered_routes,
};
use systemprompt_web_admin_core::repositories::sync::boot::reconcile_all;
use systemprompt_web_admin_core::repositories::sync::route_catalog::reconcile_route_catalog;
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
        "Materialise gateway entities and project access-control + gateway-policy YAML into the \
         authz tables"
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
    check_judge_config()?;

    systemprompt_web_admin_platform::salesforce_orgs_boot_check(&services_path)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e)))?;

    let catalog = bootstrap_gateway_entities(db_pool).await?;

    let pool = db_pool.write_pool();
    let planes = reconcile_all(&pool)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;

    let sources = record_service_sources(&pool)
        .await
        .map_err(JobError::from)?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        gateway_entities = catalog.registered.known_ids(EntityKind::GatewayRoute).len(),
        gateway_entities_pruned = catalog.pruned,
        groups_declared = planes.groups.declared,
        groups_seeded = planes.groups.seeded,
        groups_drift = planes.groups.drift_rows,
        access_rules_declared = planes.access_control.declared,
        access_rules_seeded = planes.access_control.seeded,
        access_drift = planes.access_control.drift_rows,
        gateway_policies_declared = planes.gateway_policies.declared,
        gateway_policies_seeded = planes.gateway_policies.seeded,
        gateway_policies_drift = planes.gateway_policies.drift_rows,
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
    let profile = systemprompt::config::ProfileBootstrap::get()?;
    Ok(
        systemprompt::loader::services_root::ServicesRootBootstrap::active_root_or(
            &profile.paths.services,
        ),
    )
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
    use systemprompt::security::policy::GovernanceConfig;

    let path = services_path.join("governance/config.yaml");
    let config = GovernanceConfig::load(&path)
        .map_err(|e| MarketplaceError::config_file(path.display().to_string(), e))?;
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
pub fn check_judge_config() -> Result<(), JobError> {
    let services = systemprompt::loader::ServicesBootstrap::get()?;
    let owner = services
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

struct GatewayCatalog {
    registered: RegisteredEntities,
    pruned: u64,
}

async fn bootstrap_gateway_entities(db_pool: &DbPool) -> Result<GatewayCatalog, JobError> {
    let profile = systemprompt::config::ProfileBootstrap::get()?;
    let services = systemprompt::loader::ServicesBootstrap::get()?;
    let gateway_path = std::path::Path::new(&profile.paths.services)
        .join("ai")
        .join("gateway.yaml");

    let route_ids = dispatchable_route_ids(services);
    let registered = registered_routes(&route_ids);
    if route_ids.is_empty() {
        tracing::warn!(
            gateway = %gateway_path.display(),
            "services config declares no dispatchable gateway routes — leaving the \
             gateway_route catalog untouched and not expanding the route glob in rules.yaml"
        );
        return Ok(GatewayCatalog {
            registered,
            pruned: 0,
        });
    }

    let declared_open = declared_route_defaults(&registered).await?;
    let source = format!("services:{}", gateway_path.display());
    let report =
        reconcile_route_catalog(&db_pool.write_pool(), &route_ids, &declared_open, &source)
            .await
            .map_err(JobError::from)?;

    Ok(GatewayCatalog {
        registered,
        pruned: report.pruned,
    })
}

async fn declared_route_defaults(
    registered: &RegisteredEntities,
) -> Result<std::collections::BTreeMap<String, bool>, JobError> {
    let marketplaces = marketplace_ids_from_services().map_err(JobError::from)?;
    let declared = load_declared_set(&composed_services_root()?, &marketplaces, registered)
        .await
        .map_err(JobError::from)?;
    Ok(declared
        .entities
        .into_values()
        .filter(|e| e.entity_type == EntityKind::GatewayRoute.as_str())
        .map(|e| (e.entity_id, e.default_included))
        .collect())
}

systemprompt::traits::submit_job!(&GovernanceBootstrapJob);
