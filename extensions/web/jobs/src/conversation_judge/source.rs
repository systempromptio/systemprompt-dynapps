//! Judges one leased conversation: reads it through the same readers the
//! conversation page uses, renders the transcript and asks the classifier.
//! A conversation with no readable content is recorded as unreadable without
//! an inference call.

use sqlx::PgPool;
use systemprompt::identifiers::ContextId;
use systemprompt_web_admin_analytics::repositories::analytics::context_detail::{
    list_context_requests, list_messages_for_requests,
};
use systemprompt_web_admin_analytics::repositories::analytics::context_tool_calls::list_tool_calls_for_requests;
use systemprompt_web_admin_analytics::test_support::{
    ConversationView, TranscriptOptions, build_conversation, transcript_request_ids,
};

use super::Tick;
use super::classifier::JudgeVerdict;
use super::judge::Classification;
use super::repository::{AnalysisLease, ConversationJudgeRepository, SourceMeta};
use super::transcript::{TranscriptMeta, render_transcript};
use crate::JobError;

pub(super) struct Judgement {
    pub(super) classification: Classification,
    pub(super) ai_request_id: Option<String>,
    pub(super) input_tokens: Option<i32>,
    pub(super) output_tokens: Option<i32>,
}

impl Judgement {
    fn unreadable() -> Self {
        Self {
            classification: Classification::unreadable(),
            ai_request_id: None,
            input_tokens: None,
            output_tokens: None,
        }
    }
}

impl From<JudgeVerdict> for Judgement {
    fn from(verdict: JudgeVerdict) -> Self {
        Self {
            classification: verdict.classification,
            ai_request_id: Some(verdict.ai_request_id),
            input_tokens: verdict.input_tokens,
            output_tokens: verdict.output_tokens,
        }
    }
}

pub(super) async fn judge_conversation(
    tick: &Tick<'_>,
    lease: &AnalysisLease,
    meta: &SourceMeta,
) -> Result<Judgement, JobError> {
    let transcript = render_judge_transcript(
        tick.pool,
        &tick.repo,
        &lease.context_id,
        meta,
        tick.params.transcript_token_budget,
    )
    .await?;
    let Some(transcript) = transcript else {
        return Ok(Judgement::unreadable());
    };
    let verdict = tick.classifier.classify(&transcript).await?;
    Ok(Judgement::from(verdict))
}

async fn render_judge_transcript(
    pool: &PgPool,
    repo: &ConversationJudgeRepository,
    context_id: &ContextId,
    meta: &SourceMeta,
    token_budget: usize,
) -> Result<Option<String>, JobError> {
    let view = read_conversation(pool, context_id).await?;
    if !view.has_content {
        return Ok(None);
    }
    let hooked_skills = repo.list_hooked_skills(context_id).await?;
    let transcript_meta = transcript_meta(meta, hooked_skills);
    Ok(Some(render_transcript(
        &transcript_meta,
        &view,
        token_budget,
    )))
}

async fn read_conversation(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<ConversationView, JobError> {
    let requests = list_context_requests(pool, context_id).await?;
    let ids = transcript_request_ids(&requests);
    let messages = list_messages_for_requests(pool, &ids.messages).await?;
    let tool_calls = list_tool_calls_for_requests(pool, &ids.tool_calls).await?;
    Ok(build_conversation(
        &messages,
        &tool_calls,
        &requests,
        TranscriptOptions::owner_facing(),
    ))
}

fn transcript_meta(meta: &SourceMeta, hooked_skills: Vec<String>) -> TranscriptMeta {
    TranscriptMeta {
        client_kind: meta.client_kind.clone(),
        model: meta.model.clone(),
        turn_count: meta.turn_count,
        tool_call_count: meta.tool_call_count,
        error_count: meta.error_count,
        duration_minutes: duration_minutes(meta),
        hooked_skills,
    }
}

fn duration_minutes(meta: &SourceMeta) -> i64 {
    match (meta.first_at, meta.last_at) {
        (Some(first), Some(last)) => (last - first).num_minutes().max(0),
        _ => 0,
    }
}
