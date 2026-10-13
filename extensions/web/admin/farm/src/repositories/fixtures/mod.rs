//! Queries over the fixtures a farm sync copied into the data root.

use sqlx::PgPool;

use super::filters::NewFixture;
use super::rows::FixtureRow;
use crate::types::FarmId;

pub async fn list_fixtures(
    pool: &PgPool,
    farm_id: &FarmId,
) -> Result<Vec<FixtureRow>, sqlx::Error> {
    sqlx::query_file_as!(
        FixtureRow,
        "src/repositories/fixtures/list_fixtures.sql",
        farm_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn find_fixture(
    pool: &PgPool,
    farm_id: &FarmId,
    name: &str,
) -> Result<Option<FixtureRow>, sqlx::Error> {
    sqlx::query_file_as!(
        FixtureRow,
        "src/repositories/fixtures/find_fixture.sql",
        farm_id.as_str(),
        name,
    )
    .fetch_optional(pool)
    .await
}

pub async fn upsert_fixture(pool: &PgPool, fixture: &NewFixture) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/fixtures/upsert_fixture.sql",
        fixture.farm_id.as_str(),
        fixture.name,
        fixture.file_count,
        fixture.bytes,
        fixture.stored_path,
        fixture.synced_commit_sha,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_fixtures_not_in(
    pool: &PgPool,
    farm_id: &FarmId,
    keep: &[String],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/fixtures/delete_fixtures_not_in.sql",
        farm_id.as_str(),
        keep,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
