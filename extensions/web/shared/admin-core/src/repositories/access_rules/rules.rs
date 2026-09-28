//! The whole rule ledger, joined to the catalog entity each rule names.
//!
//! Read whole rather than paged in SQL: the filters and the sort the page
//! offers are over four columns, and a rule set that outgrows [`RULE_CAP`]
//! is a configuration to fix rather than a page to scroll. The cap is
//! reported to the caller so the page can say the listing is short.
//!
//! The drift engine's reads ([`list_band_rules`], [`list_entity_defaults`])
//! are uncapped: a row missing from a capped read would look like a deletion.

use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use systemprompt_security::authz::Access;

use super::drift::{BandRuleRow, EntityDefaultRow};
use crate::types::access_control::AccessDecision;

pub const RULE_CAP: i64 = 1000;

/// One `access_control_rules` row with the entity context it needs.
#[derive(Debug, Clone)]
pub struct LedgerRuleRow {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub rule_type: String,
    pub rule_value: String,
    pub access: AccessDecision,
    pub justification: Option<String>,
    pub source: String,
    pub default_included: bool,
    pub entity_source: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
}

pub async fn list_ledger_rules(pool: &PgPool) -> Result<Vec<LedgerRuleRow>, sqlx::Error> {
    sqlx::query_as!(
        LedgerRuleRow,
        r#"SELECT
                r.id AS "id!",
                r.entity_type AS "entity_type!",
                r.entity_id AS "entity_id!",
                r.rule_type AS "rule_type!",
                r.rule_value AS "rule_value!",
                r.access AS "access!: AccessDecision",
                r.justification,
                r.source AS "source!",
                COALESCE(e.default_included, false) AS "default_included!",
                e.source AS "entity_source?",
                v.valid_until AS "valid_until?"
           FROM access_control_rules r
           LEFT JOIN access_control_entities e
                  ON e.entity_type = r.entity_type AND e.entity_id = r.entity_id
           LEFT JOIN access_control_rule_validity v ON v.rule_id = r.id
           ORDER BY r.entity_type, r.entity_id, r.rule_type, r.rule_value
           LIMIT $1"#,
        RULE_CAP,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_open_entities(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!"
           FROM access_control_entities
           WHERE default_included = true"#,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_band_rules<'e, E: PgExecutor<'e>>(
    exec: E,
) -> Result<Vec<BandRuleRow>, sqlx::Error> {
    sqlx::query_as!(
        BandRuleRow,
        r#"SELECT r.id AS "id!",
                  r.entity_type AS "entity_type!",
                  r.entity_id AS "entity_id!",
                  r.rule_type AS "rule_type!",
                  r.rule_value AS "rule_value!",
                  r.access AS "access!: Access",
                  r.justification,
                  r.source AS "source!",
                  v.valid_until AS "valid_until?"
           FROM access_control_rules r
           LEFT JOIN access_control_rule_validity v ON v.rule_id = r.id
           WHERE r.rule_type <> 'user'
           ORDER BY r.entity_type, r.entity_id, r.rule_type, r.rule_value"#,
    )
    .fetch_all(exec)
    .await
}

pub async fn list_entity_defaults<'e, E: PgExecutor<'e>>(
    exec: E,
) -> Result<Vec<EntityDefaultRow>, sqlx::Error> {
    sqlx::query_as!(
        EntityDefaultRow,
        r#"SELECT entity_type AS "entity_type!",
                  entity_id AS "entity_id!",
                  default_included AS "default_included!",
                  source AS "source!"
           FROM access_control_entities
           ORDER BY entity_type, entity_id"#,
    )
    .fetch_all(exec)
    .await
}

pub async fn count_band_rules(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!"
           FROM access_control_rules
           WHERE rule_type <> 'user'"#,
    )
    .fetch_one(pool)
    .await
}
