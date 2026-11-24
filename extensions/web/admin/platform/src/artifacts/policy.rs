//! The operator's live-pages policy: the MCP servers, and on each the tools,
//! a Drive page may ask for.
//!
//! Read once from `<services>/documents/live-pages.yaml` when the broker and
//! the `documents` server are built; Dynapps ships its defaults there. A
//! server the file does not name is closed to every page, and so is every
//! server when the file is missing. A tool outside the list is answered
//! `blocked_by_policy` at call time and refused at declaration time.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;

const POLICY_DIR: &str = "documents";
const POLICY_FILE: &str = "live-pages.yaml";

#[derive(Debug, Clone, Default)]
pub struct LivePagesPolicy {
    servers: BTreeMap<String, Allowed>,
}

#[derive(Debug, Clone)]
enum Allowed {
    Every,
    Listed(BTreeSet<String>),
}

#[derive(Debug, thiserror::Error)]
pub enum LivePagesPolicyError {
    #[error("{0} could not be read: {1}")]
    Read(String, std::io::Error),
    #[error("{0} does not parse: {1}")]
    Parse(String, serde_yaml::Error),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyFile {
    live_pages: Section,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Section {
    #[serde(default)]
    servers: BTreeMap<String, ServerRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerRule {
    tools: ToolRule,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ToolRule {
    Every(EveryTool),
    Listed(Vec<String>),
}

#[derive(Debug, Deserialize)]
enum EveryTool {
    #[serde(rename = "all")]
    All,
}

impl LivePagesPolicy {
    pub fn load(services_root: &Path) -> Result<Self, LivePagesPolicyError> {
        let path = services_root.join(POLICY_DIR).join(POLICY_FILE);
        let shown = path.display().to_string();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!(path = %shown, "no live-pages policy; pages may ask for no server");
                return Ok(Self::default());
            },
            Err(error) => return Err(LivePagesPolicyError::Read(shown, error)),
        };
        let file: PolicyFile = serde_yaml::from_str(&text)
            .map_err(|error| LivePagesPolicyError::Parse(shown, error))?;
        let servers = file
            .live_pages
            .servers
            .into_iter()
            .map(|(server, rule)| {
                let allowed = match rule.tools {
                    ToolRule::Every(EveryTool::All) => Allowed::Every,
                    ToolRule::Listed(tools) => Allowed::Listed(tools.into_iter().collect()),
                };
                (server, allowed)
            })
            .collect();
        Ok(Self { servers })
    }

    #[must_use]
    pub fn allows_server(&self, server: &str) -> bool {
        self.servers.contains_key(server)
    }

    #[must_use]
    pub fn allows(&self, server: &str, tool: &str) -> bool {
        match self.servers.get(server) {
            Some(Allowed::Every) => true,
            Some(Allowed::Listed(tools)) => tools.contains(tool),
            None => false,
        }
    }
}
