//! The inactivity job (decision D4), which also prunes old reply events.
//!
//! Every run first prunes the event log ([`ChatStore::prune_events`]): the
//! events of replies compacted more than [`EVENT_RETENTION`] (24 h) ago, and
//! events of no reply older than that. A reply still running is never
//! compacted, so its events stay. This runs whether or not `inactive_days` is
//! set; the finished messages keep the text, so a `/log` replay from the start
//! only loses events a client no longer needs.
//!
//! Threads are kept until a person deletes them, unless the instance sets
//! `inactive_days` in its chats config: a thread with no activity for that
//! long is deleted by this job, as a tombstone, so every client drops it on
//! its next `changes` call. Activity is the thread's own `updated_at` (a
//! message, the end of a reply, an edit) or any event of a reply written since,
//! so a reply that is still streaming keeps its thread. The job never removes
//! a row for good: [`ChatStore::delete_thread`] writes the tombstone and drops
//! the messages and events, which is what a person's `DELETE` does.
//!
//! A run works in batches of [`BATCH`] threads, at most [`MAX_BATCHES`] of
//! them, and the next night's run takes what is left. A thread edited between
//! the read and the delete (its revision moved) is left alone: that edit was
//! activity. The config is read on every run, so a changed or removed
//! `inactive_days` applies from the next one. [`sweep`] prunes the events and
//! then tombstones the threads idle for longer than a window at a given time
//! ([`Run`]); with no window it stops after the prune. When it has tombstoned
//! threads it sends each
//! person whose threads it removed one change hint (not one per thread), after
//! the run, so their desktops pull the tombstones now.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use systemprompt::database::DbPool;
use systemprompt::extension::ExtensionContext;
use systemprompt::identifiers::UserId;
use systemprompt::traits::{Job, JobContext, JobResult, MissingDependency, ProviderError};

use crate::clock::{Clock, SystemClock};
use crate::config::{CONFIG_PREFIX, ChatsConfig, ConfigError};
use crate::hint::{Hints, RelayHints};
use crate::store::postgres::PgChatStore;
use crate::store::{ChatStore, StoreError, WriteOutcome};

pub const JOB_NAME: &str = "dynwork_chats_retention";
pub const BATCH: u32 = 200;
pub const MAX_BATCHES: u32 = 100;
pub const EVENT_RETENTION: Duration = Duration::hours(24);

/// What one run did: events pruned, threads tombstoned, threads it left alone
/// because they changed under it, and whether `MAX_BATCHES` ended it with more
/// waiting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sweep {
    pub pruned: u64,
    pub tombstoned: u64,
    pub skipped: u64,
    pub more: bool,
}

/// One run: the window (none keeps everything), the time it runs at, and its
/// size limits.
#[derive(Debug, Clone, Copy)]
pub struct Run {
    pub window: Option<Duration>,
    pub now: DateTime<Utc>,
    pub batch: u32,
    pub max_batches: u32,
}

pub async fn sweep(
    store: &dyn ChatStore,
    hints: &dyn Hints,
    run: Run,
) -> Result<Sweep, StoreError> {
    let pruned = store.prune_events(run.now - EVENT_RETENTION).await?;
    let mut touched = Vec::new();
    let swept = walk(store, run, &mut touched).await;
    for user in &touched {
        hints.changed(user, None).await;
    }
    swept.map(|done| Sweep { pruned, ..done })
}

async fn walk(
    store: &dyn ChatStore,
    run: Run,
    touched: &mut Vec<UserId>,
) -> Result<Sweep, StoreError> {
    let Run {
        window,
        now,
        batch,
        max_batches,
    } = run;
    let mut sweep = Sweep::default();
    let Some(window) = window else {
        return Ok(sweep);
    };
    let cutoff = now - window;
    for _ in 0..max_batches {
        let idle = store.idle_threads(cutoff, batch).await?;
        if idle.is_empty() {
            return Ok(sweep);
        }
        let before = sweep.tombstoned;
        for thread in idle {
            match store
                .delete_thread(&thread.user, &thread.id, thread.revision, now)
                .await?
            {
                WriteOutcome::Done(_) => {
                    sweep.tombstoned += 1;
                    if !touched.contains(&thread.user) {
                        touched.push(thread.user);
                    }
                },
                WriteOutcome::Conflict(_) | WriteOutcome::NotFound => sweep.skipped += 1,
            }
        }
        if sweep.tombstoned == before {
            return Ok(sweep);
        }
    }
    sweep.more = true;
    Ok(sweep)
}

#[derive(Debug, thiserror::Error)]
pub enum RetentionError {
    #[error(transparent)]
    Dependency(#[from] MissingDependency),
    #[error("the {CONFIG_PREFIX} config could not be loaded")]
    Config(#[from] ConfigError),
    #[error("the chat store failed")]
    Store(#[from] StoreError),
}

impl From<RetentionError> for ProviderError {
    fn from(error: RetentionError) -> Self {
        Self::Internal(Box::new(error))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChatsRetentionJob;

impl ChatsRetentionJob {
    async fn run(ctx: &JobContext) -> Result<JobResult, RetentionError> {
        let extension = ctx.get::<Arc<dyn ExtensionContext>>()?;
        let config = ChatsConfig::load(&extension.services_root().path).await?;
        let window = config.inactivity();
        let pool = ctx.get::<DbPool>()?.write_pool();
        let store = PgChatStore::new(Arc::clone(&pool));
        let hints = RelayHints::new(pool);
        let run = Run {
            window,
            now: SystemClock.now(),
            batch: BATCH,
            max_batches: MAX_BATCHES,
        };
        let done = sweep(&store, &hints, run).await?;
        let pruned = format!("{} reply event(s) older than a day pruned", done.pruned);
        let Some(window) = window else {
            return Ok(JobResult::success().with_stats(0, 0).with_message(format!(
                "{pruned}; inactive_days is not set: threads are kept"
            )));
        };
        let more = if done.more { "; more remain" } else { "" };
        Ok(JobResult::success()
            .with_stats(done.tombstoned, done.skipped)
            .with_message(format!(
                "{pruned}; {} thread(s) idle for more than {} day(s) tombstoned, {} left alone{more}",
                done.tombstoned,
                window.num_days(),
                done.skipped
            )))
    }
}

#[async_trait::async_trait]
impl Job for ChatsRetentionJob {
    fn name(&self) -> &'static str {
        JOB_NAME
    }

    fn description(&self) -> &'static str {
        "Prunes old cloud chat reply events and tombstones threads idle beyond inactive_days"
    }

    fn schedule(&self) -> &'static str {
        "0 30 3 * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id, "Chats retention invoked");
        Ok(Self::run(ctx).await?)
    }
}

systemprompt::traits::submit_job!(&ChatsRetentionJob);
