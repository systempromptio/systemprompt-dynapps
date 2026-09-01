//! The in-process ingestion schedule.
//!
//! The in-process ingestion schedule. A tokio task sleeps until the next cron
//! occurrence and runs the ingestor; a run that overlaps the next tick is
//! skipped rather than queued, because the ingestor is single-flight and a
//! backlog of identical syncs has no value.

use super::{Ingestor, RunOptions};
use crate::error::KnowledgeBankError;
use chrono::Utc;
use croner::Cron;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

pub fn spawn(
    ingestor: Arc<Ingestor>,
    schedule: &str,
) -> Result<JoinHandle<()>, KnowledgeBankError> {
    let cron = Cron::from_str(schedule)
        // Why: CronError is variant-less for our purposes; the message is the
        // whole diagnostic and the config validator already rejected bad input.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Config(format!("ingestion.schedule: {e}")))?;
    tracing::info!(schedule, "knowledge ingestion scheduled");
    Ok(tokio::spawn(async move {
        loop {
            let now = Utc::now();
            let Ok(next) = cron.find_next_occurrence(&now, false) else {
                tracing::error!("ingestion schedule has no next occurrence; scheduler stopped");
                return;
            };
            let wait = (next - now).to_std().unwrap_or(Duration::from_secs(1));
            tokio::time::sleep(wait).await;
            if ingestor.is_running() {
                tracing::info!("ingestion tick skipped: a run is still in progress");
                continue;
            }
            match ingestor.run(&RunOptions::default()).await {
                Ok(report) => tracing::info!(
                    sources = report.sources.len(),
                    duration_ms = report.duration_ms,
                    version = report.version.as_deref().unwrap_or(""),
                    errors = report.has_errors(),
                    "scheduled knowledge ingestion finished"
                ),
                Err(e) => tracing::error!(error = %e, "scheduled knowledge ingestion failed"),
            }
        }
    }))
}
