//! Waits on a spawned Claude Code child while its output is pumped.
//!
//! The child is ended early on cancel or timeout by killing its whole
//! process group (it was spawned as a group leader, so tools it started die
//! too).

use std::path::Path;
use std::time::Duration;

use tokio::process::Child;
use tokio::sync::watch;

use super::stream_pump::{EventSink, drain_stderr, pump_stdout};

const STDERR_GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ended {
    Exited(Option<i32>),
    Cancelled,
    TimedOut,
    WaitFailed(String),
}

#[derive(Debug, Clone)]
pub struct ChildRun {
    pub ended: Ended,
    pub stderr_tail: String,
}

pub async fn supervise(
    mut child: Child,
    logs: &Path,
    sink: &mut EventSink<'_>,
    timeout: Duration,
    cancel: watch::Receiver<bool>,
) -> ChildRun {
    let pid = child.id();
    let stdout = child.stdout.take();
    let stderr_task = child.stderr.take().map(|stderr| {
        let path = logs.join("stderr.log");
        tokio::spawn(async move { drain_stderr(stderr, &path).await })
    });
    let stdout_path = logs.join("stdout.jsonl");

    let work = async {
        if let Some(stdout) = stdout {
            pump_stdout(stdout, &stdout_path, sink).await;
        }
        match child.wait().await {
            Ok(status) => Ended::Exited(status.code()),
            Err(err) => Ended::WaitFailed(err.to_string()),
        }
    };
    let ended = tokio::select! {
        ended = work => ended,
        () = cancelled(cancel) => Ended::Cancelled,
        () = tokio::time::sleep(timeout) => Ended::TimedOut,
    };

    if matches!(ended, Ended::Cancelled | Ended::TimedOut) {
        kill_group(pid).await;
        if let Err(err) = child.kill().await {
            tracing::warn!(error = %err, "farm child kill failed");
        }
    }
    sink.flush().await;

    let stderr_tail = match stderr_task {
        Some(task) => match tokio::time::timeout(STDERR_GRACE, task).await {
            Ok(Ok(tail)) => tail,
            Ok(Err(err)) => format!("<stderr reader failed: {err}>"),
            Err(_) => String::from("<stderr reader did not finish>"),
        },
        None => String::new(),
    };
    ChildRun { ended, stderr_tail }
}

async fn cancelled(mut rx: watch::Receiver<bool>) {
    loop {
        if *rx.borrow_and_update() {
            return;
        }
        if rx.changed().await.is_err() {
            std::future::pending::<()>().await;
        }
    }
}

async fn kill_group(pid: Option<u32>) {
    let Some(pid) = pid else {
        return;
    };
    let status = tokio::process::Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .status()
        .await;
    match status {
        Ok(status) if status.success() => {},
        Ok(status) => tracing::warn!(pid, %status, "farm process group kill refused"),
        Err(err) => tracing::warn!(pid, error = %err, "farm process group kill failed"),
    }
}
