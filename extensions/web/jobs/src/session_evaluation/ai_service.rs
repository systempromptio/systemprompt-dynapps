//! Builds the gateway client a job uses for judge calls.
//!
//! `AppContext` carries no `AiService`; the CLI's `agents run` assembles one
//! from the services catalog and the runtime's repositories, and this is the
//! same assembly. One service per tick, dropped after its audit tasks drain.

use std::sync::Arc;

use systemprompt::ai::{AiService, AiServiceProviders};
use systemprompt::database::DbPool;
use systemprompt::mcp::McpToolProvider;
use systemprompt::models::ServicesConfig;
use systemprompt::system::AppContext;
use systemprompt::users::{SessionRepository, UsersAiSessionProvider};

use crate::JobError;

pub(super) fn build_ai_service(
    db: &DbPool,
    app: &AppContext,
    services: &ServicesConfig,
) -> Result<Arc<AiService>, JobError> {
    let tool_provider = Arc::new(McpToolProvider::new(
        Arc::clone(db),
        app.mcp_registry().clone(),
        &services.ai.mcp.resilience,
    ));
    let session_provider = Arc::new(UsersAiSessionProvider::from_repository(
        SessionRepository::new(db).map_err(|e| JobError::Other(e.to_string()))?,
    ));
    let service = AiService::new(
        db,
        &services.providers,
        &services.ai,
        AiServiceProviders {
            tools: tool_provider,
            sessions: session_provider,
        },
        app.ai_repositories(),
    )?
    .with_context_materializer(app.context_materializer());
    Ok(Arc::new(service))
}
