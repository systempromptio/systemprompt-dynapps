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
//! `evaluation.automatic`, which switches off every background judge spend at
//! once. A manual `-p context_id=<id>` run bypasses the latter.

mod discovery;
mod judge;
mod repository;
mod transcript;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{Actor, ContextId};
use systemprompt::system::AppContext;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::repositories::analytics::context_detail::{
    list_context_requests, list_messages_for_requests,
};
use systemprompt_web_admin::repositories::analytics::context_tool_calls::list_tool_calls_for_requests;
use systemprompt_web_admin::test_support::{
    TranscriptOptions, build_conversation, transcript_request_ids,
};

use crate::JobError;

pub use judge::{Category, Classification, Outcome, classification_schema, parse_classification};
pub use transcript::{TranscriptMeta, render_transcript};

use discovery::DiscoveryWindow;
use repository::{AnalysisLease, Completion, ConversationJudgeRepository};

pub(crate) const JOB_NAME: &str = "conversation_judge";

// Why: Tunables, each overridable from the scheduler entry's `parameters:` or
// `systemprompt infra jobs run conversation_judge -p key=value`.
#[derive(Debug, Clone)]
pub(crate) struct JudgeParams {
    pub provider: String,
    pub model: String,
    pub batch_size: u32,
    pub daily_cost_cap_microdollars: i64,
    pub quiet_minutes: i32,
    pub lookback_days: i32,
    pub transcript_token_budget: usize,
    pub max_output_tokens: u32,
    // Why: one conversation forced through regardless of quiet time or
    // fingerprint — the operator's re-judge switch.
    pub context_id: Option<ContextId>,
}

impl Default for JudgeParams {
    fn default() -> Self {
        Self {
            provider: "gemini".to_owned(),
            model: "gemini-3.8-flash".to_owned(),
            batch_size: 20,
            daily_cost_cap_microdollars: 2_000_000,
            quiet_minutes: 30,
            lookback_days: 30,
            transcript_token_budget: 24_000,
            max_output_tokens: 2_048,
            context_id: None,
        }
    }
}

