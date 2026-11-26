//! The handle the admin's Brain section reads the Brain through.
//!
//! The extension publishes it when it composes the Brain, so the admin works
//! in the same process on the same Brain, with no HTTP in between. A native
//! instance also exposes its repository and its retriever, which the admin
//! reads for the technical pages; a proxy instance has only the Brain, whose
//! operations go to the source.

use std::sync::{Arc, OnceLock};

use systemprompt::identifiers::{
    Actor, AgentName, ContextId, JwtToken, SessionId, TraceId, UserId,
};
use systemprompt::models::execution::context::RequestContext;

use crate::brain::Brain;
use crate::brain::native::NativeBrain;
use crate::config::Mode;
use crate::repository::BrainRepository;
use crate::search::Retriever;

/// What only a native instance has: its index.
#[derive(Debug, Clone)]
pub struct Native {
    pub brain: Arc<NativeBrain>,
    pub repo: BrainRepository,
    pub retriever: Retriever,
}

/// The instance's Brain, with its index when the instance is native.
#[derive(Debug, Clone)]
pub struct Console {
    pub mode: Mode,
    pub brain: Arc<dyn Brain>,
    pub native: Option<Native>,
}

static LIVE: OnceLock<Console> = OnceLock::new();

pub fn publish(console: Console) {
    LIVE.get_or_init(|| console);
}

pub fn current() -> Option<Console> {
    LIVE.get().cloned()
}

// Why: `token` is the admin's own session token. A proxy instance's source
// answers as a person, and core asks for that person's identity with it, so
// without it the source refuses.
pub fn admin_context(admin: &UserId, token: Option<&JwtToken>) -> RequestContext {
    let session = SessionId::generate();
    let context = RequestContext::new(
        session.clone(),
        TraceId::generate(),
        ContextId::derived_from_session(&session),
        AgentName::new("brain-admin"),
        Actor::job(admin.clone(), "brain-admin"),
    );
    match token {
        Some(token) => context.with_auth_token(token.clone()),
        None => context,
    }
}
