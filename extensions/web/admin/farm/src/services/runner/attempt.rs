//! Per-claim helpers for the farm's run handler: the note a resumed attempt
//! records, the `runner` events those notes become (numbered after whatever
//! an earlier attempt recorded), and the poll that turns a dashboard cancel
//! request into the run token's cancellation.

use std::time::Duration;

use chrono::Utc;
// JSON: a released run's count lives in the stored request's `releases` key.
use serde_json::Value;
use systemprompt::agent_runs::service::ClaimedRun;
use systemprompt::agent_runs::{AgentRunRow, EventSink, RunEvent, RunnerLevel};
use systemprompt::identifiers::AgentRunId;
use systemprompt::traits::OwnedTask;

use crate::error::FarmResult;
use crate::types::{RunEventKind, RunEventPayload};

const CANCEL_POLL: Duration = Duration::from_secs(2);

pub(super) fn resume_note(row: &AgentRunRow) -> Option<String> {
    let releases = row
        .request
        .get("releases")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if row.attempt > 1 {
        Some(format!(
            "attempt {} resumes the earlier workspace",
            row.attempt
        ))
    } else if releases > 0 {
        Some(format!(
            "release {releases} resumes the workspace a shutdown interrupted"
        ))
    } else {
        None
    }
}

pub(super) async fn record_notes(
    claimed: &ClaimedRun,
    sink: &dyn EventSink,
    notes: &[String],
) -> FarmResult<Option<i64>> {
    let existing = claimed.repo.count_events(&claimed.row.id).await?;
    let batch: Vec<RunEvent> = notes
        .iter()
        .zip(existing..)
        .map(|(note, seq)| RunEvent {
            seq,
            kind: RunEventKind::Runner,
            tool_name: None,
            tool_use_id: None,
            payload: RunEventPayload::Runner {
                message: note.clone(),
                level: RunnerLevel::Warn,
            },
            created_at: Utc::now(),
        })
        .collect();
    let written = i64::try_from(batch.len()).unwrap_or(0);
    if !batch.is_empty() {
        sink.emit(batch).await?;
    }
    let total = existing + written;
    Ok((total > 0).then_some(total - 1))
}

pub(super) fn watch_cancel(claimed: &ClaimedRun) -> OwnedTask<()> {
    let repo = claimed.repo.clone();
    let token = claimed.run_token.clone();
    let id: AgentRunId = claimed.row.id.clone();
    OwnedTask::spawn("farm_cancel_watch", async move {
        loop {
            tokio::select! {
                () = token.cancelled() => return,
                () = tokio::time::sleep(CANCEL_POLL) => {},
            }
            match repo.find_cancel_requested(&id).await {
                Ok(true) => {
                    token.cancel();
                    return;
                },
                Ok(false) => {},
                Err(err) => tracing::warn!(run = %id, error = %err, "farm cancel poll failed"),
            }
        }
    })
}
