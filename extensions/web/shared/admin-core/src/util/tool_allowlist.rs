//! One reading of an agent's declared `tools:` list, shared by the run's
//! governance hook (which refuses an undeclared MCP tool) and the farm eval
//! (which fails a run that called one).
//!
//! An entry is an exact tool name, a `prefix*` wildcard, a bare
//! `mcp__<server>` (every tool of that server), or a Claude Code permission
//! rule such as `Bash(git:*)`, whose tool part is what is matched. `Skill` is
//! always permitted: an agent's skills reach it only through that tool.

pub const SKILL_TOOL: &str = "Skill";
pub const MCP_TOOL_PREFIX: &str = "mcp__";
pub const FARM_TOOL_ALLOWLIST_POLICY: &str = "farm_tool_allowlist";

#[must_use]
pub fn tool_permitted(tool: &str, allowlist: &[String]) -> bool {
    tool == SKILL_TOOL || allowlist.iter().any(|entry| entry_matches(entry, tool))
}

fn entry_matches(entry: &str, tool: &str) -> bool {
    let base = entry.split('(').next().unwrap_or(entry).trim();
    if let Some(prefix) = base.strip_suffix('*') {
        return tool.starts_with(prefix);
    }
    tool == base
        || (base.starts_with(MCP_TOOL_PREFIX)
            && !base[MCP_TOOL_PREFIX.len()..].contains("__")
            && tool
                .strip_prefix(base)
                .is_some_and(|rest| rest.starts_with("__")))
}

#[must_use]
pub fn undeclared_mcp_tool(tool: &str, allowlist: &[String]) -> bool {
    !allowlist.is_empty() && tool.starts_with(MCP_TOOL_PREFIX) && !tool_permitted(tool, allowlist)
}
