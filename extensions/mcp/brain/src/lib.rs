//! The `brain` MCP server: the Brain's operations as tools, run as the
//! signed-in person.
//!
//! Every call is authenticated by [`systemprompt_mcp_shared::auth`], then the
//! tool runs the operation of [`dynbrain::brain::Brain`] for the person the
//! request names (see [`tools`]). The Brain is composed from the host the
//! server runs on: its database, its file storage and its secrets, the same as
//! the API composes it, and the query embedder of the search, which the
//! server fills itself from an AI service built on the host's granted
//! secrets, because core hands that service to the API's routes only.
//! [`router`] composes the server over a booted host, for the binary. On a
//! proxy instance the Brain is the source's, as the API composes it too.

pub mod server;
pub mod tools;

use std::sync::Arc;

use anyhow::Context;
use dynbrain::extension::compose;
use dynbrain::ingest::core_ai::{CoreEmbedder, EMBEDDING_MODEL, EMBEDDING_PROVIDER};
use dynbrain::search::query::EmbedderSlot;
use systemprompt::identifiers::McpServerId;
use systemprompt::system::{AgentHost, HostContext};

pub use server::BrainServer;

pub const SERVER_NAME: &str = "brain";

pub fn router(ctx: &HostContext, service_id: &McpServerId) -> anyhow::Result<axum::Router> {
    let embedder = EmbedderSlot::default();
    let brain = compose(ctx, embedder.clone()).context("The Brain's config cannot be used")?;
    match AgentHost::compose(ctx.clone()) {
        Ok(host) => {
            let ai = Arc::clone(host.ai_service());
            embedder.fill(move || {
                CoreEmbedder::new(
                    ai,
                    EMBEDDING_PROVIDER.to_owned(),
                    EMBEDDING_MODEL.to_owned(),
                )
            });
        },
        Err(error) => {
            tracing::warn!(%error, "no AI service; searches match keywords");
        },
    }
    let server = BrainServer::new(
        service_id.clone(),
        Arc::clone(ctx.db_pool()),
        systemprompt_mcp_shared::auth::ServerAccess {
            authz_hook: Arc::clone(ctx.authz_hook()),
            services: ctx.services_state().watch().clone(),
        },
        ctx.artifact_ingest(),
        brain,
    );
    let requires_oauth = ctx
        .services_state()
        .config()
        .mcp_servers
        .get(service_id.as_str())
        .is_some_and(|deployment| deployment.oauth.required);
    Ok(systemprompt::mcp::create_router(
        server,
        Arc::clone(ctx.mcp_session_repository()),
        systemprompt::mcp::McpHttpConfig::default(),
        systemprompt::mcp::middleware::HopVerifier::new(
            ctx.verification_keys().clone(),
            &ctx.config().jwt_issuer,
            service_id.clone(),
        )
        .requiring_hop(requires_oauth),
    ))
}
