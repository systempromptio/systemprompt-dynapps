//! The task-completion report a terminal run leaves behind.
//!
//! [`write_run_report`] runs after the observer stamps a run terminal: it
//! reads the run's last assistant text from `agent_run_events`, hashes the
//! files under the agent's `output_path` in the run workspace, and upserts
//! `agent_farm_run_reports`, whose SQL joins the Odoo audit rows and the eval
//! verdict. An eval settles later, so [`refresh_report_eval`] re-reads it.

use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::agent_runs::{AgentRunRepository, RunLayout};
use walkdir::WalkDir;

use crate::error::{FarmError, FarmResult};
use crate::repositories::reports::{self, NewRunReport};
use crate::repositories::{agents, runs};
use crate::services::evals::final_result_text;
use crate::types::{EvalResultId, FarmRunId, RunEvent};

const EVENT_PAGE: i64 = 1_000;
const SUMMARY_CAP_CHARS: usize = 500;
const MAX_OUTPUT_FILES: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OutputFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[must_use]
pub fn summary_of(final_message: &str) -> Option<String> {
    let paragraph = final_message
        .split("\n\n")
        .map(str::trim)
        .find(|p| !p.is_empty())?;
    Some(paragraph.chars().take(SUMMARY_CAP_CHARS).collect())
}

pub fn collect_outputs(workspace: &Path, output_path: &str) -> FarmResult<Vec<OutputFile>> {
    let relative = Path::new(output_path.trim_start_matches('/'));
    let escapes = relative.components().any(|c| {
        !matches!(
            c,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    });
    let target = workspace.join(relative);
    if escapes || !target.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(&target)
        .follow_links(false)
        .sort_by_file_name()
    {
        let entry = entry.map_err(|err| FarmError::Io(std::io::Error::other(err)))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(workspace)
            .unwrap_or_else(|_| entry.path())
            .to_string_lossy()
            .into_owned();
        let (bytes, sha256) = hash_file(entry.path())?;
        files.push(OutputFile {
            path: relative,
            bytes,
            sha256,
        });
        if files.len() >= MAX_OUTPUT_FILES {
            break;
        }
    }
    Ok(files)
}

fn hash_file(path: &Path) -> FarmResult<(u64, String)> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    let mut bytes = 0u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        bytes += read as u64;
        hasher.update(&buffer[..read]);
    }
    Ok((bytes, hex::encode(hasher.finalize())))
}

async fn list_all_events(
    repo: &AgentRunRepository,
    id: &systemprompt::identifiers::AgentRunId,
) -> FarmResult<Vec<RunEvent>> {
    let mut events: Vec<RunEvent> = Vec::new();
    loop {
        let after = events.last().map_or(-1, |e| e.seq);
        let page = repo.list_events_after(id, after, EVENT_PAGE).await?;
        if page.is_empty() {
            return Ok(events);
        }
        events.extend(page);
    }
}

pub async fn write_run_report(pool: &Arc<PgPool>, run_id: &FarmRunId) -> FarmResult<bool> {
    let Some((run, agent_run)) = runs::find_run_with_agent_run(pool, run_id).await? else {
        return Ok(false);
    };
    let repo = AgentRunRepository::from_pool(Arc::clone(pool));
    let events = list_all_events(&repo, &agent_run.id).await?;
    let final_message = final_result_text(&events);
    let summary = final_message.as_deref().and_then(summary_of);
    let output_path = agents::find_agent(pool, &run.farm_id, &run.agent_id)
        .await?
        .and_then(|agent| agent.output_path);
    let outputs = match (agent_run.run_dir.as_deref(), output_path) {
        (Some(dir), Some(path)) => {
            let workspace = RunLayout::new(dir).workspace;
            tokio::task::spawn_blocking(move || collect_outputs(&workspace, &path))
                .await
                .map_err(|source| FarmError::Task {
                    step: "report outputs",
                    source,
                })??
        },
        _ => Vec::new(),
    };
    let outputs = serde_json::to_value(&outputs).map_err(|source| FarmError::Json {
        at: "run report outputs".to_owned(),
        source,
    })?;
    let report = NewRunReport {
        run_id,
        summary: summary.as_deref(),
        final_message: final_message.as_deref(),
        outputs: &outputs,
    };
    Ok(reports::upsert_run_report(pool, &report).await?)
}

pub async fn refresh_report_eval(pool: &PgPool, result_id: &EvalResultId) {
    if let Err(error) = reports::update_report_eval(pool, result_id).await {
        tracing::warn!(%error, result = result_id.as_str(), "run report eval verdict not refreshed");
    }
}
