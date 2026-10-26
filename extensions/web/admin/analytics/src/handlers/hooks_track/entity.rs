//! Infers which skill, agent, or MCP server a hook event belongs to.

use systemprompt_web_admin_core::types::webhook::HookEventPayload;

pub(super) fn detect_entity(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    payload: &HookEventPayload,
) -> Option<(&'static str, String)> {
    let tool_name = payload.tool_name();

    if tool_name == Some("Skill") {
        return detect_skill_entity(payload, "skill");
    }
    // Why: OpenCode's built-in skill tool is lower-case and keys the skill by
    // `name`; the bridge plugin adds the `plugin:skill` reference separately.
    if tool_name == Some("skill") {
        return detect_skill_entity(payload, "name");
    }

    if let Some(tn) = tool_name
        && let Some(server) = detect_mcp_server(services, tn)
    {
        return Some(("mcp_tool", server));
    }

    if tool_name == Some("Agent") {
        return detect_agent_entity(payload);
    }

    if matches!(payload.event_name(), "SubagentStart" | "SubagentStop") {
        let agent_type = payload
            .common
            .agent_type
            .as_deref()
            .unwrap_or("subagent")
            .to_owned();
        return Some(("agent", agent_type));
    }

    None
}

fn detect_skill_entity(payload: &HookEventPayload, key: &str) -> Option<(&'static str, String)> {
    let input = payload.tool_input()?;
    let skill = input.get(key)?.as_str()?;
    (!skill.is_empty()).then(|| ("skill", skill.to_owned()))
}

fn detect_mcp_server(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    tool_name: &str,
) -> Option<String> {
    systemprompt_web_admin_core::util::mcp_tool_name::parse_mcp_tool_name(
        &services.config(),
        tool_name,
    )
    .map(|name| name.server)
}

fn detect_agent_entity(payload: &HookEventPayload) -> Option<(&'static str, String)> {
    let input = payload.tool_input()?;
    let name = input
        .get("subagent_type")
        .and_then(|v| v.as_str())
        .or_else(|| input.get("description").and_then(|v| v.as_str()))
        .unwrap_or("subagent");
    Some(("agent", name.to_owned()))
}
