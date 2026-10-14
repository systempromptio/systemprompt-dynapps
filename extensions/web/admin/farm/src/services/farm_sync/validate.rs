//! Identifier and path rules shared by every farm definition parser, as
//! `FARM.md` states them.
//!
//! A tool is a built-in ([`ALLOWED_TOOLS`]) or an MCP entry:
//! `mcp__<server>`, `mcp__<server>__<tool>` or `mcp__<server>__*`;
//! [`mcp_server_of`] names the server one refers to.

use crate::error::{FarmError, FarmResult};

pub const ALLOWED_TOOLS: [&str; 13] = [
    "Read",
    "Grep",
    "Glob",
    "Write",
    "Edit",
    "MultiEdit",
    "Bash",
    "WebFetch",
    "WebSearch",
    "TodoWrite",
    "NotebookEdit",
    "Task",
    "Skill",
];

pub const MCP_TOOL_PREFIX: &str = "mcp__";

fn mcp_name(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[must_use]
pub fn mcp_server_of(tool: &str) -> Option<&str> {
    let rest = tool.strip_prefix(MCP_TOOL_PREFIX)?;
    let (server, name) = match rest.split_once("__") {
        Some((server, name)) => (server, Some(name)),
        None => (rest, None),
    };
    let name_ok = name.is_none_or(|name| name == "*" || mcp_name(name, 128));
    (mcp_name(server, 64) && name_ok).then_some(server)
}

#[must_use]
pub fn is_allowed_tool(tool: &str) -> bool {
    ALLOWED_TOOLS.contains(&tool) || mcp_server_of(tool).is_some()
}

pub const PERMISSION_MODES: [&str; 4] = ["default", "acceptEdits", "plan", "bypassPermissions"];

fn shaped(value: &str, extra: char) -> bool {
    let mut chars = value.chars();
    let starts_lower = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    (3..=64).contains(&value.len())
        && starts_lower
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == extra)
}

#[must_use]
pub fn is_identifier(value: &str) -> bool {
    shaped(value, '_')
}

#[must_use]
pub fn is_farm_id(value: &str) -> bool {
    shaped(value, '-')
}

#[must_use]
pub fn is_case_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[must_use]
pub fn is_safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\0')
        && !value.contains('\\')
        && value
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

pub fn definition_error(path: &str, message: impl std::fmt::Display) -> FarmError {
    FarmError::Validation(format!("{path}: {message}"))
}

pub fn require(condition: bool, path: &str, message: impl std::fmt::Display) -> FarmResult<()> {
    if condition {
        Ok(())
    } else {
        Err(definition_error(path, message))
    }
}
