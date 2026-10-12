//! The run dispatcher.
//!
//! A bounded queue of run ids, a cancel signal per in-flight run, and the
//! worker that drains the queue under a semaphore of
//! `limits.max_concurrent`, executing each run in its own task.

pub mod command;
pub mod diff;
pub mod execute;
pub mod governance_plugin;
pub mod outcome;
pub mod plugin_materialise;
pub mod prepare;
pub mod stream_json;
pub mod stream_payload;
pub mod stream_pump;
pub mod supervise;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::{Semaphore, mpsc, watch};

use crate::error::FarmResult;
use crate::repositories::runs;
use crate::state::FarmState;
use crate::types::FarmRunId;

pub const QUEUE_CAPACITY: usize = 256;

#[derive(Debug)]
pub struct RunnerHandle {
    tx: mpsc::Sender<FarmRunId>,
    cancels: Mutex<HashMap<String, watch::Sender<bool>>>,
}

impl RunnerHandle {
    #[must_use]
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<FarmRunId>) {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        (
            Self {
                tx,
                cancels: Mutex::new(HashMap::new()),
            },
            rx,
        )
    }

    pub async fn enqueue(&self, run_id: FarmRunId) -> FarmResult<()> {
        self.tx.send(run_id).await?;
        Ok(())
    }

    pub fn register_cancel(&self, run_id: &FarmRunId) -> watch::Receiver<bool> {
        let (tx, rx) = watch::channel(false);
        if let Ok(mut map) = self.cancels.lock() {
            map.insert(run_id.as_str().to_owned(), tx);
        }
        rx
    }

    pub fn request_cancel(&self, run_id: &FarmRunId) -> bool {
        self.cancels
            .lock()
            .ok()
            .and_then(|map| map.get(run_id.as_str()).map(|tx| tx.send(true).is_ok()))
            .unwrap_or(false)
    }

    pub fn forget(&self, run_id: &FarmRunId) {
        if let Ok(mut map) = self.cancels.lock() {
            map.remove(run_id.as_str());
        }
    }
}

pub fn install_runner(state: &Arc<FarmState>) -> Option<Arc<RunnerHandle>> {
    let (handle, rx) = RunnerHandle::new(QUEUE_CAPACITY);
    let handle = Arc::new(handle);
    if state.runner.set(Arc::clone(&handle)).is_err() {
        tracing::warn!("farm runner already installed; second install ignored");
        return None;
    }
    start_runner(Arc::clone(state), rx);
    Some(handle)
}

pub fn start_runner(state: Arc<FarmState>, mut rx: mpsc::Receiver<FarmRunId>) {
    tokio::spawn(async move {
        recover(&state).await;
        let permits = Arc::new(Semaphore::new(state.limits.max_concurrent.max(1)));
        let http = reqwest::Client::new();
        while let Some(run_id) = rx.recv().await {
            let Ok(permit) = Arc::clone(&permits).acquire_owned().await else {
                break;
            };
            let Some(handle) = state.runner().cloned() else {
                tracing::error!(run = %run_id, "farm runner handle missing; run left queued");
                continue;
            };
            let cancel = handle.register_cancel(&run_id);
            let task_state = Arc::clone(&state);
            let task_http = http.clone();
            tokio::spawn(async move {
                execute::execute_run(task_state, task_http, run_id.clone(), cancel).await;
                handle.forget(&run_id);
                drop(permit);
            });
        }
        tracing::info!("farm runner queue closed");
    });
}

async fn recover(state: &Arc<FarmState>) {
    let pool = state.pool.as_ref();
    match runs::update_orphaned_runs_failed(pool).await {
        Ok(count) => tracing::info!(count, "farm runs orphaned by restart marked failed"),
        Err(err) => tracing::error!(error = %err, "farm orphaned-run recovery failed"),
    }
    let queued = match runs::list_runs_to_requeue(pool).await {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(error = %err, "farm requeue listing failed");
            return;
        },
    };
    let Some(handle) = state.runner().cloned() else {
        return;
    };
    tokio::spawn(async move {
        for row in queued {
            if let Err(err) = handle.enqueue(row.id).await {
                tracing::error!(error = %err, "farm requeue failed");
                break;
            }
        }
    });
}
