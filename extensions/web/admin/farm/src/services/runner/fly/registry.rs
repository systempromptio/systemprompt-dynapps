//! Runs this API process is waiting on a worker for, keyed by `agent_runs`
//! id, and the runner machines those runs hold.
//!
//! One [`RemoteRuns`] is built by the web extension and shared by every
//! `FarmState` in the process — the runner service that launches a run and
//! the HTTP router that its worker reports to. A worker always reports to the
//! API machine that launched it (see `pool::private_api_base`), so the entry
//! it needs is in this process. An entry lives from launch until the handler
//! takes the result; once removed, the run's worker token opens nothing.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use systemprompt::agent_runs::service::DbEventSink;
use tokio::sync::oneshot;

use super::protocol::{WorkerResult, WorkerSpec, token_hash};

/// In-flight remote runs and the pooled runner machines they occupy.
#[derive(Debug, Default)]
pub struct RemoteRuns {
    runs: Mutex<HashMap<String, Arc<RemoteRun>>>,
    taken_machines: Mutex<HashSet<String>>,
}

impl RemoteRuns {
    fn runs(&self) -> MutexGuard<'_, HashMap<String, Arc<RemoteRun>>> {
        self.runs.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn taken(&self) -> MutexGuard<'_, HashSet<String>> {
        self.taken_machines
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    #[must_use]
    pub fn register(
        self: &Arc<Self>,
        spec: WorkerSpec,
        root: PathBuf,
        sink: DbEventSink,
        token: &str,
    ) -> Registration {
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
        self.runs().insert(key.clone(), Arc::clone(&run));
        Registration {
            run,
            result: rx,
            key,
            owner: Arc::clone(self),
        }
    }

    #[must_use]
    pub fn authorise(&self, agent_run_id: &str, token: &str) -> Option<Arc<RemoteRun>> {
        let run = self.runs().get(agent_run_id).cloned()?;
        run.verifies(token).then_some(run)
    }

    pub(super) fn take_machine(&self, id: &str) -> bool {
        self.taken().insert(id.to_owned())
    }

    pub(super) fn give_back_machine(&self, id: &str) {
        self.taken().remove(id);
    }
}

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
    owner: Arc<RemoteRuns>,
}

impl Drop for Registration {
    fn drop(&mut self) {
        self.owner.runs().remove(&self.key);
    }
}
