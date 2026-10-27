//! The `documents` MCP server: core's shared document store tools and its
//! `ui://documents/document-preview` resource, composed into an rmcp server.
//!
//! The tool surface, its schemas and every handler are core's
//! [`DocumentsMcpTools`](systemprompt::runtime::documents::mcp::DocumentsMcpTools);
//! this crate authenticates each call, records the access and hands the
//! authenticated request context to it, so every tool acts as the caller.

pub mod server;

pub use server::DocumentsServer;

pub const SERVER_NAME: &str = "documents";

#[must_use]
pub const fn tool_names() -> &'static [&'static str] {
    systemprompt::runtime::documents::mcp::DocumentsMcpTools::tool_names()
}
