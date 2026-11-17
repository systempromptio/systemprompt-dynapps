//! The model a reply streams from, behind [`ReplyModel`].
//!
//! Stage 1 answers through core's `AiService::generate_stream`, which yields
//! text and usage only. Stage 2 swaps that call for
//! `AiService::generate_events_stream` (decision D1) inside [`GatewayModel`]
//! and adds chunk kinds here; nothing else in the crate names the AI service.
//! A provider's finish reason maps to `OpenCode`'s `finish`; as `rawFinish`
//! the known ones are kept and anything else becomes `other`, so no provider
//! text reaches the log.

use std::collections::BTreeSet;
use std::fmt;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use systemprompt::ai::error::AiError;
use systemprompt::ai::{AiMessage, AiRequest, AiService};
use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt::models::ai::StreamChunk;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::traits::SecretStore;

use crate::catalog::{self, ModelInfo};
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

/// Which model answers a request, and which models a chat can pick.
///
/// Core hands each request the instance's AI service (wrapped as a
/// [`GatewayModel`] by the handler, `None` when the instance has none);
/// production answers with it, a suite with its fake. `list` is `None` when
/// there is no AI service to list from.
pub trait ModelSource: Send + Sync + fmt::Debug {
    fn pick(&self, gateway: Option<Arc<dyn ReplyModel>>) -> Option<Arc<dyn ReplyModel>>;

    fn list(&self, ai: Option<&AiService>) -> Option<Vec<ModelInfo>>;
}

/// [`ModelSource`] of a real instance: the AI service of the request and its
/// catalog, filtered by the secrets the instance holds (see
/// [`catalog::listed`]).
#[derive(Debug)]
pub struct Gateway {
    secrets: Arc<dyn SecretStore>,
}

impl Gateway {
    pub fn new(secrets: Arc<dyn SecretStore>) -> Self {
        Self { secrets }
    }

    fn secret_names(&self) -> BTreeSet<String> {
        match self.secrets.key_names() {
            Ok(names) => names.into_iter().map(|name| name.to_lowercase()).collect(),
            Err(error) => {
                tracing::error!(error = %error, "dynwork chats could not list the instance secrets; no model is served");
                BTreeSet::new()
            },
        }
    }
}

impl ModelSource for Gateway {
    fn pick(&self, gateway: Option<Arc<dyn ReplyModel>>) -> Option<Arc<dyn ReplyModel>> {
        gateway
    }

    fn list(&self, ai: Option<&AiService>) -> Option<Vec<ModelInfo>> {
        let names = self.secret_names();
        ai.map(|ai| catalog::listed(ai.catalog(), |name| names.contains(&name.to_lowercase())))
    }
}

/// [`ReplyModel`] over core's `AiService`, under the caller's identity so the
/// instance's quotas, policies and audit apply to every reply.
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
        let stream = self
            .ai
            .generate_stream(&ai_request)
            .await
            .map_err(|error| failure(&error))?;
        Ok(Box::pin(stream.map(|item| match item {
            Ok(StreamChunk::Text(text)) => ModelChunk::Text(text),
            Ok(StreamChunk::Usage {
                input_tokens,
                output_tokens,
                cache_read_tokens,
                cache_creation_tokens,
                reasoning_tokens,
                finish_reason,
                ..
            }) => ModelChunk::Usage(Usage {
                input: count(input_tokens),
                output: count(output_tokens),
                reasoning: count(reasoning_tokens),
                cache_read: count(cache_read_tokens),
                cache_write: count(cache_creation_tokens),
                raw_finish: finish_reason.as_deref().map(raw_finish_of),
            }),
            Err(error) => ModelChunk::Error(failure(&error)),
        })))
    }
}

fn count(tokens: Option<u32>) -> u64 {
    tokens.map_or(0, u64::from)
}

fn failure(error: &AiError) -> ReplyError {
    tracing::warn!(error = %error, "dynwork chats reply failed in the model");
    match error {
        AiError::Provider(_) => ReplyError::new("provider.error", error.to_string()),
        AiError::Routing(_) => ReplyError::new("provider.unavailable", error.to_string()),
        AiError::Request(_) => ReplyError::new("provider.invalid-request", error.to_string()),
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
