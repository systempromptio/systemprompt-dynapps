//! The `microsoft-365` MCP server: Microsoft Graph tools that act as the
//! signed-in user.
//!
//! Every call is authenticated by [`systemprompt_mcp_shared::auth`], then a
//! per-call [`graph::GraphClient`] is built over the caller's own Graph
//! token, which the connector broker hands out for that user only
//! ([`graph::token`]). Tool names and annotations follow the Belgian
//! `dynapps-365` server (`d365_*`) so clients see the same surface.

pub mod graph;
pub mod server;
pub mod tools;

pub use server::{GraphHost, Microsoft365Server};

pub const SERVER_NAME: &str = "microsoft-365";
