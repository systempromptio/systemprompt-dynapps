//! Eval suites and their cases, as a farm sync projects them. A suite's cases
//! are replaced wholesale in one transaction so a reader never sees half.

use sqlx::PgPool;

use crate::repositories::filters::{NewEvalCase, NewEvalSuite};
use crate::repositories::rows::{EvalCaseRow, EvalSuiteRow};
use crate::types::{EvalSuiteId, FarmAgentId, FarmId};

pub async fn upsert_eval_suite(pool: &PgPool, suite: &NewEvalSuite) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/evals/upsert_eval_suite.sql",
        suite.id.as_str(),
        suite.farm_id.as_str(),
        suite.agent_id.as_str(),
        suite.source_path,
        suite.case_count,
        suite.synced_commit_sha,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_eval_suites_not_in(
    pool: &PgPool,
    farm_id: &FarmId,
    keep: &[String],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/evals/delete_eval_suites_not_in.sql",
        farm_id.as_str(),
        keep,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn replace_eval_cases(
    pool: &PgPool,
    suite_id: &EvalSuiteId,
    cases: &[NewEvalCase],
) -> Result<(), sqlx::Error> {
    let ids: Vec<String> = cases.iter().map(|c| c.id.clone()).collect();
    let prompts: Vec<Option<String>> = cases.iter().map(|c| c.prompt.clone()).collect();
    let fixtures: Vec<Option<String>> = cases.iter().map(|c| c.fixture.clone()).collect();
    let timeouts: Vec<Option<i32>> = cases.iter().map(|c| c.timeout_secs).collect();
    let assertions: Vec<serde_json::Value> = cases.iter().map(|c| c.assertions.clone()).collect();
    let rubrics: Vec<Option<String>> = cases.iter().map(|c| c.judge_rubric.clone()).collect();
    let thresholds: Vec<Option<f64>> = cases.iter().map(|c| c.judge_threshold).collect();

    let mut tx = pool.begin().await?;
    sqlx::query_file!(
        "src/repositories/evals/delete_eval_cases.sql",
        suite_id.as_str()
    )
    .execute(&mut *tx)
    .await?;
    if !cases.is_empty() {
        sqlx::query_file!(
            "src/repositories/evals/insert_eval_cases.sql",
            suite_id.as_str(),
            &ids,
            &prompts as _,
            &fixtures as _,
            &timeouts as _,
            &assertions,
            &rubrics as _,
            &thresholds as _,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

pub async fn list_eval_suites(
    pool: &PgPool,
    farm_id: &FarmId,
) -> Result<Vec<EvalSuiteRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalSuiteRow,
        "src/repositories/evals/list_eval_suites.sql",
        farm_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_all_eval_suites(pool: &PgPool) -> Result<Vec<EvalSuiteRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalSuiteRow,
        "src/repositories/evals/list_all_eval_suites.sql"
    )
    .fetch_all(pool)
    .await
}

pub async fn find_eval_suite(
    pool: &PgPool,
    id: &EvalSuiteId,
) -> Result<Option<EvalSuiteRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalSuiteRow,
        "src/repositories/evals/find_eval_suite.sql",
        id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_eval_cases(
    pool: &PgPool,
    suite_id: &EvalSuiteId,
) -> Result<Vec<EvalCaseRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalCaseRow,
        "src/repositories/evals/list_eval_cases.sql",
        suite_id.as_str()
    )
    .fetch_all(pool)
    .await
}
