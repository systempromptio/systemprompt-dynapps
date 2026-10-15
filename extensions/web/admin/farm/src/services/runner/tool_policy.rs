//! The tools an agent run may use: its declared `tools:` plus the Skill tool,
//! and every Claude Code built-in it did not declare denied outright.
//!
//! An undeclared `mcp__` tool cannot be named here: Claude Code's deny rules
//! have no "every tool of this server but" form and the server's catalogue is
//! not known at launch. `--allowedTools` lists the declared tools exactly, and
//! the run's governance hook refuses the rest under `farm_tool_allowlist`.
//! A farm's `tool_policy.deny` is appended by [`with_farm_denials`].

use systemprompt_web_admin_core::util::tool_allowlist::SKILL_TOOL;

#[must_use]
pub fn allowed_tools(agent_tools: &[String]) -> Vec<String> {
    let mut tools = agent_tools.to_vec();
    if !tools.is_empty() && !tools.iter().any(|t| t == SKILL_TOOL) {
        tools.push(SKILL_TOOL.to_owned());
    }
    tools
}

// Why: Claude Code's --allowedTools only pre-approves tools; under
// acceptEdits a built-in the agent never declared (Bash) still runs, so the
// built-ins outside an agent's `tools:` are denied outright.
const BUILTIN_TOOLS: [&str; 14] = [
    "Bash",
    "BashOutput",
    "Edit",
    "Glob",
    "Grep",
    "KillShell",
    "MultiEdit",
    "NotebookEdit",
    "Read",
    "Task",
    "TodoWrite",
    "WebFetch",
    "WebSearch",
    "Write",
];

#[must_use]
pub fn disallowed_tools(agent_tools: &[String]) -> Vec<String> {
    if agent_tools.is_empty() {
        return Vec::new();
    }
    BUILTIN_TOOLS
        .iter()
        .filter(|tool| !agent_tools.iter().any(|t| t == *tool))
        .map(|tool| (*tool).to_owned())
        .collect()
}

#[must_use]
pub fn with_farm_denials(mut denied: Vec<String>, farm_deny: &[String]) -> Vec<String> {
    for tool in farm_deny {
        if !denied.contains(tool) {
            denied.push(tool.clone());
        }
    }
    denied
}
