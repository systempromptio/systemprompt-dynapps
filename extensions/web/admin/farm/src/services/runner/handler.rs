//! The farm's [`RunHandler`]: one claimed `agent_runs` row from claim to
//! finish.
//!
//! The handler materialises the run directory, issues the run's credentials
//! to the row's user, writes the governance hooks file, hands the request to
//! core's executor and finishes the row with its outcome. Credentials are
//! revoked whatever happens. A failure before the executor ran still
//! finishes the row (`failed`, `host_crashed`), so no run is left
//! `preparing`; the observer then stamps the farm row either way.
//!
//! Materialise notes become `runner` events numbered after whatever an
//! earlier attempt recorded, and `seq_after` hands the executor the last
//! one. Core's claim loop cancels a run only on shutdown or a lost lease, so
//! a cancel requested from the dashboard (`cancel_requested_at`) is polled
//! here and turned into the run token's cancellation.
//!
//! With a `fly` runner config the executor runs on a Fly Machine instead
//! ([`super::fly::remote`]); everything up to the started row is the same.

use std::sync::Arc;
use std::time::Duration;
use systemprompt::traits::OwnedTask;

use async_trait::async_trait;
use chrono::Utc;
use systemprompt::agent_runs::credentials::{
    IssueRunCredentials, RunCredentialLimits, RunCredentialService,
};
use systemprompt::agent_runs::governance::{HookBases, hooks_file};
use systemprompt::agent_runs::repository::RunStarted;
use systemprompt::agent_runs::service::{ClaimedRun, DbEventSink, RunHandler};
use systemprompt::agent_runs::{
    AgentRunsResult, EventSink, ExecuteParams, RunCredentials, RunEvent, RunLayout, RunRequest,
    RunnerLevel, execute,
};
use systemprompt::identifiers::AgentRunId;

use super::fly::remote::{RemoteJob, run_remote};
use super::{cost_window, observer, prepare};
use crate::config::FlyRunnerConfig;
use crate::error::{FarmError, FarmResult};
use crate::repositories::{agents, runs};
use crate::state::FarmState;
use crate::types::{AgentRunStatus, ErrorClass, FarmRunId, RunEventKind, RunEventPayload};

const CREDENTIAL_GRACE: Duration = Duration::from_secs(300);
const CANCEL_POLL: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub(super) struct FarmRunHandler {
    pub state: Arc<FarmState>,
    pub credentials: RunCredentialService,
    pub hooks: HookBases,
    pub fly: Option<FlyRunnerConfig>,
}

#[async_trait]
impl RunHandler for FarmRunHandler {
    async fn run(&self, claimed: ClaimedRun) -> AgentRunsResult<()> {
        let id = claimed.row.id.clone();
        if let Err(err) = self.drive(&claimed).await {
            tracing::error!(run = %id, error = %err, "farm run failed before it finished");
            let outcome = observer::unexecuted_outcome(
                AgentRunStatus::Failed,
                ErrorClass::HostCrashed,
                &err.to_string(),
            );
            claimed.repo.finish(&id, &outcome, None).await?;
        }
        if let Err(err) = observer::observe_finished(&self.state, &id).await {
            tracing::error!(run = %id, error = %err, "farm run facts not recorded");
        }
        Ok(())
    }
}

