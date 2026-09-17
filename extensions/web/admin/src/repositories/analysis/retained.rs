//! Skill drill-down figures read from retained reporting data only.
//!
//! Totals come from `analytics_snapshot_contributions` (scope `''` is the
//! organisation). Per-skill rollups and their conversations walk
//! `analytics_normalized_facts`, keyed on the **skill identity** every
//! invocation fact carries — plugin, skill, marketplace, and the services
//! source that shipped it with that source's content hash — through the
//! resource associations to the requests and assessments they name. That is
//! what lets a configured skill, a bundled kit's skill and a managed
//! resource all count; the revision proof (`resource_id`) is reported
//! beside the skill-keyed figure, never required by it. Nothing is
//! recomputed from raw events, so a skill appears only once the snapshot
//! pipeline has drained its facts.
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use systemprompt::identifiers::{
    ManagedResourceId, MarketplaceId, NativeSessionId, PluginId, UserId,
};

pub const PAGE_SIZE: u32 = 50;

/// Half-open UTC window `[start, end)` applied to a fact's `occurred_at`, the
/// same timestamp the daily snapshot buckets it under.
#[derive(Debug, Clone, Copy)]
pub struct RetainedWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, Copy, FromRow, Serialize)]
pub struct RetainedTotals {
    pub invocations: i64,
    pub attributed: i64,
    pub revision_verified: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub artifacts: i64,
    pub structured_artifacts: i64,
}

/// One skill as retained facts key it.
///
/// `resource_id`, `resource_key` and `kind` are set when some invocation of
/// the skill was revision-verified against a managed resource; `source` is
/// `base` or `bundle:<name>` and `source_hash` the content hash of that
/// source when the fact was made.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RetainedSkillRow {
    pub plugin_id: PluginId,
    pub skill: String,
    pub marketplace_id: Option<MarketplaceId>,
    pub source: Option<String>,
    pub source_hash: Option<String>,
    pub resource_id: Option<ManagedResourceId>,
    pub resource_key: Option<String>,
    pub kind: Option<String>,
    pub invocations: i64,
    pub revision_verified: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub artifacts: i64,
    pub structured_artifacts: i64,
    pub total_rows: i64,
}

/// The skill a drill-down names: the hook's `plugin_id` and its
/// `<plugin>:<skill>` string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetainedSkillRef {
    pub plugin_id: PluginId,
    pub skill: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RetainedConversationRow {
    pub session_id: NativeSessionId,
    pub user_id: UserId,
    pub host: Option<String>,
    pub first_at: DateTime<Utc>,
    pub last_at: DateTime<Utc>,
    pub invocations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub assessment_status: Option<String>,
    pub score_millionths: Option<i64>,
    pub artifacts: i64,
    pub total_rows: i64,
}

fn offset_of(page: u32) -> i64 {
    i64::from(page.saturating_sub(1)) * i64::from(PAGE_SIZE)
}

pub async fn get_retained_totals(
    pool: &PgPool,
    owner: &UserId,
    window: RetainedWindow,
) -> Result<RetainedTotals, sqlx::Error> {
    sqlx::query_as::<_, RetainedTotals>(include_str!("retained_totals.sql"))
        .bind(owner.as_str())
        .bind(window.start)
        .bind(window.end)
        .fetch_one(pool)
        .await
}

pub async fn list_retained_skills(
    pool: &PgPool,
    owner: &UserId,
    window: RetainedWindow,
    page: u32,
) -> Result<(Vec<RetainedSkillRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, RetainedSkillRow>(include_str!("retained_skills.sql"))
        .bind(owner.as_str())
        .bind(window.start)
        .bind(window.end)
        .bind(i64::from(PAGE_SIZE))
        .bind(offset_of(page))
        .fetch_all(pool)
        .await?;
    let total = rows.first().map_or(0, |row| row.total_rows);
    Ok((rows, total))
}

pub async fn list_retained_conversations(
    pool: &PgPool,
    owner: &UserId,
    resource: &ManagedResourceId,
    window: RetainedWindow,
    page: u32,
) -> Result<(Vec<RetainedConversationRow>, i64), sqlx::Error> {
    let rows =
        sqlx::query_as::<_, RetainedConversationRow>(include_str!("retained_conversations.sql"))
            .bind(owner.as_str())
            .bind(resource.as_str())
            .bind(window.start)
            .bind(window.end)
            .bind(i64::from(PAGE_SIZE))
            .bind(offset_of(page))
            .fetch_all(pool)
            .await?;
    let total = rows.first().map_or(0, |row| row.total_rows);
    Ok((rows, total))
}

pub async fn list_retained_skill_conversations(
    pool: &PgPool,
    owner: &UserId,
    skill: &RetainedSkillRef,
    window: RetainedWindow,
    page: u32,
) -> Result<(Vec<RetainedConversationRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, RetainedConversationRow>(include_str!(
        "retained_skill_conversations.sql"
    ))
    .bind(owner.as_str())
    .bind(skill.plugin_id.as_str())
    .bind(skill.skill.as_str())
    .bind(window.start)
    .bind(window.end)
    .bind(i64::from(PAGE_SIZE))
    .bind(offset_of(page))
    .fetch_all(pool)
    .await?;
    let total = rows.first().map_or(0, |row| row.total_rows);
    Ok((rows, total))
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RetainedResourceRef {
    pub resource_id: ManagedResourceId,
    pub resource_key: String,
    pub kind: String,
}

pub async fn find_retained_resource(
    pool: &PgPool,
    owner: &UserId,
    resource: &ManagedResourceId,
) -> Result<Option<RetainedResourceRef>, sqlx::Error> {
    sqlx::query_as::<_, RetainedResourceRef>(include_str!("retained_resource.sql"))
        .bind(owner.as_str())
        .bind(resource.as_str())
        .fetch_optional(pool)
        .await
}
