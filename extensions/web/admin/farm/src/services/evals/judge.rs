//! The judged half of an eval: one structured-output call through the
//! process's `AiService` scoring a case's run against its rubric, built the
//! way the conversation judge job builds its requests.
//!
//! `StubJudge` stands in where no provider is configured and in tests.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::Deserialize;
use systemprompt::ai::{AiMessage, AiRequest, AiService, StructuredOutputOptions};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::models::execution::context::RequestContext;

use crate::error::{FarmError, FarmResult};

pub const JUDGE_AGENT_NAME: &str = "farm-eval-judge";
pub const DEFAULT_PROVIDER: &str = "gemini";
pub const DEFAULT_MODEL: &str = "gemini-3.8-flash";
const MAX_OUTPUT_TOKENS: u32 = 2_048;
const JUDGE_CONTEXT_SEED: &[u8] = b"dynapps:farm_eval_judge";

const SYSTEM_PROMPT: &str = "\
You grade one run of an autonomous coding agent against a rubric. You receive the rubric, \
the task prompt, the run transcript (assistant text, tool names, final result and the \
agent's output file) and a summary of the files it changed. Apply the rubric literally.

score: 0 to 100 as the rubric defines it. Score only what the evidence shows; a claim in \
the transcript that the output file does not back up earns nothing.
passed: true when score meets the threshold you are given.
rationale: two or three sentences, at most 80 words, naming the evidence for the score.
findings: short bullet strings, one per rubric item met or missed.
Never quote credentials, keys or personal data.";

pub type JudgeFuture<'a> = Pin<Box<dyn Future<Output = FarmResult<JudgeVerdict>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgeInput {
    pub rubric: String,
    pub case_prompt: String,
    pub transcript: String,
    pub diff_summary: String,
    pub threshold: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgeVerdict {
    pub score: i32,
    pub passed: bool,
    pub rationale: String,
    pub request_id: Option<String>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

// Why: lint-ok: the orchestrator holds the judge as `Arc<dyn EvalJudge>`, and
// a native `async fn` in a trait is not dyn-compatible, so the future is boxed.
pub trait EvalJudge: Send + Sync {
    fn judge<'a>(&'a self, input: &'a JudgeInput) -> JudgeFuture<'a>;
}

// Why: Gemini's responseSchema is an OpenAPI subset (enums and `required`
// only); core's sanitizer strips anything richer.
// JSON: outgoing fixed-shape
#[must_use]
pub fn verdict_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "score": {"type": "integer"},
            "passed": {"type": "boolean"},
            "rationale": {"type": "string"},
            "findings": {"type": "array", "items": {"type": "string"}}
        },
        "required": ["score", "passed", "rationale", "findings"]
    })
}

#[derive(Debug, Deserialize)]
struct RawVerdict {
    score: i64,
    rationale: String,
    #[serde(default)]
    findings: Vec<String>,
}

pub fn parse_verdict(content: &str, threshold: i32) -> FarmResult<(i32, bool, String)> {
    let raw: RawVerdict = serde_json::from_str(content).map_err(|source| FarmError::Json {
        at: "judge verdict".to_owned(),
        source,
    })?;
    let score = i32::try_from(raw.score.clamp(0, 100)).unwrap_or(0);
    let mut rationale = raw.rationale.trim().to_owned();
    for finding in raw.findings.iter().filter(|f| !f.trim().is_empty()) {
        rationale.push_str("\n- ");
        rationale.push_str(finding.trim());
    }
    Ok((score, score >= threshold, rationale))
}

#[must_use]
pub fn render_user_message(input: &JudgeInput) -> String {
    format!(
        "THRESHOLD: {}\n\nRUBRIC:\n{}\n\nTASK PROMPT:\n{}\n\nCHANGED FILES:\n{}\n\nTRANSCRIPT:\n{}",
        input.threshold, input.rubric, input.case_prompt, input.diff_summary, input.transcript
    )
}

#[must_use]
pub fn judge_context_id() -> ContextId {
    ContextId::from_uuid(uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_OID,
        JUDGE_CONTEXT_SEED,
    ))
}

pub struct GatewayJudge {
    pub ai: Arc<AiService>,
    pub actor: Actor,
    pub provider: String,
    pub model: String,
    pub max_output_tokens: u32,
}

impl std::fmt::Debug for GatewayJudge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayJudge")
            .field("actor", &self.actor)
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field("max_output_tokens", &self.max_output_tokens)
            .finish_non_exhaustive()
    }
}

impl GatewayJudge {
    #[must_use]
    pub fn new(ai: Arc<AiService>, user_id: UserId) -> Self {
        Self {
            ai,
            actor: Actor::job(user_id, JUDGE_AGENT_NAME),
            provider: DEFAULT_PROVIDER.to_owned(),
            model: DEFAULT_MODEL.to_owned(),
            max_output_tokens: MAX_OUTPUT_TOKENS,
        }
    }

    fn request_context(&self) -> FarmResult<RequestContext> {
        let agent =
            AgentName::try_new(JUDGE_AGENT_NAME).map_err(|source| FarmError::Identifier {
                at: "judge agent name",
                source: Box::new(source),
            })?;
        Ok(RequestContext::new(
            SessionId::generate(),
            TraceId::new(uuid::Uuid::new_v4().to_string()),
            judge_context_id(),
            agent,
            self.actor.clone(),
        ))
    }

    async fn judge_inner(&self, input: &JudgeInput) -> FarmResult<JudgeVerdict> {
        let request = AiRequest::builder(
            vec![
                AiMessage::system(SYSTEM_PROMPT),
                AiMessage::user(render_user_message(input)),
            ],
            self.provider.as_str(),
            self.model.as_str(),
            self.max_output_tokens,
            self.request_context()?,
        )
        .with_structured_output(StructuredOutputOptions::with_schema(verdict_schema()))
        .build();
        let response = self
            .ai
            .generate(&request)
            .await
            .map_err(|e| FarmError::Gateway(e.to_string()))?;
        let (score, passed, rationale) = parse_verdict(&response.content, input.threshold)?;
        Ok(JudgeVerdict {
            score,
            passed,
            rationale,
            request_id: Some(response.request_id.to_string()),
            input_tokens: response.input_tokens.and_then(|t| i32::try_from(t).ok()),
            output_tokens: response.output_tokens.and_then(|t| i32::try_from(t).ok()),
        })
    }
}

impl EvalJudge for GatewayJudge {
    fn judge<'a>(&'a self, input: &'a JudgeInput) -> JudgeFuture<'a> {
        Box::pin(self.judge_inner(input))
    }
}

#[derive(Debug, Clone)]
pub struct StubJudge {
    pub score: i32,
    pub rationale: String,
}

impl EvalJudge for StubJudge {
    fn judge<'a>(&'a self, input: &'a JudgeInput) -> JudgeFuture<'a> {
        let verdict = JudgeVerdict {
            score: self.score,
            passed: self.score >= input.threshold,
            rationale: self.rationale.clone(),
            request_id: None,
            input_tokens: None,
            output_tokens: None,
        };
        Box::pin(async move { Ok(verdict) })
    }
}
