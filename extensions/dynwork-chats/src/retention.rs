//! The inactivity job (decision D4).
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
//! `inactive_days` applies from the next one. [`sweep`] tombstones the threads
//! idle for longer than a window at a given time; with no window it does
//! nothing and reads nothing.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use systemprompt::database::DbPool;
use systemprompt::extension::ExtensionContext;
use systemprompt::traits::{Job, JobContext, JobResult, MissingDependency, ProviderError};

use crate::clock::{Clock, SystemClock};
use crate::config::{CONFIG_PREFIX, ChatsConfig, ConfigError};
use crate::store::postgres::PgChatStore;
use crate::store::{ChatStore, StoreError, WriteOutcome};

pub const JOB_NAME: &str = "dynwork_chats_retention";
pub const BATCH: u32 = 200;
pub const MAX_BATCHES: u32 = 100;

/// What one run did: threads tombstoned, threads it left alone because they
/// changed under it, and whether `MAX_BATCHES` ended it with more waiting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sweep {
    pub tombstoned: u64,
    pub skipped: u64,
    pub more: bool,
}

pub async fn sweep(
    store: &dyn ChatStore,
    window: Option<Duration>,
    now: DateTime<Utc>,
    batch: u32,
    max_batches: u32,
) -> Result<Sweep, StoreError> {
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
                WriteOutcome::Done(_) => sweep.tombstoned += 1,
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
        let Some(window) = config.inactivity() else {
            return Ok(JobResult::success()
                .with_stats(0, 0)
                .with_message("inactive_days is not set: threads are kept"));
        };
        let pool = ctx.get::<DbPool>()?.write_pool();
        let store = PgChatStore::new(pool);
        let done = sweep(&store, Some(window), SystemClock.now(), BATCH, MAX_BATCHES).await?;
        let more = if done.more { "; more remain" } else { "" };
        Ok(JobResult::success()
            .with_stats(done.tombstoned, done.skipped)
            .with_message(format!(
                "{} thread(s) idle for more than {} day(s) tombstoned, {} left alone{more}",
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
        "Tombstones cloud chat threads idle for longer than the instance's inactive_days"
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
