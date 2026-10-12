//! The run-local plugin that reports a run to the governance spine: an
//! http `PreToolUse` hook to `/hooks/govern` and async track hooks for every
//! lifecycle event, authorised with the run's hook token.
//!
//! Same wire shape as core's bridge `hooks_schema.rs`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use systemprompt::identifiers::PluginId;

use serde::Serialize;

use crate::error::{FarmError, FarmResult};

pub const GOVERNANCE_PLUGIN_NAME: &str = "agent-farm-governance";

const HOOK_TIMEOUT_SECS: u32 = 10;

const TRACK_EVENTS: &[&str] = &[
    "PostToolUse",
    "PostToolUseFailure",
    "SessionStart",
    "SessionEnd",
    "UserPromptSubmit",
    "Stop",
    "SubagentStart",
    "SubagentStop",
    "Notification",
    "TaskCompleted",
    "TeammateIdle",
    "PermissionRequest",
];

#[derive(Debug, Serialize)]
struct HooksFile {
    hooks: BTreeMap<String, Vec<HookMatcher>>,
}

#[derive(Debug, Serialize)]
struct HookMatcher {
    matcher: &'static str,
    hooks: Vec<HttpHook>,
}

#[derive(Debug, Serialize)]
struct HttpHook {
    #[serde(rename = "type")]
    kind: &'static str,
    url: String,
    headers: BTreeMap<String, String>,
    #[serde(rename = "allowedEnvVars")]
    allowed_env_vars: Vec<String>,
    timeout: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
}

#[derive(Debug, Serialize)]
struct Manifest {
    name: &'static str,
    description: &'static str,
    version: &'static str,
}

fn hook(url: String, token: &str, event: Option<&str>) -> HookMatcher {
    let mut headers = BTreeMap::new();
    headers.insert("Authorization".to_owned(), format!("Bearer {token}"));
    HookMatcher {
        matcher: "*",
        hooks: vec![HttpHook {
            kind: "http",
            url,
            headers,
            allowed_env_vars: Vec::new(),
            timeout: HOOK_TIMEOUT_SECS,
            r#async: event.map(|_| true),
            event: event.map(str::to_owned),
        }],
    }
}

pub fn write_governance_plugin(
    run_dir: &Path,
    internal_base: &str,
    plugin_id: &PluginId,
    hook_token: &str,
) -> FarmResult<PathBuf> {
    let base = internal_base.trim_end_matches('/');
    let plugin_id = plugin_id.as_str();
    let govern = format!("{base}/api/public/hooks/govern?plugin_id={plugin_id}");
    let track = format!("{base}/api/public/hooks/track?plugin_id={plugin_id}");

    let mut hooks = BTreeMap::new();
    hooks.insert(
        "PreToolUse".to_owned(),
        vec![hook(govern, hook_token, None)],
    );
    for event in TRACK_EVENTS {
        hooks.insert(
            (*event).to_owned(),
            vec![hook(track.clone(), hook_token, Some(event))],
        );
    }

    let dir = run_dir.join("governance-plugin");
    std::fs::create_dir_all(dir.join(".claude-plugin"))?;
    std::fs::create_dir_all(dir.join("hooks"))?;
    let manifest = Manifest {
        name: GOVERNANCE_PLUGIN_NAME,
        description: "Reports this agent farm run to the governance spine.",
        version: "1.0.0",
    };
    std::fs::write(dir.join(".claude-plugin/plugin.json"), to_json(&manifest)?)?;
    std::fs::write(dir.join("hooks/hooks.json"), to_json(&HooksFile { hooks })?)?;
    Ok(dir)
}

fn to_json<T: Serialize>(value: &T) -> FarmResult<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|source| FarmError::Json {
        at: "governance plugin".to_owned(),
        source,
    })
}
