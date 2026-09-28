//! `conversation_judge` job: the one AI label on top of the deterministic
//! conversation record.
//!
//! Each tick: queue every quiet or closed conversation with at least one turn
//! that has not been judged (or grew since it was), lease a batch, read each
//! transcript through the same readers the conversation page uses, and ask
//! the configured model — in one structured call — for a title, a summary,
//! the intent category, an outcome and a single 0–100 `completion` score:
//! did the assistant deliver what the person originally asked for. The
//! verdict lands in `conversation_analyses` and every Analysis page joins it.
//!
//! Judge calls go through the process's shared `AiService` as this job's
//! actor, so they are audited like any other request but are excluded from
//! every conversation view (`27_conversation_requests.sql`). A daily cost cap
//! stops the run — leases are handed back — rather than letting a backlog
//! burn budget.
//!
//! Two gates: the scheduler entry's `enabled:` and the profile's
//! `judge.automatic`, which switches off every background judge spend at
//! once. A manual `-p context_id=<id>` run bypasses the latter.

mod classifier;
mod discovery;
mod judge;
mod params;
mod repository;
mod source;
mod transcript;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::database::DbPool;
use systemprompt::identifiers::Actor;
use systemprompt::system::AppContext;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::JobError;

pub use classifier::{ConversationClassifier, JudgeVerdict};
pub use judge::{Category, Classification, Outcome, classification_schema, parse_classification};
#[doc(hidden)]
pub use params::JudgeParams;
pub use transcript::{TranscriptMeta, render_transcript};

use discovery::DiscoveryWindow;
use repository::{AnalysisLease, Completion, ConversationJudgeRepository};
use source::judge_conversation;

pub(crate) const JOB_NAME: &str = "conversation_judge";

#[derive(Debug, Clone, Copy, Default)]
pub struct ConversationJudgeJob;

struct Tick<'a> {
    pool: &'a PgPool,
    repo: ConversationJudgeRepository,
    classifier: &'a dyn ConversationClassifier,
    params: JudgeParams,
    manual_only: bool,
}

impl Tick<'_> {
    const fn batch_size(&self) -> u32 {
        if self.params.context_id.is_some() {
            1
        } else {
            self.params.batch_size
        }
    }

    const fn trigger(&self, lease: &AnalysisLease) -> &'static str {
        if self.params.context_id.is_some() || lease.manual {
            "manual"
        } else {
            "automatic"
        }
    }
}

#[doc(hidden)]
pub async fn run_with_classifier(
    pool: &PgPool,
    params: JudgeParams,
    manual_only: bool,
    classifier: &dyn ConversationClassifier,
) -> Result<JobResult, JobError> {
    let tick = Tick {
        pool,
        repo: ConversationJudgeRepository::new(pool.clone()),
        classifier,
        params,
        manual_only,
    };
    ConversationJudgeJob::run_tick(&tick).await
}

enum Step {
    Judged,
    Failed,
    CapReached,
}

impl ConversationJudgeJob {
    async fn run_tick(tick: &Tick<'_>) -> Result<JobResult, JobError> {
        tick.repo.update_judge_costs().await?;
        if let Some(message) = Self::enqueue(tick).await? {
            return Ok(JobResult::success().with_message(message));
        }
        let leases = tick
            .repo
            .claim(
                tick.batch_size(),
                tick.manual_only,
                tick.params.context_id.as_ref(),
            )
            .await?;
        let (judged, failed) = Self::judge_leases(tick, &leases).await?;
        Ok(JobResult::success().with_stats(judged, failed))
    }

    async fn enqueue(tick: &Tick<'_>) -> Result<Option<String>, JobError> {
        if let Some(context_id) = &tick.params.context_id {
            if !tick.repo.mark_for_rejudging(context_id).await? {
                return Ok(Some(format!(
                    "no gateway requests for context {context_id}"
                )));
            }
        } else if tick.manual_only {
            tracing::debug!("conversation judge: automatic off, manual requests only");
        } else {
            Self::discover(tick).await?;
        }
        Ok(None)
    }

