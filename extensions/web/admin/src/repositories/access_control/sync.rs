//! Writes the declared set into the database — the only path by which
//! `rules.yaml` ever reaches `access_control_rules`.
//!
//! Boot calls it once, on an empty table. After that it runs only when an
//! administrator presses a button on `/admin/sync`, in one of
//! two modes: *insert only* adds what the file declares and the database
//! lacks; *overwrite* additionally corrects rows the two disagree on and
//! deletes rows the file does not declare on entities it governs. Neither
//! mode touches the `user` band, and neither touches an entity the file does
//! not mention.
//!
//! Everything happens in one transaction that re-reads the tables first, so
//! the picture it acts on is the one it commits against.

use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use systemprompt_security::authz::YAML_SOURCE;

use super::declared::{DeclaredEntity, DeclaredRule, DeclaredSet};
use super::drift::{DriftReport, compute_drift};
use super::rules::{list_band_rules, list_entity_defaults};
use super::validity::set_rule_validity;
pub use crate::repositories::sync::plane::SyncMode;

// Why: the `source` stamped on entity rows this module writes.
pub const RULES_SOURCE: &str = "services/access-control/rules.yaml";

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct SyncOutcome {
    pub inserted: usize,
    pub updated: usize,
    pub deleted: usize,
    pub entities_inserted: usize,
    pub entities_updated: usize,
}

pub async fn apply_sync(
    pool: &PgPool,
    declared: &DeclaredSet,
    mode: SyncMode,
) -> Result<SyncOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let rules = list_band_rules(&mut *tx).await?;
    let entities = list_entity_defaults(&mut *tx).await?;
    let drift = compute_drift(declared, &rules, &entities);

    let mut outcome = SyncOutcome::default();
    for entity in &drift.entities_missing {
        upsert_entity(&mut tx, entity).await?;
        outcome.entities_inserted += 1;
    }
    if mode == SyncMode::Overwrite {
        for changed in &drift.default_changed {
            let entity =
                &declared.entities[&(changed.entity_type.clone(), changed.entity_id.clone())];
            upsert_entity(&mut tx, entity).await?;
            outcome.entities_updated += 1;
        }
    }
    for rule in &drift.missing_in_db {
        insert_rule(&mut tx, rule).await?;
        outcome.inserted += 1;
    }
    if mode == SyncMode::Overwrite {
        outcome.updated = update_changed(&mut tx, &drift).await?;
        outcome.deleted = delete_governed_orphans(&mut tx, &drift).await?;
    }

    tx.commit().await?;
    Ok(outcome)
}

async fn upsert_entity(
    tx: &mut Transaction<'_, Postgres>,
    entity: &DeclaredEntity,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
          VALUES ($1, $2, $3, $4)
          ON CONFLICT (entity_type, entity_id) DO UPDATE
             SET default_included = EXCLUDED.default_included,
                 source = EXCLUDED.source,
                 updated_at = NOW()",
        entity.entity_type,
        entity.entity_id,
        entity.default_included,
        RULES_SOURCE,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_rule(
    tx: &mut Transaction<'_, Postgres>,
    rule: &DeclaredRule,
) -> Result<(), sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let access = rule.access.to_string();
    let written = sqlx::query_scalar!(
        r#"INSERT INTO access_control_rules
              (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
          ON CONFLICT (entity_type, entity_id, rule_type, rule_value) DO UPDATE
             SET access = EXCLUDED.access,
                 justification = EXCLUDED.justification,
                 source = EXCLUDED.source,
                 updated_at = NOW()
          RETURNING id AS "id!""#,
        id,
        rule.key.entity_type,
        rule.key.entity_id,
        rule.key.rule_type,
        rule.key.rule_value,
        access,
        rule.justification,
        YAML_SOURCE,
    )
    .fetch_one(&mut **tx)
    .await?;
    set_rule_validity(&mut **tx, &written, rule.valid_until).await
}

// Why: `updated_at` is set explicitly — core's parent-chain cache keys on
// COUNT + MAX(updated_at), and an update that left the stamp alone would keep
// serving the old decision until the next insert or delete.
async fn update_changed(
    tx: &mut Transaction<'_, Postgres>,
    drift: &DriftReport,
) -> Result<usize, sqlx::Error> {
    for changed in &drift.changed {
        let access = changed.declared_access.to_string();
        sqlx::query!(
            r"UPDATE access_control_rules
                 SET access = $2, justification = $3, source = $4, updated_at = NOW()
               WHERE id = $1",
            changed.db_id,
            access,
            changed.declared_why,
            YAML_SOURCE,
        )
        .execute(&mut **tx)
        .await?;
        set_rule_validity(&mut **tx, &changed.db_id, changed.declared_valid_until).await?;
    }
    Ok(drift.changed.len())
}

// Why: the `rule_type <> 'user'` guard is redundant with the drift engine's
// filter and stays anyway. A per-person override is the one thing this module
// must never delete, and one predicate in SQL is cheaper than trusting two.
async fn delete_governed_orphans(
    tx: &mut Transaction<'_, Postgres>,
    drift: &DriftReport,
) -> Result<usize, sqlx::Error> {
    let ids: Vec<String> = drift
        .only_in_db
        .iter()
        .filter(|o| o.governed)
        .map(|o| o.row.id.clone())
        .collect();
    if ids.is_empty() {
        return Ok(0);
    }
    let result = sqlx::query!(
        r"DELETE FROM access_control_rules WHERE id = ANY($1) AND rule_type <> 'user'",
        &ids,
    )
    .execute(&mut **tx)
    .await?;
    Ok(usize::try_from(result.rows_affected()).unwrap_or(usize::MAX))
}