impl JudgeParams {
    fn from_context(ctx: &JobContext) -> Result<Self, JobError> {
        let defaults = Self::default();
        let parse = |key: &str| -> Result<Option<String>, JobError> {
            Ok(ctx.get_parameter_parsed::<String>(key)?)
        };
        let num = |key: &str| -> Result<Option<i64>, JobError> {
            Ok(ctx.get_parameter_parsed::<i64>(key)?)
        };
        let default_batch = i64::from(defaults.batch_size);
        let default_max_out = i64::from(defaults.max_output_tokens);
        let default_budget = i64::try_from(defaults.transcript_token_budget).unwrap_or(i64::MAX);
        let batch = num("batch_size")?.unwrap_or(default_batch);
        let max_out = num("max_output_tokens")?.unwrap_or(default_max_out);
        let budget = num("transcript_token_budget")?.unwrap_or(default_budget);
        Ok(Self {
            provider: parse("provider")?.unwrap_or(defaults.provider),
            model: parse("model")?.unwrap_or(defaults.model),
            batch_size: u32::try_from(batch.clamp(1, 100)).unwrap_or(defaults.batch_size),
            daily_cost_cap_microdollars: num("daily_cost_cap_microdollars")?
                .unwrap_or(defaults.daily_cost_cap_microdollars),
            quiet_minutes: num("quiet_minutes")?.map_or(defaults.quiet_minutes, |v| {
                i32::try_from(v.clamp(0, 525_600)).unwrap_or(0)
            }),
            lookback_days: num("lookback_days")?.map_or(defaults.lookback_days, |v| {
                i32::try_from(v.clamp(1, 3_660)).unwrap_or(30)
            }),
            transcript_token_budget: usize::try_from(budget.clamp(1_000, 400_000))
                .unwrap_or(defaults.transcript_token_budget),
            max_output_tokens: u32::try_from(max_out.clamp(256, 65_536))
                .unwrap_or(defaults.max_output_tokens),
            context_id: parse("context_id")?.map(ContextId::try_new).transpose()?,
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ConversationJudgeJob;

struct Tick<'a> {
    pool: &'a PgPool,
    repo: ConversationJudgeRepository,
    judge: judge::Judge,
    params: JudgeParams,
    // Why: the profile's automatic switch is off — no discovery, and only
    // rows a person queued from the console are judged.
    manual_only: bool,
}

enum Step {
    Judged,
    Failed,
    CapReached,
}

impl ConversationJudgeJob {
    async fn run_tick(tick: &Tick<'_>) -> Result<JobResult, JobError> {
        let repo = &tick.repo;
        repo.update_judge_costs().await?;
        if let Some(context_id) = &tick.params.context_id {
            if !repo.mark_for_rejudging(context_id).await? {
                return Ok(JobResult::success()
                    .with_message(format!("no gateway requests for context {context_id}")));
            }
        } else if tick.manual_only {
            tracing::debug!("conversation judge: automatic off, manual requests only");
        } else {
            let queued = repo
                .discover_and_enqueue(DiscoveryWindow {
                    quiet_minutes: tick.params.quiet_minutes,
                    lookback_days: tick.params.lookback_days,
                })
                .await?;
            tracing::debug!(queued, "conversation judge discovery");
        }

        let batch = if tick.params.context_id.is_some() {
            1
        } else {
            tick.params.batch_size
        };
        let leases = repo.claim(batch, tick.manual_only).await?;
        let (mut judged, mut failed) = (0u64, 0u64);
        for (index, lease) in leases.iter().enumerate() {
            match Self::judge_one(tick, lease).await {
                Ok(Step::Judged) => judged += 1,
                Ok(Step::Failed) => failed += 1,
                Ok(Step::CapReached) => {
                    repo.release(&leases[index..]).await?;
                    tracing::warn!(
                        cap_microdollars = tick.params.daily_cost_cap_microdollars,
                        "conversation judge daily cost cap reached; leases released"
                    );
                    break;
                },
                Err(error) => {
                    failed += 1;
                    repo.retry(lease, &error.to_string()).await?;
                    tracing::warn!(%error, context_id = %lease.context_id, "conversation judgement will retry");
                },
            }
        }
        Ok(JobResult::success().with_stats(judged, failed))
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

        let requests = list_context_requests(tick.pool, &lease.context_id).await?;
        let ids = transcript_request_ids(&requests);
        let messages = list_messages_for_requests(tick.pool, &ids.messages).await?;
        let tool_calls = list_tool_calls_for_requests(tick.pool, &ids.tool_calls).await?;
        let view = build_conversation(
            &messages,
            &tool_calls,
            &requests,
            TranscriptOptions::owner_facing(),
        );

        let (classification, ai_request_id, input_tokens, output_tokens) = if view.has_content {
            let hooked = repo.list_hooked_skills(&lease.context_id).await?;
            let transcript_meta = TranscriptMeta {
                client_kind: meta.client_kind.clone(),
                model: meta.model.clone(),
                turn_count: meta.turn_count,
                tool_call_count: meta.tool_call_count,
                error_count: meta.error_count,
                duration_minutes: match (meta.first_at, meta.last_at) {
                    (Some(first), Some(last)) => (last - first).num_minutes().max(0),
                    _ => 0,
                },
                hooked_skills: hooked,
            };
            let transcript =
                render_transcript(&transcript_meta, &view, tick.params.transcript_token_budget);
            let verdict = tick.judge.classify(&transcript).await?;
            (
                verdict.classification,
                Some(verdict.ai_request_id),
                verdict.input_tokens,
                verdict.output_tokens,
            )
        } else {
            (Classification::unreadable(), None, None, None)
        };

        repo.complete(
            lease,
            Completion {
                classification: &classification,
                provider: &tick.params.provider,
                model: &tick.params.model,
                ai_request_id: ai_request_id.as_deref(),
                input_tokens,
                output_tokens,
                trigger: if tick.params.context_id.is_some() || lease.manual {
                    "manual"
                } else {
                    "automatic"
                },
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
            .evaluation
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
        let tick = Tick {
            pool: &pool,
            repo: ConversationJudgeRepository::new(PgPool::clone(&pool)),
            judge,
            manual_only: !automatic && params.context_id.is_none(),
            params,
        };
        Ok(Self::run_tick(&tick).await?)
    }
}

systemprompt::traits::submit_job!(&ConversationJudgeJob);
