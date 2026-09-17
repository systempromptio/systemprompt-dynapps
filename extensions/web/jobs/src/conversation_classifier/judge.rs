//! The judge: one structured-output call per conversation through the
//! process's `AiService`, audited as this job's actor. The response shape is
//! enforced by the provider API through the JSON schema below; parsing here
//! only normalises what came back.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use systemprompt::ai::{AiMessage, AiRequest, AiService, StructuredOutputOptions};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId};
use systemprompt::models::execution::context::RequestContext;

use crate::JobError;

pub(super) const AGENT_NAME: &str = "conversation-classifier";

// Why: the judge's own requests need a context of their own. Using the
// analysed conversation's id would grow that conversation by one request per
// classification and re-queue it forever.
const JOB_CONTEXT_SEED: &[u8] = b"astound:conversation_classifier";

const SYSTEM_PROMPT: &str = "\
You classify one conversation between a person and an AI assistant, recorded through an \
enterprise AI gateway. Read the transcript and decide what the person was trying to do.

category — the SINGLE best fit:
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Development,
    BusinessAnalysis,
    Operations,
    AdminConfig,
    WritingComms,
    ResearchLearning,
    Other,
}

impl Category {
    pub const ALL: [Self; 7] = [
        Self::Development,
        Self::BusinessAnalysis,
        Self::Operations,
        Self::AdminConfig,
        Self::WritingComms,
        Self::ResearchLearning,
        Self::Other,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::BusinessAnalysis => "business-analysis",
            Self::Operations => "operations",
            Self::AdminConfig => "admin-config",
            Self::WritingComms => "writing-comms",
            Self::ResearchLearning => "research-learning",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Achieved,
    Partial,
    Abandoned,
    Unclear,
}

impl Outcome {
    pub const ALL: [Self; 4] = [
        Self::Achieved,
        Self::Partial,
        Self::Abandoned,
        Self::Unclear,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Achieved => "achieved",
            Self::Partial => "partial",
            Self::Abandoned => "abandoned",
            Self::Unclear => "unclear",
        }
    }
}

/// The judge's verdict after normalisation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Classification {
    pub category: Category,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub outcome: Outcome,
    #[serde(default)]
    pub skills_observed: Vec<String>,
    #[serde(default)]
    pub confidence: f32,
}

impl Classification {
    // Why: what an unreadable conversation is recorded as, so it is not
    // retried forever.
    #[must_use]
    pub fn unreadable() -> Self {
        Self {
            category: Category::Other,
            summary: "No readable transcript was retained for this conversation.".to_owned(),
            tags: Vec::new(),
            outcome: Outcome::Unclear,
            skills_observed: Vec::new(),
            confidence: 0.0,
        }
    }
}

const MAX_TAGS: usize = 8;
const MAX_SKILLS: usize = 20;
const MAX_SUMMARY_CHARS: usize = 600;

// JSON: the response schema handed to the provider API; Gemini's
// responseSchema is an OpenAPI subset, so enums and `required` are all it
// uses and core's sanitizer strips anything richer.
#[must_use]
pub fn classification_schema() -> serde_json::Value {
    let categories: Vec<&str> = Category::ALL.iter().map(|c| c.as_str()).collect();
    let outcomes: Vec<&str> = Outcome::ALL.iter().map(|o| o.as_str()).collect();
    serde_json::json!({
        "type": "object",
        "properties": {
            "category": {"type": "string", "enum": categories},
            "summary": {"type": "string"},
            "tags": {"type": "array", "items": {"type": "string"}},
            "outcome": {"type": "string", "enum": outcomes},
            "skills_observed": {"type": "array", "items": {"type": "string"}},
            "confidence": {"type": "number"}
        },
        "required": ["category", "summary", "tags", "outcome", "skills_observed", "confidence"]
    })
}

// Why: clamps every list and number into the bounds the table expects. An
// unknown category or outcome is a parse error the caller retries; the
// API-enforced schema makes that rare.
pub fn parse_classification(content: &str) -> Result<Classification, serde_json::Error> {
    let mut parsed: Classification = serde_json::from_str(content)?;
    parsed.summary = parsed
        .summary
        .trim()
        .chars()
        .take(MAX_SUMMARY_CHARS)
        .collect();
    parsed.tags = dedupe_lower(parsed.tags, MAX_TAGS);
    parsed.skills_observed = dedupe_lower(parsed.skills_observed, MAX_SKILLS);
    parsed.confidence = parsed.confidence.clamp(0.0, 1.0);
    Ok(parsed)
}

fn dedupe_lower(values: Vec<String>, cap: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for value in values {
        let v = value.trim().to_lowercase();
        if v.is_empty() || out.contains(&v) {
            continue;
        }
        out.push(v);
        if out.len() == cap {
            break;
        }
    }
    out
}

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

pub(super) struct Verdict {
    pub(super) classification: Classification,
    pub(super) ai_request_id: String,
}

impl Judge {
    // Why: an empty session id keeps core from binding the judge call to a
    // user session; the job context id keeps it out of every conversation.
    fn request_context(&self) -> Result<RequestContext, JobError> {
        Ok(RequestContext::new(
            SessionId::new(""),
            TraceId::new(uuid::Uuid::new_v4().to_string()),
            job_context_id(),
            AgentName::try_new(AGENT_NAME)?,
        )
        .with_actor(self.actor.clone()))
    }

    pub(super) async fn classify(&self, transcript: &str) -> Result<Verdict, JobError> {
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
        Ok(Verdict {
            classification,
            ai_request_id: response.request_id.to_string(),
        })
    }
}
