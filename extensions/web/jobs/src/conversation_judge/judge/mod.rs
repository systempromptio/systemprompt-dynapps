//! The judge: one structured-output call per conversation through the
//! process's `AiService`, audited as this job's actor. It labels what the
//! deterministic record cannot — a title, a summary, the intent behind the
//! conversation and one 0–100 verdict on whether that intent was completed.
//! Its requests run under a fixed job context, never the analysed
//! conversation's id, which would grow that conversation by one request per
//! judgement and re-queue it forever. The response shape and its
//! normalisation live in `classification`.

use std::sync::Arc;

use systemprompt::ai::{AiMessage, AiRequest, AiService, StructuredOutputOptions};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId};
use systemprompt::models::execution::context::RequestContext;

use crate::JobError;

use super::classifier::{ConversationClassifier, JudgeVerdict};

mod classification;

pub use classification::{
    Category, Classification, Outcome, classification_schema, parse_classification,
};

pub(super) const AGENT_NAME: &str = "conversation-judge";

const JOB_CONTEXT_SEED: &[u8] = b"astound:conversation_judge";

const SYSTEM_PROMPT: &str = "\
You judge one conversation between a person and an AI assistant, recorded through an \
enterprise AI gateway. Read the transcript, work out what the person originally asked for, \
and decide whether they got it.

title: at most 8 words naming what the conversation was about, as a person would label it \
in a list (for example \"Fix checkout tax rounding\"). No trailing punctuation, no quotes.

completion: 0 to 100 — how completely the assistant delivered what the person ORIGINALLY \
asked for, judged from the first user prompt against the final state of the conversation. \
100: fully done and confirmed or evidently correct. 75: done with minor gaps the person \
accepted. 50: half done, or done for a narrower ask than the original. 25: barely advanced \
or the person had to redo it elsewhere. 0: nothing delivered, wrong direction, or the \
assistant refused. A conversation the person abandoned scores what was delivered before \
they left. Ignore politeness and effort; score the result.

completion_rationale: one or two sentences, at most 60 words, saying what was asked and \
what the evidence for the score is. Never quote credentials, keys or personal data.

category — the intent, the SINGLE best fit:
- development: writing, fixing, reviewing, testing or deploying software
- business-analysis: requirements, user stories, epics, sprints, tickets, impact analysis, \
release or change management, stakeholder documentation
- operations: infrastructure, monitoring, incidents, databases, CI/CD pipelines, environments
- admin-config: administering the platform itself — users, groups, access, plugins, \
settings, gateway or MCP configuration
- writing-comms: emails, meeting notes, summaries, presentations, announcements, translation
- research-learning: exploring a topic, asking how something works, comparing options, learning
- other: none of the above, or too little content to tell

outcome: achieved (the person got what they asked for), partial (some of it), abandoned \
(they stopped before getting it), unclear (cannot tell from the transcript).

summary: one paragraph, at most 60 words, in the third person, describing what the person \
wanted and what happened. Never quote credentials, keys or personal data.

tags: up to 8 short lowercase labels that add detail beyond the category (for example \
\"jira\", \"sql\", \"sfcc\", \"onboarding\").

skills_observed: the names of any skills or slash commands the person invoked, exactly as \
they appear (for example \"ba-story-drafting\"). Empty if none.

confidence: 0 to 1, how sure you are of the category.";

#[must_use]
pub(super) fn job_context_id() -> ContextId {
    ContextId::from_uuid(uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_OID,
        JOB_CONTEXT_SEED,
    ))
}

pub(super) struct Judge {
    pub(super) ai: Arc<AiService>,
    pub(super) actor: Actor,
    pub(super) provider: String,
    pub(super) model: String,
    pub(super) max_output_tokens: u32,
}

impl Judge {
    fn request_context(&self) -> Result<RequestContext, JobError> {
        Ok(RequestContext::new(
            SessionId::generate(),
            TraceId::new(uuid::Uuid::new_v4().to_string()),
            job_context_id(),
            AgentName::try_new(AGENT_NAME)?,
            self.actor.clone(),
        ))
    }
}

#[async_trait::async_trait]
impl ConversationClassifier for Judge {
    async fn classify(&self, transcript: &str) -> Result<JudgeVerdict, JobError> {
        let request = AiRequest::builder(
            vec![
                AiMessage::system(SYSTEM_PROMPT),
                AiMessage::user(transcript),
            ],
            self.provider.as_str(),
            self.model.as_str(),
            self.max_output_tokens,
            self.request_context()?,
        )
        .with_structured_output(StructuredOutputOptions::with_schema(classification_schema()))
        .build();
        let response = self.ai.generate(&request).await?;
        let classification = parse_classification(&response.content)?;
        Ok(JudgeVerdict {
            classification,
            ai_request_id: response.request_id.to_string(),
            input_tokens: response.input_tokens.and_then(|t| i32::try_from(t).ok()),
            output_tokens: response.output_tokens.and_then(|t| i32::try_from(t).ok()),
        })
    }
}
