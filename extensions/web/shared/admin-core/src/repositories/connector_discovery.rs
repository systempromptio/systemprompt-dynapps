//! `services/web/config/connectors.yaml` — per-connector OAuth discovery mode.
//!
//! Core owns the `connector:` block of an MCP deployment and rejects unknown
//! keys there, so the one setting the generic connector reads beyond it lives
//! in this extension-owned file, keyed by MCP server id. Every server is
//! `strict` unless it is listed here as `lenient`. The file is read once at
//! the admin composition root, like the security policy: a missing file means
//! every server is strict, and a file that does not parse (an unknown mode or
//! key included) is logged and served as all-strict, the safe direction.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const CONNECTORS_FILE: &str = "web/config/connectors.yaml";

/// How tolerant the generic connector is of a resource's discovery documents.
///
/// `strict` follows RFC 9728 and RFC 8414 to the letter. `lenient` accepts
/// two shapes some MCP servers publish and Claude Code's MCP client tolerates:
/// a protected resource that names the bare origin of its MCP endpoint, and
/// an `authorization_servers` entry that is the authorization server metadata
/// document URL rather than its issuer.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMode {
    #[default]
    Strict,
    Lenient,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectorEntry {
    #[serde(default)]
    discovery: DiscoveryMode,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectorsDoc {
    #[serde(default)]
    connectors: BTreeMap<String, ConnectorEntry>,
}

/// The discovery mode of every listed connector; unlisted ones are strict.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectorDiscoveryPolicy {
    modes: BTreeMap<String, DiscoveryMode>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectorDiscoveryError {
    #[error("{path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("connector discovery policy does not parse: {0}")]
    Parse(#[source] serde_yaml::Error),
}

impl ConnectorDiscoveryPolicy {
    #[must_use]
    pub fn load_discovery_policy_or_strict(services_dir: &Path) -> Self {
        match Self::load_discovery_policy(services_dir) {
            Ok(policy) => policy,
            Err(error) => {
                tracing::error!(error = %error, file = CONNECTORS_FILE, "connector discovery policy unreadable; every connector is strict");
                Self::default()
            },
        }
    }

    pub fn load_discovery_policy(services_dir: &Path) -> Result<Self, ConnectorDiscoveryError> {
        let path = services_dir.join(CONNECTORS_FILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => return Err(ConnectorDiscoveryError::Read { path, source }),
        };
        Self::parse_discovery_policy(&text)
    }

    pub fn parse_discovery_policy(text: &str) -> Result<Self, ConnectorDiscoveryError> {
        let doc: Option<ConnectorsDoc> =
            serde_yaml::from_str(text).map_err(ConnectorDiscoveryError::Parse)?;
        let modes = doc
            .unwrap_or_default()
            .connectors
            .into_iter()
            .map(|(server, entry)| (server, entry.discovery))
            .collect();
        Ok(Self { modes })
    }

    #[must_use]
    pub fn mode(&self, server: &str) -> DiscoveryMode {
        self.modes.get(server).copied().unwrap_or_default()
    }
}
