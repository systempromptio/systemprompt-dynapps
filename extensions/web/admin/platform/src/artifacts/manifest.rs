//! A live page's manifest: the capabilities it declares, in the shape of
//! Anthropic's artifact runtime contract (`CONTRACT_VERSION`).
//!
//! `{mcp: {servers: [{server, tools}]}, artifact: {publish?}, user:
//! {scopes?}}`. Each server lists the tools the page calls on it; `artifact`
//! lets the page republish itself unless `publish` is `false`; `user` lets it
//! know who is viewing. Capabilities this instance does not offer yet (`db`,
//! `sample`, ...) are refused rather than ignored, so a page never counts on
//! one that will not answer. `{}` declares nothing.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
// JSON: the manifest arrives as a tool argument and is stored as JSONB.
use serde_json::Value;

pub const CONTRACT_VERSION: &str = "0.2.99";

const MAX_NAME_BYTES: usize = 128;
const MAX_SERVERS: usize = 16;
const MAX_TOOLS: usize = 64;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpDeclaration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ArtifactDeclaration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<UserDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpDeclaration {
    pub servers: Vec<ServerDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerDeclaration {
    pub server: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDeclaration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserDeclaration {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<UserScope>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserScope {
    Profile,
    Email,
}

/// Why a declaration was not recorded, in words the agent can act on.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct DeclarationRefused(pub String);

impl Capabilities {
    // JSON: the `capabilities` argument as the agent sent it.
    pub fn parse(value: &Value) -> Result<Self, DeclarationRefused> {
        let capabilities: Self = serde_json::from_value(value.clone()).map_err(|error| {
            DeclarationRefused(format!(
                "`capabilities` is {{mcp: {{servers: [{{server, tools}}]}}, artifact: {{}}, user: {{}}}}; \
                 this instance offers no other capability: {error}"
            ))
        })?;
        capabilities.check_shape()?;
        Ok(capabilities)
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.mcp.is_none() && self.artifact.is_none() && self.user.is_none()
    }

    #[must_use]
    pub fn servers(&self) -> &[ServerDeclaration] {
        self.mcp.as_ref().map_or(&[], |mcp| mcp.servers.as_slice())
    }

    #[must_use]
    pub fn server(&self, server: &str) -> Option<&ServerDeclaration> {
        self.servers()
            .iter()
            .find(|declared| declared.server == server)
    }

    #[must_use]
    pub fn declares(&self, server: &str, tool: &str) -> bool {
        self.server(server)
            .is_some_and(|declared| declared.tools.iter().any(|name| name == tool))
    }

    #[must_use]
    pub fn publishes(&self) -> bool {
        self.artifact
            .as_ref()
            .is_some_and(|artifact| artifact.publish != Some(false))
    }

    fn check_shape(&self) -> Result<(), DeclarationRefused> {
        let refuse = |why: String| Err(DeclarationRefused(why));
        let Some(mcp) = &self.mcp else {
            return Ok(());
        };
        if mcp.servers.is_empty() || mcp.servers.len() > MAX_SERVERS {
            return refuse(format!(
                "`mcp.servers` lists 1 to {MAX_SERVERS} servers; leave `mcp` out to declare none"
            ));
        }
        let mut seen = BTreeSet::new();
        for declared in &mcp.servers {
            if !is_name(&declared.server) || !seen.insert(declared.server.as_str()) {
                return refuse(format!(
                    "`{}` is not a server name, or is listed twice",
                    declared.server
                ));
            }
            if declared.tools.is_empty() || declared.tools.len() > MAX_TOOLS {
                return refuse(format!(
                    "`{}` lists 1 to {MAX_TOOLS} tools: the ones the page calls",
                    declared.server
                ));
            }
            let mut tools = BTreeSet::new();
            if let Some(bad) = declared
                .tools
                .iter()
                .find(|tool| !is_name(tool) || !tools.insert(tool.as_str()))
            {
                return refuse(format!(
                    "`{bad}` on `{}` is not a tool name, or is listed twice",
                    declared.server
                ));
            }
        }
        Ok(())
    }
}

// Why: MCP's tool-name alphabet also covers every server key this instance
// declares, and it bounds what a manifest and an audit row can hold.
pub(crate) fn is_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME_BYTES
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}
