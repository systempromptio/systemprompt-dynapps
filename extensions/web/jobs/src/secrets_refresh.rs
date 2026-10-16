//! `secrets_refresh` job: re-reads an external secrets source on the
//! profile's `secrets.external.refresh_secs` cadence.
//!
//! A job's cron is fixed at compile time and the interval is profile data, so
//! the job wakes every minute and reloads only once the interval has passed
//! since the last load or reload. Every other source has no interval and the
//! run is a no-op. A reload that changes nothing swaps nothing; one that
//! changes a pinned key is refused by core and reported here as a failure.
//! Only key names are ever logged. A reload that changed keys leaves a
//! `user_activity` row under the job's configured owner.
//!
//! The secrets live in each process, so the job is node-scoped: every replica
//! reloads its own copy rather than the one that wins the cluster lock.

use chrono::{DateTime, Utc};
use systemprompt::config::{ReloadOutcome, SecretsBootstrap};
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult, JobScope};
use systemprompt_web_admin_core::activity::NewActivity;
use systemprompt_web_admin_core::repositories::user_activity;

#[derive(Debug, Clone, Copy, Default)]
pub struct SecretsRefreshJob;

#[must_use]
pub fn reload_due(last: DateTime<Utc>, now: DateTime<Utc>, interval: std::time::Duration) -> bool {
    let elapsed = now.signed_duration_since(last).to_std().unwrap_or_default();
    elapsed >= interval
}

impl SecretsRefreshJob {
    pub async fn run_once(ctx: &JobContext) -> JobResult {
        let Some(interval) = SecretsBootstrap::refresh_interval() else {
            return JobResult::success().with_message("no external secrets refresh configured");
        };
        if let Some(status) = SecretsBootstrap::status() {
            let last = status.last_reload_at.unwrap_or(status.initialised_at);
            if !reload_due(last, Utc::now(), interval) {
                return JobResult::success().with_message("secrets refresh not due");
            }
        }
        let start = std::time::Instant::now();
        let outcome = SecretsBootstrap::reload().await;
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        match outcome {
            Ok(outcome) => {
                record_change(ctx, &outcome).await;
                tracing::info!(
                    generation = outcome.generation,
                    swapped = outcome.swapped,
                    changed_keys = ?outcome.changed_keys,
                    duration_ms,
                    "Secrets refreshed from external source"
                );
                JobResult::success()
                    .with_stats(outcome.changed_keys.len() as u64, 0)
                    .with_duration(duration_ms)
                    .with_message(format!(
                        "generation {} ({} key(s) changed)",
                        outcome.generation,
                        outcome.changed_keys.len()
                    ))
            },
            Err(error) => {
                tracing::warn!(error = %error, duration_ms, "Secrets refresh failed; previous secrets kept");
                JobResult::failure(error.to_string()).with_duration(duration_ms)
            },
        }
    }
}

async fn record_change(ctx: &JobContext, outcome: &ReloadOutcome) {
    if !outcome.swapped {
        return;
    }
    let Ok(db) = ctx.get::<DbPool>() else {
        return;
    };
    let activity = NewActivity::secrets_reloaded(
        &ctx.actor().user_id,
        outcome.generation,
        &outcome.changed_keys,
    );
    user_activity::record(&db.write_pool(), activity).await;
}

#[async_trait::async_trait]
impl Job for SecretsRefreshJob {
    fn name(&self) -> &'static str {
        "secrets_refresh"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Reloads secrets from the external source every secrets.external.refresh_secs"
    }

    fn schedule(&self) -> &'static str {
        "0 * * * * *"
    }

    fn scope(&self) -> JobScope {
        JobScope::Node
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(Self::run_once(ctx).await)
    }
}

systemprompt::traits::submit_job!(&SecretsRefreshJob);
