//! Inventory generations as provenance: which declarations the instance was
//! observing when a generation was minted, and what moved between two of them.
//!
//! Core stamps every inventory observation with the composed services hash,
//! this repository's own tree hash and each pinned bundle's digest, and only
//! mints a new generation when the observed entry set actually moved. That
//! makes the generation number a durable handle: a campaign, a publication
//! and a skill revision can all be pinned to it, and this repository is what
//! resolves that number back to the sources behind it.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;
use systemprompt::loader::bundle::SourcesProvenance;

/// One reconcile pass that changed something, with the declaration state it
/// ran against and the entry movement that earned it a new number.
#[derive(Debug, Clone, Serialize)]
pub struct SourceGeneration {
    pub generation: i64,
    pub observed_at: DateTime<Utc>,
    pub entries: i64,
    pub sources: SourcesProvenance,
    pub added: i64,
    pub removed: i64,
    pub changed: i64,
    pub publications: i64,
}

/// One inventory entry as it stood at a chosen moment.
#[derive(Debug, Clone, Serialize)]
pub struct GenerationEntry {
    pub entry_id: String,
    pub kind: String,
    pub resource_key: String,
    pub availability: String,
    pub latest_revision_id: Option<String>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Movement {
    added: i64,
    removed: i64,
    changed: i64,
}

pub async fn list_source_generations(
    pool: &PgPool,
    owner: &UserId,
    limit: i64,
) -> Result<Vec<SourceGeneration>, sqlx::Error> {
    let movement = movement_by_generation(pool, owner).await?;
    let publications = publications_by_generation(pool, owner).await?;
    let rows = sqlx::query!(
        r#"SELECT generation, observed_at, entries, sources AS "sources: Json<SourcesProvenance>"
           FROM managed_inventory_observations
           WHERE owner_id = $1 ORDER BY generation DESC LIMIT $2"#,
        owner.as_str(),
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let moved = movement.get(&row.generation).copied().unwrap_or_default();
            SourceGeneration {
                generation: row.generation,
                observed_at: row.observed_at,
                entries: row.entries,
                sources: row.sources.0,
                added: moved.added,
                removed: moved.removed,
                changed: moved.changed,
                publications: publications.get(&row.generation).copied().unwrap_or(0),
            }
        })
        .collect())
}

pub async fn find_source_generation(
    pool: &PgPool,
    owner: &UserId,
    generation: i64,
) -> Result<Option<SourceGeneration>, sqlx::Error> {
    Ok(list_source_generations(pool, owner, 10_000)
        .await?
        .into_iter()
        .find(|row| row.generation == generation))
}

pub async fn list_generation_entries(
    pool: &PgPool,
    owner: &UserId,
    observed_at: DateTime<Utc>,
) -> Result<Vec<GenerationEntry>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT entry_id,
                  COALESCE(record->>'kind','') AS "kind!",
                  COALESCE(record->>'resource_key','') AS "resource_key!",
                  COALESCE(record->>'availability','') AS "availability!",
                  record->>'latest_revision_id' AS latest_revision_id
           FROM managed_inventory_membership
           WHERE owner_id = $1 AND effective_from <= $2
             AND (effective_until IS NULL OR effective_until > $2)
           ORDER BY entry_id"#,
        owner.as_str(),
        observed_at
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| GenerationEntry {
            entry_id: row.entry_id,
            kind: row.kind,
            resource_key: row.resource_key,
            availability: row.availability,
            latest_revision_id: row.latest_revision_id,
        })
        .collect())
}

pub async fn list_revision_digests(
    pool: &PgPool,
    owner: &UserId,
    revisions: &[String],
) -> Result<BTreeMap<String, String>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT id, digest FROM managed_revisions WHERE owner_id = $1 AND id = ANY($2)",
        owner.as_str(),
        revisions
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| (row.id, row.digest)).collect())
}

// Why: a retirement is written as a new membership record carrying
// `withdrawn`, not as a closed range, so removal is read from the record the
// pass wrote rather than from `effective_until`.
async fn movement_by_generation(
    pool: &PgPool,
    owner: &UserId,
) -> Result<BTreeMap<i64, Movement>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT o.generation AS "generation!",
                  m.record->>'availability' AS availability,
                  (SELECT p.record->>'availability'
                     FROM managed_inventory_membership p
                    WHERE p.owner_id = m.owner_id AND p.entry_id = m.entry_id
                      AND p.effective_until = m.effective_from
                    ORDER BY p.effective_from DESC LIMIT 1) AS previous
           FROM managed_inventory_observations o
           JOIN managed_inventory_membership m
             ON m.owner_id = o.owner_id AND m.effective_from = o.observed_at
           WHERE o.owner_id = $1"#,
        owner.as_str()
    )
    .fetch_all(pool)
    .await?;
    let mut movement: BTreeMap<i64, Movement> = BTreeMap::new();
    for row in rows {
        let entry = movement.entry(row.generation).or_default();
        let withdrawn = row.availability.as_deref() == Some("withdrawn");
        match row.previous {
            None => entry.added += 1,
            Some(previous) if withdrawn && previous != "withdrawn" => entry.removed += 1,
            Some(_) => entry.changed += 1,
        }
    }
    Ok(movement)
}

async fn publications_by_generation(
    pool: &PgPool,
    owner: &UserId,
) -> Result<BTreeMap<i64, i64>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT (r.comparison_evidence->>'inventory_generation')::BIGINT AS "generation!",
                  COUNT(*) AS "publications!"
           FROM managed_publications p
           JOIN managed_publication_reviews r ON r.owner_id = p.owner_id AND r.id = p.review_id
           WHERE p.owner_id = $1
             AND r.comparison_evidence->>'inventory_generation' IS NOT NULL
           GROUP BY 1"#,
        owner.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.generation, row.publications))
        .collect())
}
