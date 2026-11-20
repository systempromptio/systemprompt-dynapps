//! The model a reply streams from, behind [`ReplyModel`].
//!
//! [`GatewayModel`] answers through core's `AiService::generate_events_stream`
//! (decision D1): text deltas become text chunks, and the usage the stream
//! reports at its start and in later updates is folded into one usage chunk at
//! the message stop. Thinking and tool events are not chunk kinds yet; nothing
//! else in the crate names the AI service.
//! A provider's finish reason maps to `OpenCode`'s `finish`; as `rawFinish`
//! the known ones are kept and anything else becomes `other`, so no provider
//! text reaches the log. A model error likewise reaches the client as its kind
//! and a fixed message ([`failure`]); the provider's own text, which can echo
//! the request, goes to the server log only.

use std::fmt;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use systemprompt::ai::error::AiError;
use systemprompt::ai::{AiMessage, AiRequest, AiService};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::models::execution::context::RequestContext;
use systemprompt::wire::canonical::{CanonicalEvent, CanonicalStopReason, CanonicalUsage};

use crate::ids::ThreadId;
use crate::store::{ModelRef, Role};

pub const AGENT_NAME: &str = "dynwork-chats";
const CONTEXT_SEED: &str = "dynapps:dynwork-chats:";

/// The token counts of one model call, as the provider reported them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub raw_finish: Option<&'static str>,
}

/// Why a reply could not go on, as the client sees it: `kind` is the error's
/// `type`, `message` is safe to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplyError {
    pub kind: String,
    pub message: String,
}

impl ReplyError {
    pub fn new(kind: &str, message: impl Into<String>) -> Self {
        Self {
            kind: kind.to_owned(),
            message: message.into(),
        }
    }
}

/// What a model stream yields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelChunk {
    Text(String),
    Usage(Usage),
    Error(ReplyError),
}

pub type ChunkStream = Pin<Box<dyn Stream<Item = ModelChunk> + Send>>;

/// One message of the history sent to the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub role: Role,
    pub text: String,
}

/// A reply to generate: who asks, in which thread, with which model, and the
/// thread's text so far (the new prompt last).
#[derive(Debug, Clone)]
pub struct ReplyRequest {
    pub user: UserId,
    pub thread: ThreadId,
    pub model: ModelRef,
    pub history: Vec<Turn>,
}

/// A model to stream replies from. `#[async_trait]` because the runner holds
/// it as `Arc<dyn ReplyModel>`: the instance's AI service in production, a
/// scripted fake in the suites.
#[async_trait]
pub trait ReplyModel: Send + Sync + fmt::Debug {
    fn default_model(&self) -> Option<ModelRef>;

    async fn stream(&self, request: ReplyRequest) -> Result<ChunkStream, ReplyError>;
}

/// [`ReplyModel`] over core's `AiService`, under the caller's identity.
///
/// Every reply is audited in `ai_requests` as the caller, and the route admits
/// its model first (`model_not_allowed`). It does not pass the gateway's
/// quota and rate windows, guards or safety scanners: those sit on the
/// gateway's HTTP routes, not on `AiService` (stage 2).
pub struct GatewayModel {
    ai: Arc<AiService>,
}

impl fmt::Debug for GatewayModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GatewayModel").finish_non_exhaustive()
    }
}

impl GatewayModel {
    pub fn shared(ai: Arc<AiService>) -> Arc<dyn ReplyModel> {
        Arc::new(Self { ai })
    }

    fn request(&self, request: &ReplyRequest) -> Result<AiRequest, ReplyError> {
        let agent = AgentName::try_new(AGENT_NAME).map_err(|source| {
            tracing::error!(error = %source, "dynwork chats agent name refused");
            ReplyError::new("internal", "the reply could not be prepared")
        })?;
        let context = RequestContext::new(
            SessionId::generate(),
            TraceId::new(uuid::Uuid::new_v4().to_string()),
            thread_context(&request.thread),
            agent,
            Actor::user(request.user.clone()),
        );
        let messages = request
            .history
            .iter()
            .map(|turn| match turn.role {
                Role::User => AiMessage::user(turn.text.clone()),
                Role::Assistant => AiMessage::assistant(turn.text.clone()),
            })
            .collect();
        Ok(AiRequest::builder(
            messages,
            request.model.provider_id.as_str(),
            request.model.model_id.as_str(),
            self.ai.default_max_output_tokens(),
            context,
        )
        .build())
    }
}

