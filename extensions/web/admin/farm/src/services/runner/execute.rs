//! One run from dequeue to terminal status: prepare the run directory, mint
//! the owner's credentials, spawn Claude Code, pump its output, record the
//! outcome.
//!
//! The PAT is always revoked and `home/` always deleted; the workspace and
//! logs stay for the Artifacts tab.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::watch;

use super::command::{ChildEnv, CommandSpec, build_claude_command};
use super::diff::{WorkspaceDiff, snapshot, workspace_diff};
use super::governance_plugin::write_governance_plugin;
use super::outcome::{self, OutcomeInputs, run_outcome};
use super::prepare::{PreparedRun, prepare_run};
use super::stream_pump::EventSink;
use super::supervise::supervise;
use crate::error::{FarmError, FarmResult};
use crate::repositories::filters::RunOutcome;
use crate::repositories::rows::RunRow;
use crate::repositories::runs;
use crate::services::gateway_auth::{
    CredentialRequest, RunCredentials, issue_run_credentials, revoke_run_credentials,
};
use crate::state::FarmState;
use crate::types::{FarmRunId, RunEventPayload, RunStatus};

const CREDENTIAL_GRACE: Duration = Duration::from_secs(300);

pub async fn execute_run(
    state: Arc<FarmState>,
    http: reqwest::Client,
    run_id: FarmRunId,
    cancel: watch::Receiver<bool>,
) {
    if let Err(err) = execute(&state, &http, &run_id, cancel).await {
        tracing::error!(run = %run_id, error = %err, "farm run failed");
        finish(&state, &run_id, &outcome::failed(err.to_string())).await;
    }
}

async fn finish(state: &FarmState, run_id: &FarmRunId, outcome: &RunOutcome) {
    if let Err(err) = runs::update_run_finished(state.pool.as_ref(), run_id, outcome).await {
        tracing::error!(run = %run_id, error = %err, "farm run finish update failed");
    }
}

fn timeout_for(state: &FarmState, run: &RunRow) -> Duration {
    let secs = u64::try_from(run.timeout_seconds)
        .ok()
        .filter(|secs| *secs > 0)
        .unwrap_or(state.limits.default_timeout_seconds);
    Duration::from_secs(secs)
}

async fn execute(
    state: &FarmState,
    http: &reqwest::Client,
    run_id: &FarmRunId,
    cancel: watch::Receiver<bool>,
) -> FarmResult<()> {
    let pool = state.pool.as_ref();
    let Some(run) = runs::find_run(pool, run_id).await? else {
        return Ok(());
    };
    if RunStatus::parse(&run.status) != Some(RunStatus::Queued) {
        return Ok(());
    }
    if *cancel.borrow() {
        finish(
            state,
            run_id,
            &outcome::terminal(RunStatus::Cancelled, None),
        )
        .await;
        return Ok(());
    }
    runs::update_run_preparing(pool, run_id).await?;
    let prepared = prepare_run(state, &run).await?;
    let timeout = timeout_for(state, &run);

    let mut sink = EventSink::new(pool, run_id);
    for note in &prepared.notes {
        sink.note(note.clone());
    }
    let request = CredentialRequest {
        user_id: &run.user_id,
        farm_id: &run.farm_id,
        run_id,
        plugin_id: &prepared.agent.plugin_id,
        ttl: timeout + CREDENTIAL_GRACE,
    };
    let creds = match issue_run_credentials(pool, http, &state.gateway, request).await {
        Ok(creds) => creds,
        Err(err) => {
            sink.note(format!("credentials: {err}"));
            sink.flush().await;
            remove_home(&prepared).await;
            return Err(err);
        },
    };

    let job = LaunchJob {
        run: &run,
        prepared: &prepared,
        creds: &creds,
    };
    let outcome = launch(state, &job, &mut sink, timeout, cancel).await;
    sink.flush().await;
    if let Err(err) = revoke_run_credentials(pool, &run.user_id, &creds.api_key_id).await {
        tracing::warn!(run = %run_id, error = %err, "farm PAT revoke failed");
    }
    remove_home(&prepared).await;
    finish(state, run_id, &outcome?).await;
    Ok(())
}

struct LaunchJob<'a> {
    run: &'a RunRow,
    prepared: &'a PreparedRun,
    creds: &'a RunCredentials,
}

fn plugin_dirs(
    state: &FarmState,
    prepared: &PreparedRun,
    creds: &RunCredentials,
    sink: &mut EventSink<'_>,
) -> FarmResult<Vec<PathBuf>> {
    let mut dirs: Vec<PathBuf> = vec![prepared.plugin_dir.clone()];
    match (&creds.hook_token, &creds.hook_token_error) {
        (Some(token), _) => dirs.push(write_governance_plugin(
            &prepared.layout.root,
            &state.gateway.internal_base,
            &prepared.agent.plugin_id,
            token,
        )?),
        (None, error) => sink.note(format!(
            "hook token unavailable, run is not reported to governance hooks: {}",
            error.as_deref().unwrap_or("unknown")
        )),
    }
    Ok(dirs)
}

