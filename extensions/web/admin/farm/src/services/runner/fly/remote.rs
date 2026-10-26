//! The handler's half of a run on a Fly Machine: register the run, launch a
//! runner machine, record it on the farm row, then wait for the worker's
//! terminal result and finish the `agent_runs` row with it.
//!
//! The wait ends one of five ways: the worker posts its result; the worker
//! never makes contact within `start_timeout_seconds`; it goes silent for
//! `heartbeat_timeout_seconds`; the run's wall clock (plus the start
//! allowance) runs out; or a cancel is not acknowledged within
//! `CANCEL_GRACE`. Every way stops the machine, so no runner bills past
//! its run.

use std::sync::Arc;
use std::time::{Duration, Instant};

// JSON: `agent_runs.diff_summary` is a JSONB column core's `finish` takes as a
// value; the worker's typed `DiffSummary` is encoded into it here.
use serde_json::Value;
use systemprompt::agent_runs::service::{ClaimedRun, DbEventSink};
use systemprompt::agent_runs::{RunCredentials, RunLayout, RunOutcome, RunRequest};
use uuid::Uuid;

use super::api::MachineLaunch;
use super::egress::EgressPolicy;
use super::pool::{FlyTarget, RunnerPool, resolve_target, run_env};
use super::protocol::{WorkerCredentials, WorkerResult, WorkerSpec};
use super::registry::Registration;
use crate::config::FlyRunnerConfig;
use crate::error::{FarmError, FarmResult};
use crate::repositories::runs;
use crate::services::runner::observer::unexecuted_outcome;
use crate::state::{FarmState, GatewayEndpoints};
use crate::types::{AgentRunStatus, ErrorClass, FarmRunId};

pub const HEARTBEAT_SECONDS: u64 = 10;
const CANCEL_GRACE: Duration = Duration::from_secs(30);
const WATCH_EVERY: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct RemoteJob<'a> {
    pub state: &'a FarmState,
    pub claimed: &'a ClaimedRun,
    pub farm_run_id: &'a FarmRunId,
    pub request: &'a RunRequest,
    pub layout: &'a RunLayout,
    pub credentials: &'a RunCredentials,
    pub sink: &'a DbEventSink,
    pub seq_after: Option<i64>,
}

#[must_use]
pub fn request_for_worker(request: &RunRequest, internal_base: &str, api_base: &str) -> RunRequest {
    let mut request = request.clone();
    if internal_base.is_empty() {
        return request;
    }
    for spec in request.mcp_servers.values_mut() {
        if let systemprompt::agent_runs::McpServerSpec::Http { url, .. } = spec
            && let Some(rest) = url.strip_prefix(internal_base)
        {
            *url = format!("{api_base}{rest}");
        }
    }
    request
}

fn worker_spec(job: &RemoteJob<'_>, target: &FlyTarget, egress_allow: &[String]) -> WorkerSpec {
    let hooks_base = hooks_base(&job.state.gateway.external_base, &target.api_base);
    let bases = GatewayEndpoints {
        internal_base: hooks_base.clone(),
        external_base: hooks_base.clone(),
    };
    let request = request_for_worker(
        job.request,
        &job.state.gateway.internal_base,
        &target.api_base,
    );
    let egress = EgressPolicy::for_run(
        egress_allow,
        &request,
        &[target.api_base.as_str(), hooks_base.as_str()],
    );
    WorkerSpec {
        agent_run_id: job.claimed.row.id.as_str().to_owned(),
        farm_run_id: job.farm_run_id.as_str().to_owned(),
        server_root: job.layout.root.clone(),
        request,
        credentials: WorkerCredentials::from_run(job.credentials),
        gateway_base_url: target.api_base.clone(),
        govern_url: bases.govern_url(),
        track_url: bases.track_url(),
        seq_after: job.seq_after,
        heartbeat_seconds: HEARTBEAT_SECONDS,
        egress,
    }
}

