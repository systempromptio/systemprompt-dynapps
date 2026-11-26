//! The indexer's models through core's `AiService`.
//!
//! Embeddings go to `text-embedding-3-large` on `OpenAI` at 1536 dimensions,
//! priced from the catalog (`embed` leaves no audit row, so the caller
//! records the cost). The synopsis goes to Claude Haiku as the job's actor,
//! so it is audited like any other request. The provider and the models are
//! the catalog's (`services/ai`); the key of each is a secret of the
//! instance, and an endpoint whose key is missing is withheld by core, which
//! reaches here as [`Failure::NotConfigured`].

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::ai::error::{AiError, ProviderError};
use systemprompt::ai::{AiMessage, AiRequest, AiService, EmbeddingRequest};
use systemprompt::identifiers::{Actor, AgentName, ContextId, ProviderId, SessionId, TraceId};
use systemprompt::models::execution::context::RequestContext;

use super::context::SynopsisPrompt;
use super::provider::{DIMENSIONS, Embedder, Embedding, Failure, Summarizer};

pub const AGENT_NAME: &str = "dynbrain-index";
const SYNOPSIS_MAX_TOKENS: u32 = 120;
const JOB_CONTEXT: u128 = 0x0d79_6272_6169_6e5f_696e_6465_7800_0001;

/// `OpenAI` embeddings through `AiService::embed`.
pub struct CoreEmbedder {
    ai: Arc<AiService>,
    provider: String,
    model: String,
}

/// Claude Haiku synopses through `AiService::generate`, as a job.
pub struct CoreSummarizer {
    ai: Arc<AiService>,
    actor: Actor,
    provider: String,
    model: String,
}

impl fmt::Debug for CoreEmbedder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreEmbedder")
            .field("provider", &self.provider)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for CoreSummarizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreSummarizer")
            .field("provider", &self.provider)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl CoreEmbedder {
    pub const fn new(ai: Arc<AiService>, provider: String, model: String) -> Self {
        Self {
            ai,
            provider,
            model,
        }
    }
}

impl CoreSummarizer {
    pub const fn new(ai: Arc<AiService>, actor: Actor, provider: String, model: String) -> Self {
        Self {
            ai,
            actor,
            provider,
            model,
        }
    }
}

#[async_trait]
impl Embedder for CoreEmbedder {
    fn fingerprint(&self) -> String {
        format!("{}:{}:{DIMENSIONS}", self.provider, self.model)
    }

    async fn embed(&self, texts: &[String]) -> Result<Embedding, Failure> {
        let request = EmbeddingRequest::new(self.model.as_str(), texts.to_vec())
            .with_dimensions(DIMENSIONS as u32)
            .with_provider(ProviderId::new(self.provider.as_str()));
        let done = self
            .ai
            .embed(&request)
            .await
            .map_err(|error| classify(&error))?;
        Ok(Embedding {
            vectors: done.response.vectors,
            input_tokens: done.response.usage.input_tokens,
            cost_microdollars: done.cost_microdollars,
        })
    }
}

#[async_trait]
impl Summarizer for CoreSummarizer {
    async fn summarize(&self, prompt: &SynopsisPrompt) -> Result<String, Failure> {
        let context = RequestContext::new(
            SessionId::generate(),
            TraceId::new(uuid::Uuid::new_v4().to_string()),
            ContextId::from_uuid(uuid::Uuid::from_u128(JOB_CONTEXT)),
            AgentName::new(AGENT_NAME),
            self.actor.clone(),
        );
        let request = AiRequest::builder(
            vec![
                AiMessage::system(prompt.system.as_str()),
                AiMessage::user(prompt.user.as_str()),
            ],
            self.provider.as_str(),
            self.model.as_str(),
            SYNOPSIS_MAX_TOKENS,
            context,
        )
        .build();
        let response = self
            .ai
            .generate(&request)
            .await
            .map_err(|error| classify(&error))?;
        Ok(response.content)
    }
}

fn classify(error: &AiError) -> Failure {
    match error {
        AiError::Provider(ProviderError::HttpStatus { status: 429, .. }) => Failure::RateLimited,
        AiError::Provider(ProviderError::HttpStatus {
            status: 401 | 403, ..
        })
        | AiError::Routing(_) => Failure::NotConfigured(detail(error)),
        AiError::Provider(
            ProviderError::HttpStatus {
                status: 500..=599, ..
            }
            | ProviderError::Timeout { .. }
            | ProviderError::CircuitOpen { .. }
            | ProviderError::DependencyUnavailable { .. }
            | ProviderError::Http(_),
        ) => Failure::Unavailable(detail(error)),
        _ => Failure::Rejected(detail(error)),
    }
}

fn detail(error: &AiError) -> String {
    error.to_string()
}
