//! The tool surface of the `microsoft-365` server: each tool's name, title
//! and behaviour hints, which follow the Belgian server's
//! `tools/_annotations.py` so clients see the same `d365_*` contract, and
//! the dispatch from a tool name to its handler.

mod handlers;
pub mod mail;
pub mod reply;

use rmcp::model::{Tool, ToolAnnotations};
use systemprompt::identifiers::McpServerId;

pub use handlers::{ToolCall, dispatch};

use crate::SERVER_NAME;

pub const INSTRUCTIONS: &str = "Microsoft 365 through Microsoft Graph, acting as the signed-in \
    user on their own tenant. Connect Microsoft 365 on the Connectors page first; every tool \
    then reads or changes only what that user may reach in Graph. List tools page with `top` \
    and `skip`: when a reply says `has_more`, call again with `skip` increased by `top`.";

/// The four behaviour classes of `_annotations.py`: `Read` for list, get,
/// search and downloads; `Write` for creates (twice creates twice);
/// `IdempotentWrite` for updates and responses; `Destructive` for deletes
/// and sends. Every tool is open-world (it reaches Microsoft's service).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Read,
    Write,
    IdempotentWrite,
    Destructive,
}

impl ToolKind {
    pub fn annotations(self, title: &str) -> ToolAnnotations {
        let (read_only, destructive, idempotent) = match self {
            Self::Read => (true, false, true),
            Self::Write => (false, false, false),
            Self::IdempotentWrite => (false, false, true),
            Self::Destructive => (false, true, false),
        };
        ToolAnnotations::with_title(title)
            .read_only(read_only)
            .destructive(destructive)
            .idempotent(idempotent)
            .open_world(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: &'static str,
    pub title: &'static str,
    pub kind: ToolKind,
}

pub const TOOLS: &[ToolSpec] = &[ToolSpec {
    name: "d365_list_mail",
    title: "List emails",
    kind: ToolKind::Read,
}];

pub fn spec(name: &str) -> Option<&'static ToolSpec> {
    TOOLS.iter().find(|spec| spec.name == name)
}

pub fn tool_names() -> Vec<&'static str> {
    TOOLS.iter().map(|spec| spec.name).collect()
}

pub fn list_tools() -> Vec<Tool> {
    handlers::definitions(&McpServerId::new(SERVER_NAME))
        .into_iter()
        .map(|mut tool| {
            tool.annotations = spec(&tool.name).map(|spec| spec.kind.annotations(spec.title));
            tool
        })
        .collect()
}