impl FarmRunHandler {
    async fn drive(&self, claimed: &ClaimedRun) -> FarmResult<()> {
        let row = &claimed.row;
        let request: RunRequest =
            serde_json::from_value(row.request.clone()).map_err(|source| FarmError::Json {
                at: format!("agent run {} request", row.id),
                source,
            })?;
        if claimed.repo.find_cancel_requested(&row.id).await? {
            let outcome = observer::unexecuted_outcome(
                AgentRunStatus::Cancelled,
                ErrorClass::Ok,
                "cancelled before it started",
            );
            claimed.repo.finish(&row.id, &outcome, None).await?;
            return Ok(());
        }
        let pool = self.state.pool.as_ref();
        let farm_run_id = FarmRunId::new(row.origin_id.clone());
        let run = runs::find_run(pool, &farm_run_id)
            .await?
            .ok_or_else(|| FarmError::NotFound(format!("farm run {farm_run_id}")))?;
        let agent = agents::find_agent(pool, &run.farm_id, &run.agent_id)
            .await?
            .ok_or_else(|| {
                FarmError::NotFound(format!("agent {}/{}", run.farm_id, run.agent_id))
            })?;

        let layout = prepare::run_layout(&self.state, &row.id);
        let notes = if row.attempt > 1 {
            layout.create().await?;
            vec![format!(
                "attempt {} resumes the earlier workspace",
                row.attempt
            )]
        } else {
            prepare::materialise(&self.state, &run, &agent, &layout).await?
        };
        runs::update_run_started(pool, &farm_run_id).await?;

        let credentials = self
            .credentials
            .issue(IssueRunCredentials {
                user_id: row.user_id.clone(),
                name: format!("farm-run-{}", row.id),
                ttl: request.wall_clock + CREDENTIAL_GRACE,
                scopes: Vec::new(),
                plugin_id: request.plugin_id.clone(),
                limits: RunCredentialLimits {
                    max_budget_usd: cost_window::run_cost_window_usd(
                        request.max_budget_usd,
                        request.model.as_deref(),
                    ),
                    max_requests: None,
                },
                purpose: Some(prepare::credential_purpose(&run.id)),
            })
            .await?;
        let request = prepare::with_mcp_bearer(&request, credentials.mcp_token.as_deref());
        let job = Job {
            farm_run_id: &farm_run_id,
            claimed,
            request: &request,
            layout: &layout,
            credentials: &credentials,
        };
        let result = self.run_host(&job, &notes).await;
        if let Err(err) = self.credentials.revoke(&credentials).await {
            tracing::warn!(run = %row.id, error = %err, "farm run credentials not revoked");
        }
        result
    }

    async fn run_host(&self, job: &Job<'_>, notes: &[String]) -> FarmResult<()> {
        let Job {
            farm_run_id,
            claimed,
            request,
            layout,
            credentials,
        } = *job;
        let id = &claimed.row.id;
        claimed
            .repo
            .update_started(
                id,
                RunStarted {
                    run_dir: &layout.root,
                    pid: None,
                    gateway_session_id: Some(&credentials.gateway_session_id),
                    api_key_id: Some(&credentials.api_key_id),
                },
            )
            .await?;
        let sink = DbEventSink::new(claimed.repo.clone(), id.clone());
        let seq_after = record_notes(claimed, &sink, notes).await?;
        if let Some(fly) = &self.fly {
            let remote = RemoteJob {
                state: &self.state,
                claimed,
                farm_run_id,
                request,
                layout,
                credentials,
                sink: &sink,
                seq_after,
            };
            return run_remote(fly, remote).await;
        }
        let hooks = match (&request.plugin_id, &credentials.hook_token) {
            (Some(plugin_id), Some(token)) => {
                hooks_file(request.host, plugin_id, token, &self.hooks, layout).await?
            },
            _ => None,
        };

        let watcher = watch_cancel(claimed);
        let report = execute(ExecuteParams {
            request,
            layout,
            credentials,
            gateway_base_url: &self.state.gateway.internal_base,
            hooks_file: hooks,
            seq_after,
            sink: &sink,
            cancel: claimed.run_token.clone(),
            harness: None,
        })
        .await;
        watcher.abort();
        let report = report?;
        let diff = serde_json::to_value(&report.diff).map_err(|source| FarmError::Json {
            at: format!("agent run {id} diff"),
            source,
        })?;
        claimed
            .repo
            .finish(id, &report.outcome, Some(&diff))
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Job<'a> {
    farm_run_id: &'a FarmRunId,
    claimed: &'a ClaimedRun,
    request: &'a RunRequest,
    layout: &'a RunLayout,
    credentials: &'a RunCredentials,
}

async fn record_notes(
    claimed: &ClaimedRun,
    sink: &DbEventSink,
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

fn watch_cancel(claimed: &ClaimedRun) -> OwnedTask<()> {
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