// Why: Claude Code refuses HTTP hooks whose host resolves to a private address,
// so a runner VM posts its governance hooks to the API's public URL, not 6PN.
#[must_use]
pub fn hooks_base(external_base: &str, api_base: &str) -> String {
    if external_base.is_empty() {
        api_base.to_owned()
    } else {
        external_base.to_owned()
    }
}

fn worker_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

pub async fn run_remote(fly: &FlyRunnerConfig, job: RemoteJob<'_>) -> FarmResult<()> {
    let target = resolve_target(fly)?;
    let pool = RunnerPool::new(
        fly,
        &target,
        Arc::clone(&job.state.runs),
        job.state.boot.secrets().as_ref(),
    )?;
    let id = &job.claimed.row.id;
    let token = worker_token();
    let registration = job.state.runs.register(
        worker_spec(&job, &target, &fly.egress_allow),
        job.layout.root.clone(),
        job.sink.clone(),
        &token,
    );
    let launch = MachineLaunch {
        image: target.image.clone(),
        env: run_env(id.as_str(), &target.api_base, &token),
    };
    let machine = pool.acquire(&target.region, &launch).await?;
    tracing::info!(run = %id, machine = %machine.id, region = %machine.region, "farm run on a runner machine");
    let recorded = runs::update_run_runner(
        job.state.pool.as_ref(),
        job.farm_run_id,
        &machine.id,
        &machine.region,
    )
    .await;
    let waited = match recorded {
        Ok(_) => Ok(wait_for_worker(fly, &job, registration).await),
        Err(err) => Err(FarmError::from(err)),
    };
    pool.release(&machine.id).await;
    let (outcome, diff) = waited?;
    job.claimed.repo.finish(id, &outcome, diff.as_ref()).await?;
    Ok(())
}

fn failed(class: ErrorClass, message: &str) -> (RunOutcome, Option<Value>) {
    (
        unexecuted_outcome(AgentRunStatus::Failed, class, message),
        None,
    )
}

async fn wait_for_worker(
    fly: &FlyRunnerConfig,
    job: &RemoteJob<'_>,
    mut registration: Registration,
) -> (RunOutcome, Option<Value>) {
    let run = Arc::clone(&registration.run);
    let begun = Instant::now();
    let start_allowance = Duration::from_secs(fly.start_timeout_seconds);
    let silence = Duration::from_secs(fly.heartbeat_timeout_seconds);
    let deadline = begun + job.request.wall_clock + start_allowance;
    let mut cancel_by: Option<Instant> = None;
    loop {
        tokio::select! {
            delivered = &mut registration.result => {
                return match delivered {
                    Ok(WorkerResult { outcome, diff }) => {
                        (outcome, diff.and_then(|d| serde_json::to_value(d).ok()))
                    },
                    Err(_) => failed(ErrorClass::HostCrashed, "worker result channel closed"),
                };
            },
            () = job.claimed.run_token.cancelled(), if cancel_by.is_none() => {
                run.request_cancel();
                cancel_by = Some(Instant::now() + CANCEL_GRACE);
            },
            () = tokio::time::sleep(WATCH_EVERY) => {},
        }
        let now = Instant::now();
        if cancel_by.is_some_and(|by| now >= by) {
            let outcome = unexecuted_outcome(
                AgentRunStatus::Cancelled,
                ErrorClass::Ok,
                "cancelled; the runner machine did not acknowledge",
            );
            return (outcome, None);
        }
        match run.last_seen() {
            None if now.duration_since(begun) > start_allowance => {
                return failed(ErrorClass::HostCrashed, "runner machine never reported");
            },
            Some(seen) if now.duration_since(seen) > silence => {
                return failed(ErrorClass::HostCrashed, "runner machine went silent");
            },
            _ => {},
        }
        if now >= deadline {
            let outcome = unexecuted_outcome(
                AgentRunStatus::TimedOut,
                ErrorClass::Timeout,
                "runner machine did not finish within the run's wall clock",
            );
            return (outcome, None);
        }
    }
}
