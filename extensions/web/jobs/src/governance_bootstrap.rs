//! Governance bootstrap: project the committed access-control baseline into the
//! authz tables.
//!
//! Four steps, in dependency order:
//! 0. Validate `services/governance/config.yaml`, failing the boot rather than
//!    letting an unparseable file degrade to the built-in defaults unnoticed,
//!    and warn when the resulting chain enforces nothing. Then validate
//!    `services/evaluation/config.yaml` against the provider catalog.
//! 1. Reconcile the services gateway-route entities into
//!    `access_control_entities` (so the FK on `access_control_rules` is
//!    satisfied and a `gateway_route` `entity_match` glob has routes to expand
//!    over), deleting catalog rows no configured route claims.
//! 2. Seed the authz tables from `services/access-control/rules.yaml` when they
//!    are empty; otherwise compare the file against the tables and log the
//!    drift without writing. Step 1's route ids are handed over as the
//!    authoritative `gateway_route` catalog, which is what expands the
//!    `gateway_route/*` glob and rejects a hand-written route id. Step 1 must
//!    therefore run first.
//! 3. Load the gateway model allow-list into `ai_gateway_policies`.
//!
//! Runs once at boot as a `scheduler.bootstrap_jobs` entry so authorization is
//! correct at app start; it is not cron-scheduled (`schedule()` is empty). The
//! CLI `admin config` reconcile path re-materialises the catalog after a live
//! gateway/provider edit, so no recurring cadence is needed. The catalog ids
//! are deterministic, so re-runs are idempotent.

use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt::security::authz::{EntityKind, RegisteredEntities};

use crate::error::JobError;
use systemprompt_web_admin::repositories::config::gateway::{
    dispatchable_route_ids, registered_routes,
};
use systemprompt_web_admin::repositories::config::{acl_bootstrap, groups_yaml_loader};
use systemprompt_web_admin::repositories::sync::sources_db::record_service_sources;
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

    let db_pool = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
        "Database not available in job context".to_owned(),
    ))?;
    let paths = ctx
        .app_paths::<Arc<AppPaths>>()
        .ok_or(MarketplaceError::Internal(
            "AppPaths not available in job context".to_owned(),
        ))?;
    let services_path = paths.system().services().to_path_buf();

    let governance = check_governance_config(&services_path)?;
    check_evaluation_config(&services_path)?;

    systemprompt_web_admin::salesforce_orgs_boot_check(&services_path)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e)))?;

    let catalog = bootstrap_gateway_entities(db_pool).await?;

    let pool = db_pool.write_pool();
    // Why: before the ACL pass. rules.yaml writes rows whose `rule_value` is a
    // group or project id, so those rows must exist first or a fresh install
    // would seed a gate naming a group nothing can ever be a member of.
    groups_yaml_loader::load_groups_from_yaml(&pool, &services_path)
        .await
        .map_err(JobError::from)?;

    // Why: seeds an empty table from rules.yaml and otherwise only reports
    // drift. Reconciling code and database is an administrator's act on the
    // sync page, never a side effect of a restart.
    let acl = acl_bootstrap::run(&pool, &services_path, &catalog.registered)
        .await
        .map_err(JobError::from)?;

    // Why: the feedback projection stamps every invocation with the source
    // that shipped its skill and that source's content hash; those come from
    // the composition this process booted on, so they are recorded here.
    let sources = record_service_sources(&pool)
        .await
        .map_err(JobError::from)?;

    let policy_repo = systemprompt::ai::repository::AiGatewayPolicyRepository::new(db_pool)
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;
    let policy = systemprompt::ai::load_gateway_policies_from_yaml(&policy_repo, &services_path)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        gateway_entities = catalog.registered.known_ids(EntityKind::GatewayRoute).len(),
        gateway_entities_pruned = catalog.pruned,
        gateway_policies = policy.inserted + policy.updated,
        access_rules_declared = acl.declared,
        access_rules_seeded = acl.seeded,
        access_drift = acl.drift.total,
        service_sources = sources.sources,
        service_owned_ids = sources.owned_ids,
        governance_policies_active = governance.active,
        governance_policies_warning = governance.warning,
        duration_ms,
        "governance bootstrap completed"
    );
    Ok(JobResult::success().with_duration(duration_ms))
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct GovernanceStatus {
    pub active: usize,
    // Why: reported separately from `active` because the two answer different
    // questions. `active` says the chain is wired; `warning` says how much of
    // it will actually refuse anything. Four active and four warning policies
    // enforce nothing, and a boot line showing only `active = 4` would read as
    // fully protected.
    pub warning: usize,
}

// Why: an unparseable policy file must fail boot rather than degrade to the
// built-in defaults — an operator who edited it to relax a policy would get
// stricter enforcement than before and no signal.
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

// Why: the plugin flag and the config file are two halves of one switch. A
// flag with no file is an operator who thinks sessions are being scored when
// nothing will run, so that fails the boot; a file with no flag is merely
// idle and is only reported.
#[doc(hidden)]
pub fn check_evaluation_config(services_path: &std::path::Path) -> Result<(), JobError> {
    use systemprompt_web_admin::repositories::config::evaluation::load_evaluation;

    let services = systemprompt::loader::ServicesBootstrap::get()?;
    let owner = services
        .plugins
        .values()
        .find(|p| p.enabled && p.hooks.evaluation)
        .map(|p| p.id.as_str());
    let loaded = load_evaluation(services_path, &services.providers, &services.ai)?;
    match (owner, loaded) {
        (Some(owner), None) => Err(JobError::Config(format!(
            "plugin '{owner}' sets hooks.evaluation but services/evaluation/config.yaml is missing"
        ))),
        (Some(owner), Some(config)) => {
            tracing::info!(
                owner,
                enabled = config.enabled,
                criteria = config.criteria.len(),
                config_hash = %config.config_hash,
                "session evaluation configured"
            );
            Ok(())
        },
        (None, Some(_)) => {
            tracing::warn!(
                "services/evaluation/config.yaml is present but no enabled plugin sets \
                 hooks.evaluation: true — sessions will not be scored"
            );
            Ok(())
        },
        (None, None) => Ok(()),
    }
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
    let id_refs: Vec<&str> = route_ids.iter().map(String::as_str).collect();

    // Why: reconciling against an empty set would delete every gateway_route
    // entity and cascade away every route grant. A services tree with no
    // gateway is a legitimate configuration, not a signal to empty the catalog,
    // so leave it untouched and let step 2 run unenforced.
    if id_refs.is_empty() {
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

    let source = format!("services:{}", gateway_path.display());
    let repo = systemprompt::security::authz::AccessControlRepository::new(db_pool)
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    let report =
        systemprompt::security::authz::reconcile_gateway_entities_exact(&repo, &id_refs, &source)
            .await
            .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;

    Ok(GatewayCatalog {
        registered,
        pruned: report.pruned,
    })
}

systemprompt::traits::submit_job!(&GovernanceBootstrapJob);