    async fn discover(tick: &Tick<'_>) -> Result<(), JobError> {
        let queued = tick
            .repo
            .discover_and_enqueue(DiscoveryWindow {
                quiet_minutes: tick.params.quiet_minutes,
                lookback_days: tick.params.lookback_days,
            })
            .await?;
        tracing::debug!(queued, "conversation judge discovery");
        Ok(())
    }

    async fn judge_leases(
        tick: &Tick<'_>,
        leases: &[AnalysisLease],
    ) -> Result<(u64, u64), JobError> {
        let (mut judged, mut failed) = (0u64, 0u64);
        for (index, lease) in leases.iter().enumerate() {
            match Self::judge_one(tick, lease).await {
                Ok(Step::Judged) => judged += 1,
                Ok(Step::Failed) => failed += 1,
                Ok(Step::CapReached) => {
                    Self::release_at_cap(tick, &leases[index..]).await?;
                    break;
                },
                Err(error) => {
                    failed += 1;
                    Self::retry_later(tick, lease, &error).await?;
                },
            }
        }
        Ok((judged, failed))
    }

    async fn release_at_cap(tick: &Tick<'_>, remaining: &[AnalysisLease]) -> Result<(), JobError> {
        tick.repo.release(remaining).await?;
        tracing::warn!(
            cap_microdollars = tick.params.daily_cost_cap_microdollars,
            "conversation judge daily cost cap reached; leases released"
        );
        Ok(())
    }

    async fn retry_later(
        tick: &Tick<'_>,
        lease: &AnalysisLease,
        error: &JobError,
    ) -> Result<(), JobError> {
        tick.repo.retry(lease, &error.to_string()).await?;
        tracing::warn!(%error, context_id = %lease.context_id, "conversation judgement will retry");
        Ok(())
    }

    async fn judge_one(tick: &Tick<'_>, lease: &AnalysisLease) -> Result<Step, JobError> {
        let repo = &tick.repo;
        let spend = repo.get_job_spend_today(JOB_NAME).await?;
        if spend >= tick.params.daily_cost_cap_microdollars {
            return Ok(Step::CapReached);
        }
        let Some(meta) = repo.find_source_meta(&lease.context_id).await? else {
            repo.retry(lease, "conversation has no readable requests")
                .await?;
            return Ok(Step::Failed);
        };
        let judgement = judge_conversation(tick, lease, &meta).await?;
        repo.complete(
            lease,
            Completion {
                classification: &judgement.classification,
                provider: &tick.params.provider,
                model: &tick.params.model,
                ai_request_id: judgement.ai_request_id.as_deref(),
                input_tokens: judgement.input_tokens,
                output_tokens: judgement.output_tokens,
                trigger: tick.trigger(lease),
                fingerprint: meta.fingerprint,
            },
        )
        .await?;
        Ok(Step::Judged)
    }
}

#[async_trait::async_trait]
impl Job for ConversationJudgeJob {
    fn name(&self) -> &'static str {
        JOB_NAME
    }

    fn description(&self) -> &'static str {
        "Labels each finished conversation with a title, summary, intent and one completion score"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn schedule(&self) -> &'static str {
        "0 */5 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let app = ctx
            .app_context::<Arc<AppContext>>()
            .ok_or(JobError::MissingContext("AppContext"))?;
        let params = JudgeParams::from_context(ctx)?;
        let automatic = ProfileBootstrap::get()
            .map_err(JobError::from)?
            .judge
            .automatic;
        let Some(ai) = app.ai_service_arc() else {
            return Ok(
                JobResult::success().with_message("no AI provider is configured; judge idle")
            );
        };
        let judge = judge::Judge {
            ai,
            actor: Actor::job(ctx.actor().user_id.clone(), JOB_NAME),
            provider: params.provider.clone(),
            model: params.model.clone(),
            max_output_tokens: params.max_output_tokens,
        };
        let pool = db.write_pool();
        let manual_only = !automatic && params.context_id.is_none();
        Ok(run_with_classifier(&pool, params, manual_only, &judge).await?)
    }
}

systemprompt::traits::submit_job!(&ConversationJudgeJob);
