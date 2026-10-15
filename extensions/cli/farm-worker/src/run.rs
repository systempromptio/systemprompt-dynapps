//! One farm run on this machine, from spec to reported result.
//!
//! The worker fetches the run's spec, confines itself (the egress allowlist
//! installed as root, then every id dropped to the agent user; a failure
//! there fails the run before the agent exists), fetches the materialised
//! run directory,
//! writes the governance hooks file against the API's hook endpoints, and
//! runs core's executor with an [`HttpEventSink`]. A heartbeat task keeps
//! the API's watchdog fed and turns a cancel the API relays into the run's
//! cancellation. After the host exits, the edited workspace and logs are
//! uploaded, then the result; the result goes last because it is what lets
//! the API finish the row and stop this machine.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use systemprompt::agent_runs::governance::{HookBases, hooks_file};
use systemprompt::agent_runs::{ExecuteParams, RunLayout, execute};
use systemprompt_web_admin_farm::services::runner::fly::protocol::{
    Bundle, HeartbeatReply, OUTPUT_DIRS, WorkerResult, WorkerSpec, collect_bundle, rebase_request,
    write_bundle,
};
use systemprompt_web_admin_farm::services::runner::observer::unexecuted_outcome;
use systemprompt_web_admin_farm::types::{AgentRunStatus, ErrorClass};
use tokio_util::sync::CancellationToken;

use crate::client::{HttpEventSink, WorkerClient};
use crate::{egress, privilege};

pub(crate) async fn run(client: &WorkerClient, root: Option<PathBuf>) -> Result<WorkerResult> {
    let spec: WorkerSpec = client.get("spec").await?;
    let root = root.unwrap_or_else(|| spec.server_root.clone());
    confine(&spec, &root).await?;
    let layout = RunLayout::new(&root);
    layout.create().await.context("run directory")?;
    let bundle: Bundle = client.get("bundle").await?;
    let written = write_root(&root, bundle).await?;
    tracing::info!(files = written, root = %root.display(), "run directory materialised");

    let request = rebase_request(&spec.request, &spec.server_root, &root);
    let credentials = spec.credentials.clone().into_run();
    let bases = HookBases {
        govern_url: spec.govern_url.clone(),
        track_url: spec.track_url.clone(),
    };
    let hooks = match (&request.plugin_id, &credentials.hook_token) {
        (Some(plugin_id), Some(token)) => {
            hooks_file(request.host, plugin_id, token, &bases, &layout).await?
        },
        _ => None,
    };
    let cancel = CancellationToken::new();
    let beat = heartbeat(client.clone(), spec.heartbeat_seconds, cancel.clone());
    let sink = HttpEventSink { client };
    let report = execute(ExecuteParams {
        request: &request,
        layout: &layout,
        credentials: &credentials,
        gateway_base_url: &spec.gateway_base_url,
        hooks_file: hooks,
        seq_after: spec.seq_after,
        sink: &sink,
        cancel,
    })
    .await;
    upload_outputs(client, &root).await;
    beat.abort();
    Ok(match report {
        Ok(report) => WorkerResult {
            outcome: report.outcome,
            diff: Some(report.diff),
        },
        Err(error) => crashed(&error.to_string()),
    })
}

async fn confine(spec: &WorkerSpec, root: &Path) -> Result<()> {
    privilege::become_root()?;
    egress::install(&spec.egress)
        .await
        .context("egress allowlist not installed")?;
    privilege::drop_to_agent(root)
}

#[must_use]
pub(crate) fn crashed(message: &str) -> WorkerResult {
    WorkerResult {
        outcome: unexecuted_outcome(AgentRunStatus::Failed, ErrorClass::HostCrashed, message),
        diff: None,
    }
}

async fn write_root(root: &Path, bundle: Bundle) -> Result<usize> {
    let root = root.to_path_buf();
    tokio::task::spawn_blocking(move || write_bundle(&root, &bundle))
        .await
        .context("materialise task")?
        .context("materialise run directory")
}

async fn upload_outputs(client: &WorkerClient, root: &Path) {
    let dir = root.to_path_buf();
    let collected = tokio::task::spawn_blocking(move || collect_bundle(&dir, &OUTPUT_DIRS)).await;
    let bundle = match collected {
        Ok(Ok(bundle)) => bundle,
        Ok(Err(error)) => return tracing::error!(%error, "outputs not collected"),
        Err(error) => return tracing::error!(%error, "outputs task failed"),
    };
    if let Err(error) = client.post("outputs", &bundle).await {
        tracing::error!(%error, "outputs not uploaded");
    }
}

fn heartbeat(
    client: WorkerClient,
    every_seconds: u64,
    cancel: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let every = Duration::from_secs(every_seconds.max(1));
    tokio::spawn(async move {
        loop {
            match client.post("heartbeat", &serde_json::json!({})).await {
                Ok(text) => {
                    let reply = serde_json::from_str::<HeartbeatReply>(&text);
                    if reply.is_ok_and(|r| r.cancel) {
                        tracing::info!("cancel relayed by the API");
                        cancel.cancel();
                    }
                },
                Err(error) => tracing::warn!(%error, "heartbeat failed"),
            }
            tokio::time::sleep(every).await;
        }
    })
}
