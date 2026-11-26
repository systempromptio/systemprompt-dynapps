//! The Brain's settings, from `<services>/config/dynbrain.yaml`.
//!
//! The file has one block, `dynbrain:`. `mode` is `native` (the default: the
//! Brain answers from this instance's own store) or `proxy` (every operation
//! is a call to a tool of another MCP server, the source, made as the person
//! who asks). In proxy mode, `proxy.server` names an MCP server declared in
//! `services/mcp/`, and `proxy.operations` says, per operation, which tool of
//! the source answers it: `tool`, `args` (our argument name to the source's,
//! for the ones that differ; the others go under their own name) and `fixed`
//! (constants added to every call). An operation with no entry answers
//! `unavailable`. `proxy.file_urls` are the URL paths the source names its
//! files with, `{id}` standing for the file's id; `get` turns them into
//! `asset://<id>`.
//!
//! Nothing here names a particular source: what a source needs lives in the
//! instance's own file. Core validates the file at startup through
//! [`BrainConfig::from_json`]; the server a proxy names is checked against
//! the declared MCP servers when the Brain is composed, and an undeclared one
//! stops it from being mounted.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use serde::Deserialize;
// JSON: core hands an extension its config block as a JSON document, and the
// constants of a mapping are free JSON values sent as tool arguments.
use serde_json::{Map, Value};
use systemprompt::manifest::ServicesConfig;
use systemprompt::mcp::McpDomainError;

use crate::brain::operation::Operation;

pub const CONFIG_PREFIX: &str = "dynbrain";

// Why: core calls validate_config again at mount time with {"paths": ...}; that
// key is set aside before the strict parse.
const MOUNT_KEY: &str = "paths";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Native,
    Proxy,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrainConfig {
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxyConfig {
    pub server: String,
    #[serde(default)]
    pub operations: BTreeMap<Operation, ToolMapping>,
    #[serde(default)]
    pub file_urls: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolMapping {
    pub tool: String,
    #[serde(default)]
    pub args: BTreeMap<String, String>,
    #[serde(default)]
    pub fixed: Map<String, Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    #[serde(default)]
    dynbrain: BrainConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("the Brain config {path} could not be read")]
    Read {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("the Brain config is not valid YAML of the expected shape")]
    Yaml(#[from] serde_yaml::Error),
    #[error("the Brain config is not a valid block")]
    Json(#[from] serde_json::Error),
    #[error("mode `proxy` needs a `proxy` block naming its source server")]
    NoSource,
    #[error("the source tool of the operation `{0}` is empty")]
    EmptyTool(&'static str),
    #[error("the source server `{0}` is not a declared, enabled MCP server")]
    UndeclaredServer(String),
    #[error("the source server could not be resolved")]
    Registry(#[source] Box<McpDomainError>),
    #[error("the database is not Postgres")]
    NotPostgres,
}

impl BrainConfig {
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let parsed: Option<ConfigFile> = serde_yaml::from_str(text)?;
        parsed.unwrap_or_default().dynbrain.checked()
    }

    // JSON: the block core read from the file.
    pub fn from_json(block: &Value) -> Result<Self, ConfigError> {
        let mut own = block.clone();
        if let Value::Object(keys) = &mut own {
            keys.remove(MOUNT_KEY);
        }
        if own.is_null() || own.as_object().is_some_and(Map::is_empty) {
            return Ok(Self::default());
        }
        serde_json::from_value::<ConfigFile>(own)?
            .dynbrain
            .checked()
    }

    pub fn load(services_root: &Path) -> Result<Self, ConfigError> {
        let path = services_root
            .join("config")
            .join(format!("{CONFIG_PREFIX}.yaml"));
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse(&text),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ConfigError::Read {
                path: path.display().to_string(),
                source,
            }),
        }
    }

    fn checked(self) -> Result<Self, ConfigError> {
        if self.mode == Mode::Native {
            return Ok(self);
        }
        let proxy = self.proxy.as_ref().ok_or(ConfigError::NoSource)?;
        if proxy.server.trim().is_empty() {
            return Err(ConfigError::NoSource);
        }
        if let Some((operation, _)) = proxy
            .operations
            .iter()
            .find(|(_, mapping)| mapping.tool.trim().is_empty())
        {
            return Err(ConfigError::EmptyTool(operation.name()));
        }
        Ok(self)
    }
}

impl ToolMapping {
    // JSON: the arguments of a tool call, ours in and the source's out.
    pub fn translate(&self, args: Value) -> Value {
        let given = match args {
            Value::Object(given) => given,
            _ => Map::new(),
        };
        let mut out: Map<String, Value> = given
            .into_iter()
            .filter(|(_, value)| !value.is_null())
            .map(|(name, value)| (self.args.get(&name).cloned().unwrap_or(name), value))
            .collect();
        out.extend(self.fixed.clone());
        Value::Object(out)
    }
}

pub fn check_server(services: &ServicesConfig, name: &str) -> Result<(), ConfigError> {
    services
        .mcp_servers
        .get(name)
        .filter(|deployment| deployment.enabled)
        .map(|_| ())
        .ok_or_else(|| ConfigError::UndeclaredServer(name.to_owned()))
}
