//! The tool surface of the `brain` server: the Brain's operations as MCP
//! tools, without a prefix.
//!
//! The names are the operations' own (`search`, `answer`, `get`...); a client
//! prefixes them with the server's name. Descriptions and input schemas keep
//! the guidance of the `docs_*` tools of the dynbrain documentation server,
//! which agents were tested against, minus everything that belongs to that
//! server's Git sync and its versions. The server has no logic of its own:
//! [`run`] turns a call into the operation of the [`dynbrain::brain::Brain`]
//! trait and back into the structured reply.

mod browse;
pub mod fault;
mod files;
mod fragments;
mod handler;
mod read;
mod run;
mod write;

use std::sync::LazyLock;

use rmcp::model::{Tool, ToolAnnotations};
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::McpToolHandler;

pub use handler::{BrainReply, BrainTool, ToolCall, dispatch};
pub use run::run;

use crate::SERVER_NAME;

// JSON: a tool's `inputSchema` is a JSON Schema document and its arguments and
// reply are the JSON the protocol carries.
pub type Json = serde_json::Value;

pub const INSTRUCTIONS: &str = "The Brain: documentation spaces with trees of articles, in \
    several languages. Every call runs as the connected person and honors their access: \
    results are limited to what they may read or edit, and what they may not read does not \
    exist for them. Reading: `search` for a hybrid semantic and keyword lookup and `answer` \
    for the cited context a reply is written from; `list_spaces` and `outline` to discover \
    and navigate; `get` to read a full article. Writing: `create`, `create_tree`, `update`, \
    `move` and `trash` author articles in Markdown. Search before creating, to avoid \
    duplicates. Files travel out of band: `request_upload` and `asset_upload` for adding \
    one, `asset_link` for a download URL.";

/// The behaviour classes the clients' auto-approval reads: reads, plain
/// writes, writes that can be repeated and what removes something from sight.
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
            .open_world(false)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ToolSpec {
    pub name: &'static str,
    pub title: &'static str,
    pub kind: ToolKind,
    pub description: &'static str,
    pub schema: fn() -> Json,
}

pub static TOOLS: LazyLock<Vec<ToolSpec>> =
    LazyLock::new(|| [read::SPECS, browse::SPECS, write::SPECS, files::SPECS].concat());

pub fn spec(name: &str) -> Option<&'static ToolSpec> {
    TOOLS.iter().find(|spec| spec.name == name)
}

pub fn tool_names() -> Vec<&'static str> {
    TOOLS.iter().map(|spec| spec.name).collect()
}

pub fn list_tools() -> Vec<Tool> {
    let server = McpServerId::new(SERVER_NAME);
    TOOLS
        .iter()
        .map(|spec| {
            let mut tool = BrainTool { spec, brain: None }.tool_definition(&server);
            tool.annotations = Some(spec.kind.annotations(spec.title));
            tool
        })
        .collect()
}
