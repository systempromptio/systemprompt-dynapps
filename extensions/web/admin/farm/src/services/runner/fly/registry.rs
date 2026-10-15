//! Runs this API process is waiting on a worker for, keyed by `agent_runs`
//! id.
//!
//! The runner service and the HTTP router each build their own
//! `FarmState`, so the registry is process-wide. A worker always reports to
//! the API machine that launched it (see `pool::private_api_base`), so the
//! entry it needs is in this process. An entry lives from launch until the
//! handler takes the result; once removed, the run's worker token opens
//! nothing.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

use systemprompt::agent_runs::service::DbEventSink;
use tokio::sync::oneshot;

use super::protocol::{WorkerResult, WorkerSpec, token_hash};

static RUNS: LazyLock<Mutex<HashMap<String, Arc<RemoteRun>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct RemoteRun {
    token_hash: [u8; 32],
    pub spec: WorkerSpec,
    pub root: PathBuf,
    pub sink: DbEventSink,
    last_seen: Mutex<Option<Instant>>,
    cancel: AtomicBool,
    result: Mutex<Option<oneshot::Sender<WorkerResult>>>,
}

impl RemoteRun {
    fn verifies(&self, token: &str) -> bool {
        let presented = token_hash(token);
        presented
            .iter()
            .zip(self.token_hash.iter())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
    }

    pub fn touch(&self) {
        if let Ok(mut seen) = self.last_seen.lock() {
            *seen = Some(Instant::now());
        }
    }

    #[must_use]
    pub fn last_seen(&self) -> Option<Instant> {
        self.last_seen.lock().ok().and_then(|seen| *seen)
    }

    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn cancel_requested(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    pub fn deliver(&self, result: WorkerResult) -> bool {
        let sender = self.result.lock().ok().and_then(|mut slot| slot.take());
        sender.is_some_and(|tx| tx.send(result).is_ok())
    }
}

/// What the handler keeps while its run is in flight; dropping it removes
/// the entry.
#[derive(Debug)]
pub struct Registration {
    pub run: Arc<RemoteRun>,
    pub result: oneshot::Receiver<WorkerResult>,
    key: String,
}

impl Drop for Registration {
    fn drop(&mut self) {
        if let Ok(mut runs) = RUNS.lock() {
            runs.remove(&self.key);
        }
    }
}

#[must_use]
pub fn register(spec: WorkerSpec, root: PathBuf, sink: DbEventSink, token: &str) -> Registration {
    let (tx, rx) = oneshot::channel();
    let key = spec.agent_run_id.clone();
    let run = Arc::new(RemoteRun {
        token_hash: token_hash(token),
        spec,
        root,
        sink,
        last_seen: Mutex::new(None),
        cancel: AtomicBool::new(false),
        result: Mutex::new(Some(tx)),
    });
    if let Ok(mut runs) = RUNS.lock() {
        runs.insert(key.clone(), Arc::clone(&run));
    }
    Registration {
        run,
        result: rx,
        key,
    }
}

#[must_use]
pub fn authorise(agent_run_id: &str, token: &str) -> Option<Arc<RemoteRun>> {
    let run = RUNS.lock().ok()?.get(agent_run_id).cloned()?;
    run.verifies(token).then_some(run)
}
