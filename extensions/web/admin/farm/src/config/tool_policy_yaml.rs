//! A farm's `tool_policy:` in agent-farms.yaml: which MCP servers its runs
//! may reach, and which tools they are denied outright.
//!
//! `mcp_servers` is an allowlist. Empty means no farm-level restriction; set,
//! an agent whose `tools:` name a server outside it is refused at launch, so
//! the run is never handed an endpoint the farm does not allow. `deny` is
//! appended to the run's `--disallowedTools`: a farm runs unattended, so a
//! write tool the agent's `mcp__<server>` entry would otherwise allow is
//! refused by Claude Code before any call leaves the runner.

use serde::{Deserialize, Serialize};

use crate::services::farm_sync::validate::{is_allowed_tool, mcp_server_of};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarmToolPolicy {
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

impl FarmToolPolicy {
    pub(super) fn validate(&self, farm: &str) -> Result<(), String> {
        if let Some(bad) = self
            .mcp_servers
            .iter()
            .find(|s| mcp_server_of(&format!("mcp__{s}")).is_none())
        {
            return Err(format!(
                "farm {farm:?}: tool_policy.mcp_servers entry {bad:?} is not a server id"
            ));
        }
        if let Some(bad) = self.deny.iter().find(|t| !is_allowed_tool(t)) {
            return Err(format!(
                "farm {farm:?}: tool_policy.deny entry {bad:?} is not a tool"
            ));
        }
        Ok(())
    }

    /// The first server an agent's tools name that the allowlist leaves out.
    #[must_use]
    pub fn disallowed_server<'a>(&self, agent_tools: &'a [String]) -> Option<&'a str> {
        if self.mcp_servers.is_empty() {
            return None;
        }
        agent_tools
            .iter()
            .filter_map(|tool| mcp_server_of(tool))
            .find(|server| !self.mcp_servers.iter().any(|s| s == server))
    }
}
