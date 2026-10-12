//! Queries over the ordered stream-json events of one run. Events are keyed
//! by `(run_id, seq)`, so re-inserting a batch after a retry is a no-op.

use sqlx::PgPool;

use super::filters::NewRunEvent;
use super::rows::RunEventRow;
use crate::types::FarmRunId;

pub async fn insert_run_events_batch(
    pool: &PgPool,
    run_id: &FarmRunId,
    events: &[NewRunEvent],
) -> Result<u64, sqlx::Error> {
    if events.is_empty() {
        return Ok(0);
    }
    let seqs: Vec<i64> = events.iter().map(|e| e.seq).collect();
    let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
    let tool_names: Vec<Option<String>> = events.iter().map(|e| e.tool_name.clone()).collect();
    let tool_use_ids: Vec<Option<String>> = events.iter().map(|e| e.tool_use_id.clone()).collect();
    // JSON: protocol boundary — each payload is a serialized `RunEventPayload`.
    let payloads: Vec<serde_json::Value> = events.iter().map(|e| e.payload.clone()).collect();
    let result = sqlx::query_file!(
        "src/repositories/run_events/insert_run_events_batch.sql",
        run_id.as_str(),
        &seqs,
        &kinds as _,
        &tool_names as _,
        &tool_use_ids as _,
        &payloads,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn list_run_events_after(
    pool: &PgPool,
    run_id: &FarmRunId,
    after_seq: i64,
    limit: i64,
) -> Result<Vec<RunEventRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunEventRow,
        "src/repositories/run_events/list_run_events_after.sql",
        run_id.as_str(),
        after_seq,
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_run_events(
    pool: &PgPool,
    run_id: &FarmRunId,
) -> Result<Vec<RunEventRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunEventRow,
        "src/repositories/run_events/list_run_events.sql",
        run_id.as_str()
    )
    .fetch_all(pool)
    .await
}
