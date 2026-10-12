//! The suite-by-case matrix: the latest decided result for every case.

use sqlx::PgPool;

use crate::repositories::summaries::MatrixCellRow;
use crate::types::{EvalRunId, EvalSuiteId, FarmAgentId, FarmId};

pub async fn list_latest_results_matrix(
    pool: &PgPool,
    farm_id: Option<&FarmId>,
) -> Result<Vec<MatrixCellRow>, sqlx::Error> {
    sqlx::query_file_as!(
        MatrixCellRow,
        "src/repositories/evals/list_latest_results_matrix.sql",
        farm_id.map(FarmId::as_str)
    )
    .fetch_all(pool)
    .await
}
