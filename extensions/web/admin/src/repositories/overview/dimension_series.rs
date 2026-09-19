//! Spend and request volume per bucket, split by model or by client, for the
//! overview's two stacked charts.
//!
//! Both dimensions come from one statement so the fold rule — top six keys by
//! window cost, the rest as "Other" — is the same for models and clients and
//! cannot drift between the two charts. The rows are sparse; the view pivots
//! them onto the spine the overview series established, which is
//! authoritative for how many buckets there are.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::series::{bucket_count, bucket_width_secs};


/// Which `ai_requests` column splits the series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverviewDimension {
    Model,
    Client,
}

impl OverviewDimension {
    // Why: the SQL branches on this literal in a CASE, never on interpolated
    // text, so the dimension cannot become an injection surface.
    const fn as_sql(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Client => "client",
        }
    }
}

/// One (bucket, key) cell of a dimension series.
#[derive(Debug, Clone)]
pub struct DimensionBucket {
    pub bucket: usize,
    pub key: String,
    pub cost_microdollars: i64,
    pub requests: i64,
}

pub async fn list_dimension_series(
    pool: &PgPool,
    range: TimeRange,
    dimension: OverviewDimension,
) -> Result<Vec<DimensionBucket>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/overview/dimension_series.sql",
        range.from,
        range.to,
        bucket_count(),
        bucket_width_secs(range),
        dimension.as_sql(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| DimensionBucket {
            bucket: usize::try_from(row.bucket).unwrap_or(0),
            key: row.key,
            cost_microdollars: row.cost,
            requests: row.requests,
        })
        .collect())
}
