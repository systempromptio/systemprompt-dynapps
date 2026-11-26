//! The handle the admin's Brain section reads the Brain through.
//!
//! The extension publishes it when it composes the Brain, so the admin works
//! in the same process on the same Brain, with no HTTP in between. A native
//! instance also exposes its repository and its retriever, which the admin
//! reads for the technical pages; a proxy instance has only the Brain, whose
//! operations go to the source.

use std::sync::{Arc, OnceLock};

use systemprompt::identifiers::{Actor, AgentName, ContextId, SessionId, TraceId, UserId};
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

pub fn admin_context(admin: &UserId) -> Result<RequestContext, String> {
    let session = SessionId::generate();
    let agent = AgentName::try_new("brain-admin").map_err(|error| error.to_string())?;
    Ok(RequestContext::new(
        session.clone(),
        TraceId::generate(),
        ContextId::derived_from_session(&session),
        agent,
        Actor::job(admin.clone(), "brain-admin"),
    ))
}
