//! Moves a running child's output into storage.
//!
//! Every stdout line is kept verbatim in `logs/stdout.jsonl` and decoded
//! into run events persisted in batches (50 events or 500 ms); stderr goes
//! to `logs/stderr.log` and its tail is kept for the failure message.

use std::path::Path;
use std::time::Duration;

use sqlx::PgPool;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};
use tokio::time::Instant;

use super::stream_json::{ResultSummary, StreamEvent, parse_stream_line, to_payload};
use crate::repositories::filters::NewRunEvent;
use crate::repositories::{run_events, runs};
use crate::types::{FarmRunId, RunEventPayload};

pub const BATCH_SIZE: usize = 50;
pub const BATCH_INTERVAL: Duration = Duration::from_millis(500);
pub const STDERR_TAIL_BYTES: usize = 4 * 1024;

#[derive(Debug)]
pub struct EventSink<'a> {
    pool: &'a PgPool,
    run_id: &'a FarmRunId,
    next_seq: i64,
    buffer: Vec<NewRunEvent>,
    last_flush: Instant,
    pub result: Option<ResultSummary>,
    pub claude_session_id: Option<String>,
}

impl<'a> EventSink<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool, run_id: &'a FarmRunId) -> Self {
        Self {
            pool,
            run_id,
            next_seq: 0,
            buffer: Vec::new(),
            last_flush: Instant::now(),
            result: None,
            claude_session_id: None,
        }
    }

    pub fn push(&mut self, payload: &RunEventPayload) {
        let (tool_name, tool_use_id) = match payload {
            RunEventPayload::ToolUse {
                tool_use_id, name, ..
            } => (Some(name.clone()), Some(tool_use_id.clone())),
            RunEventPayload::ToolResult { tool_use_id, .. } => (None, Some(tool_use_id.clone())),
            _ => (None, None),
        };
        let Ok(json) = serde_json::to_value(payload) else {
            return;
        };
        self.buffer.push(NewRunEvent {
            seq: self.next_seq,
            kind: payload.kind(),
            tool_name,
            tool_use_id,
            payload: json,
        });
        self.next_seq += 1;
    }

    pub fn note(&mut self, message: impl Into<String>) {
        self.push(&RunEventPayload::Runner {
            message: message.into(),
        });
    }

    pub async fn flush(&mut self) {
        self.last_flush = Instant::now();
        if self.buffer.is_empty() {
            return;
        }
        let batch = std::mem::take(&mut self.buffer);
        if let Err(err) = run_events::insert_run_events_batch(self.pool, self.run_id, &batch).await
        {
            tracing::warn!(run = %self.run_id, error = %err, "farm run events insert failed");
        }
    }

    pub async fn flush_if_due(&mut self) {
        if self.buffer.len() >= BATCH_SIZE || self.last_flush.elapsed() >= BATCH_INTERVAL {
            self.flush().await;
        }
    }

    async fn accept(&mut self, event: StreamEvent) {
        match &event {
            StreamEvent::Init { session_id, .. } if !session_id.as_str().is_empty() => {
                self.claude_session_id = Some(session_id.as_str().to_owned());
                if let Err(err) =
                    runs::update_run_claude_session(self.pool, self.run_id, session_id.as_str())
                        .await
                {
                    tracing::warn!(run = %self.run_id, error = %err, "claude session update failed");
                }
            },
            StreamEvent::Result(summary) => self.result = Some(summary.clone()),
            _ => {},
        }
        self.push(&to_payload(&event));
    }
}

pub async fn pump_stdout<R: AsyncRead + Unpin>(
    stdout: R,
    log_path: &Path,
    sink: &mut EventSink<'_>,
) {
    let mut log = open_log(log_path).await;
    let mut lines = BufReader::new(stdout).lines();
    let mut tick = tokio::time::interval(BATCH_INTERVAL);
    loop {
        tokio::select! {
            line = lines.next_line() => match line {
                Ok(Some(line)) => {
                    append_line(&mut log, &line).await;
                    for event in parse_stream_line(&line) {
                        sink.accept(event).await;
                    }
                    sink.flush_if_due().await;
                },
                Ok(None) | Err(_) => break,
            },
            _ = tick.tick() => sink.flush_if_due().await,
        }
    }
    if let Some(file) = log.as_mut()
        && let Err(err) = file.flush().await
    {
        tracing::warn!(error = %err, "farm stdout log flush failed");
    }
    sink.flush().await;
}

pub async fn drain_stderr<R: AsyncRead + Unpin>(stderr: R, log_path: &Path) -> String {
    let mut log = open_log(log_path).await;
    let mut lines = BufReader::new(stderr).lines();
    let mut tail = String::new();
    while let Ok(Some(line)) = lines.next_line().await {
        append_line(&mut log, &line).await;
        tail.push_str(&line);
        tail.push('\n');
        if tail.len() > STDERR_TAIL_BYTES * 2 {
            tail = keep_tail(&tail);
        }
    }
    keep_tail(&tail)
}

async fn open_log(path: &Path) -> Option<tokio::fs::File> {
    match tokio::fs::File::create(path).await {
        Ok(file) => Some(file),
        Err(err) => {
            tracing::warn!(path = %path.display(), error = %err, "farm run log open failed");
            None
        },
    }
}

async fn append_line(log: &mut Option<tokio::fs::File>, line: &str) {
    let Some(file) = log.as_mut() else {
        return;
    };
    let mut bytes = Vec::with_capacity(line.len() + 1);
    bytes.extend_from_slice(line.as_bytes());
    bytes.push(b'\n');
    if let Err(err) = file.write_all(&bytes).await {
        tracing::warn!(error = %err, "farm run log write failed; logging stops");
        *log = None;
    }
}

fn keep_tail(text: &str) -> String {
    if text.len() <= STDERR_TAIL_BYTES {
        return text.to_owned();
    }
    let mut start = text.len() - STDERR_TAIL_BYTES;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    text[start..].to_owned()
}
