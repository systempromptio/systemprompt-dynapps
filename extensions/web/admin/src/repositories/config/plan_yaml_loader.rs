//! Bootstrap loader: `services/access-control/plans.yaml` → DB.
//!
//! Runs inside the same governance bootstrap pass as the roles and departments
//! files, whose earlier passes materialise the catalog rows a plan's grants
//! reference.
//!
//! The load is staged so nothing is written until the whole document is known
//! to be coherent: `validate` parses every reference in the file,
//! `resolve_catalog` proves each granted entity already has a catalog row,
//! and only then does `persist` upsert plans and organizations and project
//! each organization's plan grants into `access_control_rules` at
//! `rule_type = 'organization'`. Each stage consumes its predecessor's output,
//! so the ordering is carried by the types rather than by call-site
//! discipline. A grant naming an entity no earlier pass registered is a typo
//! in the YAML and fails the load; it does not mint a phantom catalog row.
//!
//! Projection is the whole point — a plan is a named bundle of ordinary rules,
//! so nothing downstream needs to know plans exist. The resolver, the access
//! matrix, and the governance audit all see rule rows they already understand.
//!
//! Like every other bootstrap loader here, the direction is fixed (YAML → DB)
//! and there is no write-back: a customer's own narrowing of their plan lives
//! in the DB and survives redeploys, because projection only ever touches rows
//! whose `rule_value` is an organization slug.

use std::collections::BTreeSet;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind};
use systemprompt_web_shared::error::MarketplaceError;


use super::plan_yaml_types::{PlanLoadReport, PlansDoc, YamlGrant, YamlOrganization, YamlPlan};
use super::plan_yaml_writes;

pub(super) const PLANS_FILE: &str = "access-control/plans.yaml";

pub(super) const MICRODOLLARS_PER_USD: f64 = 1_000_000.0;

pub async fn load_plans_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<PlanLoadReport, MarketplaceError> {
    let Some(doc) = read_plans(services_path).await? else {
        return Ok(PlanLoadReport::default());
    };

    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    let validated = validate(&doc)?;
    let resolved = resolve_catalog(&repo, validated).await?;
    let report = persist(pool, &repo, resolved).await?;

    tracing::info!(
        plans = report.plans_upserted,
        organizations = report.organizations_upserted,
        grants = report.grants_projected,
        "bootstrap_plans_loaded"
    );
    Ok(report)
}

pub(super) struct ValidatedGrant<'a> {
    pub(super) kind: EntityKind,
    pub(super) access: Access,
    pub(super) grant: &'a YamlGrant,
}

struct ValidatedOrg<'a> {
    org: &'a YamlOrganization,
    grants: Vec<ValidatedGrant<'a>>,
}

struct ValidatedDoc<'a> {
    plans: &'a [YamlPlan],
    organizations: Vec<ValidatedOrg<'a>>,
    referenced: BTreeSet<(EntityKind, &'a str)>,
}

struct ResolvedDoc<'a>(ValidatedDoc<'a>);

// Why: The default organization is where a user lands when no email domain
// claims them, so an ambiguous or absent answer is only discovered at the
// moment a real person signs in. Both failures are caught here, at boot.
//
// It may not be the platform tenant: `is_platform` decides who administers
// every organization and `is_default` decides whose seat a stranger consumes.
// Fusing them would enrol every unclaimed arrival into the operator's own
// tenant — the one organization whose membership is an authorisation grant.
fn validate_default_organization(doc: &PlansDoc) -> Result<(), MarketplaceError> {
    let defaults: Vec<&str> = doc
        .organizations
        .iter()
        .filter(|o| o.default)
        .map(|o| o.slug.as_str())
        .collect();

    if defaults.len() > 1 {
        return Err(MarketplaceError::Internal(format!(
            "{PLANS_FILE}: {} organizations set `default: true` ({}) — exactly one may, or a \
             user nobody claims has no single home",
            defaults.len(),
            defaults.join(", "),
        )));
    }

    if let Some(org) = doc.organizations.iter().find(|o| o.default && o.platform) {
        return Err(MarketplaceError::Internal(format!(
            "{PLANS_FILE}: organization '{}' sets both `platform: true` and `default: true` — \
             the operator's tenant decides who administers every organization, so enrolling \
             every unclaimed arrival into it would hand out that authority by default",
            org.slug,
        )));
    }

    Ok(())
}

fn validate(doc: &PlansDoc) -> Result<ValidatedDoc<'_>, MarketplaceError> {
    validate_default_organization(doc)?;
    let mut referenced = BTreeSet::new();
    let mut organizations = Vec::with_capacity(doc.organizations.len());

    for org in &doc.organizations {
        let Some(plan) = doc.plans.iter().find(|p| p.id == org.plan) else {
            return Err(MarketplaceError::Internal(format!(
                "{PLANS_FILE}: organization '{}' references unknown plan '{}'",
                org.slug, org.plan
            )));
        };

        let mut grants = Vec::with_capacity(plan.grants.len());
        for grant in &plan.grants {
            let entry = format!("{PLANS_FILE}: {}", org.slug);
            let kind = EntityKind::from_str(&grant.entity_type)
                .map_err(|e| MarketplaceError::config_file(entry.clone(), e))?;
            let access = Access::from_str(&grant.access)
                .map_err(|e| MarketplaceError::config_file(entry, e))?;
            referenced.insert((kind, grant.entity_id.as_str()));
            grants.push(ValidatedGrant {
                kind,
                access,
                grant,
            });
        }
        organizations.push(ValidatedOrg { org, grants });
    }

    Ok(ValidatedDoc {
        plans: &doc.plans,
        organizations,
        referenced,
    })
}

async fn resolve_catalog<'a>(
    repo: &AccessControlRepository,
    doc: ValidatedDoc<'a>,
) -> Result<ResolvedDoc<'a>, MarketplaceError> {
    for (kind, entity_id) in &doc.referenced {
        let known = repo
            .get_entity(*kind, entity_id)
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
        if known.is_none() {
            return Err(MarketplaceError::Internal(format!(
                "{PLANS_FILE}: grant references {kind} '{entity_id}', which no catalog row \
                 registers — check the id, or the bootstrap pass that should have loaded it",
                kind = kind.as_str(),
            )));
        }
    }
    Ok(ResolvedDoc(doc))
}

async fn persist(
    pool: &PgPool,
    repo: &AccessControlRepository,
    ResolvedDoc(doc): ResolvedDoc<'_>,
) -> Result<PlanLoadReport, MarketplaceError> {
    let mut report = PlanLoadReport::default();

    for plan in doc.plans {
        plan_yaml_writes::upsert_plan(pool, plan).await?;
        report.plans_upserted += 1;
    }

    for entry in &doc.organizations {
        plan_yaml_writes::upsert_organization(pool, entry.org).await?;
        report.organizations_upserted += 1;
        report.grants_projected +=
            plan_yaml_writes::project_grants(pool, repo, &entry.org.slug, &entry.grants).await?;
    }

    Ok(report)
}

async fn read_plans(services_path: &Path) -> Result<Option<PlansDoc>, MarketplaceError> {
    let path = services_path.join(PLANS_FILE);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(PlansDoc::default())),
        Ok(s) => serde_yaml::from_str::<PlansDoc>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(PLANS_FILE, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