fn allowed_tools(agent_tools: &[String]) -> Vec<String> {
    let mut tools = agent_tools.to_vec();
    if !tools.is_empty() && !tools.iter().any(|t| t == "Skill") {
        tools.push("Skill".to_owned());
    }
    tools
}

async fn persist_diff(root: &std::path::Path, run: &RunRow, diff: &WorkspaceDiff) {
    match serde_json::to_vec_pretty(diff) {
        Ok(json) => {
            if let Err(err) = tokio::fs::write(root.join("diff.json"), json).await {
                tracing::warn!(run = %run.id, error = %err, "farm diff.json write failed");
            }
        },
        Err(err) => tracing::warn!(run = %run.id, error = %err, "farm diff encode failed"),
    }
}

fn spawn_failed(state: &FarmState, sink: &mut EventSink<'_>, err: &std::io::Error) -> RunOutcome {
    let message = format!("spawn {}: {err}", state.claude_bin.display());
    sink.note(message.clone());
    outcome::failed(message)
}

async fn record_pid(state: &FarmState, run_id: &FarmRunId, pid: Option<u32>) {
    let pid = pid.and_then(|pid| i32::try_from(pid).ok());
    if let Err(err) = runs::update_run_pid(state.pool.as_ref(), run_id, pid).await {
        tracing::warn!(run = %run_id, error = %err, "farm run pid not recorded");
    }
}

async fn launch(
    state: &FarmState,
    job: &LaunchJob<'_>,
    sink: &mut EventSink<'_>,
    timeout: Duration,
    cancel: watch::Receiver<bool>,
) -> FarmResult<RunOutcome> {
    let LaunchJob {
        run,
        prepared,
        creds,
    } = *job;
    let layout = &prepared.layout;
    let plugin_dirs = plugin_dirs(state, prepared, creds, sink)?;

    let run_dir = layout.root.to_string_lossy().into_owned();
    runs::update_run_started(
        state.pool.as_ref(),
        &run.id,
        &creds.gateway_session_id,
        &creds.api_key_id,
        &run_dir,
    )
    .await?;

    let allowed_tools = allowed_tools(&prepared.agent.tools);
    let model = run.model.as_deref().or(prepared.agent.model.as_deref());
    let spec = CommandSpec {
        claude_bin: &state.claude_bin,
        prompt: &prepared.prompt,
        agent_id: prepared.agent.id.as_str(),
        agents_json: &layout.agents_json(),
        settings_json: &layout.settings_json(),
        plugin_dirs: &plugin_dirs,
        permission_mode: &run.permission_mode,
        allowed_tools: &allowed_tools,
        max_turns: run.max_turns,
        max_budget_usd: run.max_budget_usd,
        model,
        home: &layout.home,
        workspace: &layout.workspace,
        env: ChildEnv {
            internal_base: &state.gateway.internal_base,
            external_base: &state.gateway.external_base,
            pat: &creds.pat_secret,
            gateway_session_id: &creds.gateway_session_id,
            hook_token: creds.hook_token.as_deref(),
            run_id: run.id.as_str(),
        },
    };

    let started = Instant::now();
    let child = match build_claude_command(&spec).spawn() {
        Ok(child) => child,
        Err(err) => return Ok(spawn_failed(state, sink, &err)),
    };
    record_pid(state, &run.id, child.id()).await;
    let child_run = supervise(child, &layout.logs, sink, timeout, cancel).await;
    if !child_run.stderr_tail.trim().is_empty() {
        sink.push(&RunEventPayload::Stderr {
            text: child_run.stderr_tail.clone(),
        });
    }
    let wall_ms = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);

    let workspace = layout.workspace.clone();
    let after = tokio::task::spawn_blocking(move || snapshot(&workspace))
        .await
        .map_err(|source| FarmError::Task {
            step: "snapshot",
            source,
        })?;
    let diff = workspace_diff(&prepared.before, &after);
    persist_diff(&layout.root, run, &diff).await;

    Ok(run_outcome(OutcomeInputs {
        ended: &child_run.ended,
        result: sink.result.as_ref(),
        stderr_tail: &child_run.stderr_tail,
        diff: &diff,
        wall_ms,
    }))
}

async fn remove_home(prepared: &PreparedRun) {
    let home = &prepared.layout.home;
    if let Err(err) = tokio::fs::remove_dir_all(home).await
        && err.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(path = %home.display(), error = %err, "farm run home cleanup failed");
    }
}