fn thread_context(thread: &ThreadId) -> ContextId {
    let seed = format!("{CONTEXT_SEED}{thread}");
    ContextId::from_uuid(uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_OID,
        seed.as_bytes(),
    ))
}

#[async_trait]
impl ReplyModel for GatewayModel {
    fn default_model(&self) -> Option<ModelRef> {
        let provider = self.ai.default_provider();
        let model = self.ai.default_model();
        (!provider.is_empty() && !model.is_empty()).then(|| ModelRef {
            provider_id: provider.to_owned(),
            model_id: model.to_owned(),
        })
    }

    async fn stream(&self, request: ReplyRequest) -> Result<ChunkStream, ReplyError> {
        let ai_request = self.request(&request)?;
        let events = self
            .ai
            .generate_events_stream(&ai_request)
            .await
            .map_err(|error| failure(&error))?;
        let mut usage = CanonicalUsage::default();
        Ok(Box::pin(events.filter_map(move |item| {
            std::future::ready(match item {
                Ok(CanonicalEvent::TextDelta { text, .. }) => Some(ModelChunk::Text(text)),
                Ok(CanonicalEvent::MessageStart { usage: start, .. }) => {
                    usage = start;
                    None
                },
                Ok(CanonicalEvent::UsageDelta(update)) => {
                    update.apply_to(&mut usage);
                    None
                },
                Ok(CanonicalEvent::MessageStop {
                    stop_reason,
                    raw_finish_reason,
                    ..
                }) => Some(ModelChunk::Usage(usage_of(
                    &usage,
                    raw_finish_reason
                        .as_deref()
                        .or_else(|| stop_reason.map(CanonicalStopReason::anthropic_str)),
                ))),
                Ok(CanonicalEvent::Error(detail)) => {
                    tracing::warn!(detail = %detail, "dynwork chats reply stream reported an error");
                    Some(ModelChunk::Error(ReplyError::new(
                        "provider.error",
                        "the model provider returned an error",
                    )))
                },
                Ok(_) => None,
                Err(error) => Some(ModelChunk::Error(failure(&error))),
            })
        })))
    }
}

fn usage_of(usage: &CanonicalUsage, raw_finish: Option<&str>) -> Usage {
    Usage {
        input: u64::from(usage.input_tokens),
        output: u64::from(usage.output_tokens),
        reasoning: u64::from(usage.reasoning_tokens),
        cache_read: u64::from(usage.cache_read_tokens),
        cache_write: u64::from(usage.cache_creation_tokens),
        raw_finish: raw_finish.map(raw_finish_of),
    }
}

pub fn failure(error: &AiError) -> ReplyError {
    tracing::warn!(error = %error, "dynwork chats reply failed in the model");
    match error {
        AiError::Provider(_) => {
            ReplyError::new("provider.error", "the model provider returned an error")
        },
        AiError::Routing(_) => ReplyError::new(
            "provider.unavailable",
            "no model provider can serve this model now",
        ),
        AiError::Request(_) => ReplyError::new(
            "provider.invalid-request",
            "the model provider cannot take this request",
        ),
        _ => ReplyError::new("internal", "the reply failed on the server"),
    }
}

pub fn finish_of(raw: &str) -> &'static str {
    match raw {
        "end_turn" | "stop" | "stop_sequence" | "STOP" => "stop",
        "max_tokens" | "length" | "MAX_TOKENS" => "length",
        "tool_use" | "tool_calls" | "tool-calls" => "tool-calls",
        "content_filter" | "refusal" | "SAFETY" => "content-filter",
        _ => "unknown",
    }
}

pub fn raw_finish_of(raw: &str) -> &'static str {
    const KNOWN: [&str; 13] = [
        "end_turn",
        "stop",
        "stop_sequence",
        "STOP",
        "max_tokens",
        "length",
        "MAX_TOKENS",
        "tool_use",
        "tool_calls",
        "tool-calls",
        "content_filter",
        "refusal",
        "SAFETY",
    ];
    KNOWN
        .iter()
        .find(|known| **known == raw)
        .copied()
        .unwrap_or("other")
}
