//! Performance keyed on marketplace version: every figure below is grouped
//! by the content hash of the marketplace a skill was served from.
//!
//! Facts made since the hash was stamped carry it directly; older facts are
//! resolved to the version whose interval contains them
//! (`marketplace_version_at`). Requests, spend and latency reach a version
//! through the conversation association, so they are *related* and overlap
//! when several skills share a conversation — never sum them across
//! versions or skills. Latency percentiles are real `percentile_cont` over
//! request latencies, not histogram bounds.
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{MarketplaceId, PluginId, UserId};

use super::retained::RetainedWindow;
use crate::repositories::sync::marketplace_hash::MarketplaceManifest;

/// One marketplace on the Versions landing: its current version, how many
/// versions it has had, and the window's figures across all of them.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceListRow {
    pub marketplace_id: MarketplaceId,
    pub name: Option<String>,
    pub content_hash: Option<String>,
    pub source: Option<String>,
    pub source_hash: Option<String>,
    pub plugin_count: Option<i32>,
    pub skill_count: Option<i32>,
    pub versions: i64,
    pub first_seen_at: DateTime<Utc>,
    pub last_changed_at: DateTime<Utc>,
    pub invocations: i64,
    pub failed_invocations: i64,
    pub users: i64,
    pub requests: i64,
    pub failed: i64,
    pub cost: i64,
}

/// One version of one marketplace with the window's figures.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceVersionMetricsRow {
    pub marketplace_id: MarketplaceId,
    pub content_hash: String,
    pub source: String,
    pub source_hash: Option<String>,
    pub origin: String,
    pub manifest: Option<Json<MarketplaceManifest>>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,
    pub plugin_count: i32,
    pub skill_count: i32,
    pub invocations: i64,
    pub failed_invocations: i64,
    pub users: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub p50_micros: Option<i64>,
    pub p95_micros: Option<i64>,
    pub latency_measured: i64,
}

/// One skill within one marketplace version.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceVersionSkillRow {
    pub marketplace_id: Option<MarketplaceId>,
    pub marketplace_hash: Option<String>,
    pub plugin_id: PluginId,
    pub skill: String,
    pub invocations: i64,
    pub failed_invocations: i64,
    pub users: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub p50_micros: Option<i64>,
    pub p95_micros: Option<i64>,
    pub latency_measured: i64,
}

pub async fn list_marketplace_rollups(
    pool: &PgPool,
    owner: &UserId,
    window: RetainedWindow,
) -> Result<Vec<MarketplaceListRow>, sqlx::Error> {
    sqlx::query_file_as!(
        MarketplaceListRow,
        "src/repositories/analysis/marketplace_version_list.sql",
        owner.as_str(),
        window.start,
        window.end,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_marketplace_version_metrics(
    pool: &PgPool,
    owner: &UserId,
    window: RetainedWindow,
    marketplace_id: &MarketplaceId,
) -> Result<Vec<MarketplaceVersionMetricsRow>, sqlx::Error> {
    sqlx::query_file_as!(
        MarketplaceVersionMetricsRow,
        "src/repositories/analysis/marketplace_version_metrics.sql",
        owner.as_str(),
        window.start,
        window.end,
        marketplace_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

// Why: `None` for `marketplace_id` spans every marketplace and `None` for
// `hashes` every version; the compare view passes exactly two hashes.
pub async fn list_marketplace_version_skills(
    pool: &PgPool,
    owner: &UserId,
    window: RetainedWindow,
    marketplace_id: Option<&MarketplaceId>,
    hashes: Option<&[String]>,
) -> Result<Vec<MarketplaceVersionSkillRow>, sqlx::Error> {
    sqlx::query_file_as!(
        MarketplaceVersionSkillRow,
        "src/repositories/analysis/marketplace_version_skills.sql",
        owner.as_str(),
        window.start,
        window.end,
        marketplace_id.map(MarketplaceId::as_str),
        hashes,
    )
    .fetch_all(pool)
    .await
}

/// Window-scoped figures the Skills table cannot read from a snapshot: the
/// distinct people who invoked a skill and a real latency percentile.
#[derive(Debug, Clone, Serialize)]
pub struct SkillLiveRow {
    pub plugin_id: PluginId,
    pub skill: String,
    pub users: i64,
    pub p50_micros: Option<i64>,
    pub p95_micros: Option<i64>,
    pub latency_measured: i64,
}

pub async fn list_skill_live_figures(
    pool: &PgPool,
    owner: &UserId,
    window: RetainedWindow,
) -> Result<Vec<SkillLiveRow>, sqlx::Error> {
    sqlx::query_file_as!(
        SkillLiveRow,
        "src/repositories/analysis/skill_live_figures.sql",
        owner.as_str(),
        window.start,
        window.end,
    )
    .fetch_all(pool)
    .await
}
