//! Worker-side lines the run page should show (egress install, refusals,
//! confinement failures), shipped to the API as runner events over the same
//! `events` endpoint the executor streams through.
//!
//! The notes are numbered from just past the spec's `seq_after`, and the new
//! last sequence number is handed to the executor so its events follow them.

use chrono::Utc;
use systemprompt::agent_runs::{RunEvent, RunEventKind, RunEventPayload, RunnerLevel};
use systemprompt_web_admin_farm::services::runner::fly::protocol::EventBatch;

use crate::client::WorkerClient;

#[derive(Debug, Default)]
pub(crate) struct Notes {
    lines: Vec<(RunnerLevel, String)>,
}

impl Notes {
    pub(crate) fn info(&mut self, message: String) {
        self.lines.push((RunnerLevel::Info, message));
    }

    pub(crate) fn error(&mut self, message: String) {
        self.lines.push((RunnerLevel::Error, message));
    }

    pub(crate) async fn ship(self, client: &WorkerClient, seq_after: Option<i64>) -> Option<i64> {
        if self.lines.is_empty() {
            return seq_after;
        }
        let first = seq_after.map_or(0, |s| s + 1);
        let events: Vec<RunEvent> = self
            .lines
            .into_iter()
            .zip(first..)
            .map(|((level, message), seq)| RunEvent {
                seq,
                kind: RunEventKind::Runner,
                tool_name: None,
                tool_use_id: None,
                payload: RunEventPayload::Runner { message, level },
                created_at: Utc::now(),
            })
            .collect();
        let last = events.last().map(|e| e.seq);
        match client.post("events", &EventBatch { events }).await {
            Ok(_) => last,
            Err(error) => {
                tracing::warn!(%error, "worker notes not shipped");
                seq_after
            },
        }
    }
}
