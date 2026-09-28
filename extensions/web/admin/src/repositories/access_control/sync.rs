//! Writes the declared set into the database — the only path by which
//! `rules.yaml` ever reaches `access_control_rules`.
//!
//! Boot calls it once, on an empty table. After that it runs only when an
//! administrator presses a button on `/admin/sync`, in one of
//! two modes: *insert only* adds what the file declares and the database
//! lacks; *overwrite* additionally corrects rows the two disagree on and
//! deletes rows the file does not declare: every undeclared row on an entity
//! it names, and every row this file itself wrote on an entity it has since
//! dropped — with that entity's default row, once nothing is left on it.
//! Neither mode touches the `user` band, and rows the console or a bundle
//! wrote on an entity the file does not mention are left alone.
//!
//! Everything happens in one transaction that re-reads the tables first, so
//! the picture it acts on is the one it commits against.
//! A scoped apply narrows both sides to the scope before drift is computed,
//! so it cannot see a row on any other entity as an orphan. The delete repeats
//! the `rule_type <> 'user'` guard in SQL rather than trusting the drift
//! filter.

use sqlx::{PgPool, Postgres, Transaction};
use systemprompt_security::authz::YAML_SOURCE;

use super::declared::{DeclaredEntity, DeclaredRule, DeclaredSet};
use super::drift::{DriftReport, compute_drift};
use super::rules::{list_band_rules, list_entity_defaults};
use super::validity::set_rule_validity;
pub use crate::repositories::sync::plane::{ApplyOutcome, EntityScope, SyncMode};

pub const RULES_SOURCE: &str = "services/access-control/rules.yaml";


pub async fn apply_sync(
    pool: &PgPool,
    declared: &DeclaredSet,
    mode: SyncMode,
) -> Result<ApplyOutcome, sqlx::Error> {
    apply_sync_scoped(pool, declared, mode, EntityScope::All).await
}

pub async fn apply_sync_scoped(
    pool: &PgPool,
    declared: &DeclaredSet,
    mode: SyncMode,
    scope: EntityScope<'_>,
) -> Result<ApplyOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let rules: Vec<_> = list_band_rules(&mut *tx)
        .await?
        .into_iter()
        .filter(|r| scope.admits(&r.entity_type, &r.entity_id))
        .collect();
    let entities: Vec<_> = list_entity_defaults(&mut *tx)
        .await?
        .into_iter()
        .filter(|e| scope.admits(&e.entity_type, &e.entity_id))
        .collect();
    let declared = scoped_declared(declared, scope);
    let declared = &declared;
    let drift = compute_drift(declared, &rules, &entities);

    let mut outcome = ApplyOutcome::default();
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
        outcome.deleted = delete_retired_orphans(&mut tx, &drift).await?;
        outcome.entities_retired = retire_entities(&mut tx, declared, &entities).await?;
    }

    tx.commit().await?;
    Ok(outcome)
}

fn scoped_declared(declared: &DeclaredSet, scope: EntityScope<'_>) -> DeclaredSet {
    DeclaredSet {
        entities: declared
            .entities
            .iter()
            .filter(|((kind, id), _)| scope.admits(kind, id))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        rules: declared
            .rules
            .iter()
            .filter(|(key, _)| scope.admits(&key.entity_type, &key.entity_id))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        awaiting: declared.awaiting.clone(),
        owners: declared.owners.clone(),
    }
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

// Why: core's parent-chain cache keys on COUNT + MAX(updated_at); an UPDATE
// that left `updated_at` alone would keep serving the old decision.
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

async fn delete_retired_orphans(
    tx: &mut Transaction<'_, Postgres>,
    drift: &DriftReport,
) -> Result<usize, sqlx::Error> {
    let ids: Vec<String> = drift
        .only_in_db
        .iter()
        .filter(|o| o.retire)
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

async fn retire_entities(
    tx: &mut Transaction<'_, Postgres>,
    declared: &DeclaredSet,
    entities: &[super::drift::EntityDefaultRow],
) -> Result<usize, sqlx::Error> {
    let mut retired = 0;
    for entity in entities {
        if declared.governs(&entity.entity_type, &entity.entity_id) || entity.source != RULES_SOURCE
        {
            continue;
        }
        let result = sqlx::query!(
            r"DELETE FROM access_control_entities e
               WHERE e.entity_type = $1 AND e.entity_id = $2 AND e.source = $3
                 AND NOT EXISTS (
                     SELECT 1 FROM access_control_rules r
                      WHERE r.entity_type = e.entity_type AND r.entity_id = e.entity_id)",
            entity.entity_type,
            entity.entity_id,
            RULES_SOURCE,
        )
        .execute(&mut **tx)
        .await?;
        retired += usize::try_from(result.rows_affected()).unwrap_or(0);
    }
    Ok(retired)
}
