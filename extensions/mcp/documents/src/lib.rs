//! The `documents` MCP server: core's shared document store tools and its
//! `ui://documents/document-preview` resource, composed into an rmcp server.
//!
//! The tool surface, its schemas and every handler are core's
//! [`DocumentsMcpTools`]; this crate authenticates each call, records the
//! access and hands the authenticated request context to it, so every tool
//! acts as the caller.
//! A live HTML page's `capabilities` ride on the create, import, edit and
//! update tools and are recorded as the page's manifest (`server`).
//! [`router`] composes the server over a booted host, for the binary and for
//! a test that hosts the same server in-process.

pub mod server;

use std::sync::Arc;

use anyhow::Context;
use systemprompt::identifiers::McpServerId;
use systemprompt::runtime::documents::mcp::DocumentsMcpTools;
use systemprompt::system::HostContext;
use systemprompt_web_admin_platform::artifacts::LivePages;

pub use server::DocumentsServer;

pub const SERVER_NAME: &str = "documents";

#[must_use]
pub const fn tool_names() -> &'static [&'static str] {
    DocumentsMcpTools::tool_names()
}

pub fn router(ctx: &HostContext, service_id: &McpServerId) -> anyhow::Result<axum::Router> {
    let server = DocumentsServer::new(
        Arc::clone(ctx.db_pool()),
        systemprompt_mcp_shared::auth::ServerAccess {
            authz_hook: Arc::clone(ctx.authz_hook()),
            services: ctx.services_state().watch().clone(),
        },
        ctx.artifact_ingest(),
        DocumentsMcpTools::from_host(ctx, service_id.clone())
            .context("Document store unavailable")?,
        LivePages::new(
            &ctx.services_root().path,
            ctx.services_state().watch().clone(),
            &ctx.config().api_internal_url,
        )
        .context("Live pages unavailable")?,
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
