//! The tools an agent run may use: its declared `tools:` plus the Skill tool,
//! and every Claude Code built-in it did not declare denied outright.

const SKILL_TOOL: &str = "Skill";

pub(crate) fn allowed_tools(agent_tools: &[String]) -> Vec<String> {
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

pub(crate) fn disallowed_tools(agent_tools: &[String]) -> Vec<String> {
    if agent_tools.is_empty() {
        return Vec::new();
    }
    BUILTIN_TOOLS
        .iter()
        .filter(|tool| !agent_tools.iter().any(|t| t == *tool))
        .map(|tool| (*tool).to_owned())
        .collect()
}
