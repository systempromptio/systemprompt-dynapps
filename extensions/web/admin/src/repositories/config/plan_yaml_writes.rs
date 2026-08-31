//! The write half of the plans bootstrap: the upserts and the grant
//! projection that `plan_yaml_loader`'s `persist` stage drives.
//!
//! Split from the loader so that file keeps to the staging it documents —
//! read, validate, resolve — while the statements that actually touch the
//! database live together. Nothing here validates: every argument has already
//! been proven coherent by the time it arrives, which is why these take the
//! YAML types directly.


use sqlx::PgPool;
use systemprompt_security::authz::{AccessControlRepository, UpsertRuleParams};
use systemprompt_web_shared::error::MarketplaceError;

use crate::authz::organization::organization_rule_type;

use super::plan_yaml_loader::{MICRODOLLARS_PER_USD, PLANS_FILE, ValidatedGrant};
use super::plan_yaml_types::{YamlOrganization, YamlPlan};

pub(super) async fn upsert_plan(pool: &PgPool, plan: &YamlPlan) -> Result<(), MarketplaceError> {
    let entry = format!("{PLANS_FILE}: {}", plan.id);
    let grants = serde_json::to_value(&plan.grants)
        .map_err(|e| MarketplaceError::config_file(entry.clone(), e))?;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a contract cap in dollars cannot exceed i64 microdollars at any plausible price"
    )]
    let cap = plan
        .monthly_cost_cap_usd
        .map(|usd| (usd * MICRODOLLARS_PER_USD) as i64);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a licence price in dollars cannot exceed i64 microdollars at any plausible price"
    )]
    let price = plan
        .monthly_price_usd
        .map_or(0, |usd| (usd * MICRODOLLARS_PER_USD) as i64);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a warning threshold in dollars cannot exceed i64 microdollars at any plausible price"
    )]
    let warn = plan
        .monthly_cost_warn_usd
        .map(|usd| (usd * MICRODOLLARS_PER_USD) as i64);

    if let Some(warn_usd) = plan.monthly_cost_warn_usd {
        let below_cap = plan.monthly_cost_cap_usd.is_some_and(|cap| warn_usd < cap);
        if !below_cap {
            return Err(MarketplaceError::Internal(format!(
                "{entry}: monthly_cost_warn_usd must be set below monthly_cost_cap_usd — a \
                 warning threshold without a cap beneath it has nothing to warn about"
            )));
        }
    }

    sqlx::query!(
        "INSERT INTO plans (id, name, description, seat_limit, monthly_cost_cap_microdollars, monthly_cost_warn_microdollars, monthly_price_microdollars, grants)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                description = EXCLUDED.description,
                seat_limit = EXCLUDED.seat_limit,
                monthly_cost_cap_microdollars = EXCLUDED.monthly_cost_cap_microdollars,
                monthly_cost_warn_microdollars = EXCLUDED.monthly_cost_warn_microdollars,
                monthly_price_microdollars = EXCLUDED.monthly_price_microdollars,
                grants = EXCLUDED.grants,
                updated_at = NOW()",
        plan.id,
        plan.name,
        plan.description,
        plan.seat_limit,
        cap,
        warn,
        price,
        grants,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn upsert_organization(
    pool: &PgPool,
    org: &YamlOrganization,
) -> Result<(), MarketplaceError> {
    sqlx::query!(
        "INSERT INTO organizations (slug, name, plan_id, seat_limit_override, email_domains, status, is_platform, is_default)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (slug) DO UPDATE
            SET name = EXCLUDED.name,
                plan_id = EXCLUDED.plan_id,
                seat_limit_override = EXCLUDED.seat_limit_override,
                email_domains = EXCLUDED.email_domains,
                status = EXCLUDED.status,
                is_platform = EXCLUDED.is_platform,
                is_default = EXCLUDED.is_default,
                updated_at = NOW()",
        org.slug,
        org.name,
        org.plan,
        org.seat_limit_override,
        &org.email_domains,
        org.status,
        org.platform,
        org.default,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: the delete is scoped to `rule_value = <this org's slug>`, so a plan
// downgrade cannot touch another customer's rules or the per-user overrides.
pub(super) async fn project_grants(
    pool: &PgPool,
    repo: &AccessControlRepository,
    slug: &str,
    grants: &[ValidatedGrant<'_>],
) -> Result<usize, MarketplaceError> {
    let rule_type = organization_rule_type();
    let mut kept_types: Vec<String> = Vec::with_capacity(grants.len());
    let mut kept_ids: Vec<String> = Vec::with_capacity(grants.len());

    for entry in grants {
        repo.upsert_rule(UpsertRuleParams {
            entity_type: entry.kind,
            entity_id: &entry.grant.entity_id,
            rule_type: rule_type.clone(),
            rule_value: slug,
            access: entry.access,
            justification: Some("granted by plan"),
        })
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        kept_types.push(entry.grant.entity_type.clone());
        kept_ids.push(entry.grant.entity_id.clone());
    }

    sqlx::query!(
        "DELETE FROM access_control_rules
         WHERE rule_type = $1
           AND rule_value = $2
           AND (entity_type, entity_id) NOT IN (
                 SELECT t, i FROM UNNEST($3::TEXT[], $4::TEXT[]) AS kept(t, i)
               )",
        rule_type.as_str(),
        slug,
        &kept_types,
        &kept_ids,
    )
    .execute(pool)
    .await?;

    Ok(grants.len())
}
