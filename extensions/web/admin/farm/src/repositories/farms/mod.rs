//! Queries over the farms declared in agent-farms.yaml and their sync record.
//! The YAML is the source of a farm row, so a sync upserts and never deletes.

use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, UserId};

use super::filters::NewFarm;
use super::rows::FarmRow;
use crate::types::FarmId;

pub async fn list_farms(pool: &PgPool) -> Result<Vec<FarmRow>, sqlx::Error> {
    sqlx::query_file_as!(FarmRow, "src/repositories/farms/list_farms.sql")
        .fetch_all(pool)
        .await
}

pub async fn find_farm(pool: &PgPool, id: &FarmId) -> Result<Option<FarmRow>, sqlx::Error> {
    sqlx::query_file_as!(FarmRow, "src/repositories/farms/find_farm.sql", id.as_str())
        .fetch_optional(pool)
        .await
}

pub async fn find_service_account(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<UserId>, sqlx::Error> {
    sqlx::query_file_scalar!(
        "src/repositories/farms/find_service_account.sql",
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn upsert_farm(pool: &PgPool, farm: &NewFarm) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/farms/upsert_farm.sql",
        farm.id.as_str(),
        farm.name,
        farm.description,
        farm.marketplace_id.as_str(),
        farm.source_repo,
        farm.source_ref,
        farm.source_subdir,
        farm.permission_mode,
        farm.timeout_seconds,
        farm.max_turns,
        farm.max_budget_usd,
        farm.model,
        farm.run_as_user_id.as_ref().map(UserId::as_str),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_farm_sync_ok(
    pool: &PgPool,
    id: &FarmId,
    commit_sha: &str,
    synced_by: Option<&UserId>,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/farms/update_farm_sync_ok.sql",
        id.as_str(),
        commit_sha,
        synced_by.map(UserId::as_str),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_farm_sync_failed(
    pool: &PgPool,
    id: &FarmId,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/farms/update_farm_sync_failed.sql",
        id.as_str(),
        error,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_farms(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_file_scalar!("src/repositories/farms/count_farms.sql")
        .fetch_one(pool)
        .await
}
