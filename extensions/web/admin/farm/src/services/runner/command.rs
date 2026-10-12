//! The `claude -p` invocation for one run, with a cleared environment so the
//! child sees only the run's home, the gateway, and its own credentials.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

const INHERITED_ENV: &[&str] = &["PATH", "LANG"];

#[derive(Debug, Clone)]
pub struct CommandSpec<'a> {
    pub claude_bin: &'a Path,
    pub prompt: &'a str,
    pub agent_id: &'a str,
    pub agents_json: &'a Path,
    pub settings_json: &'a Path,
    pub plugin_dirs: &'a [PathBuf],
    pub permission_mode: &'a str,
    pub allowed_tools: &'a [String],
    pub max_turns: i32,
    pub max_budget_usd: f64,
    pub model: Option<&'a str>,
    pub home: &'a Path,
    pub workspace: &'a Path,
    pub env: ChildEnv<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct ChildEnv<'a> {
    pub internal_base: &'a str,
    pub external_base: &'a str,
    pub pat: &'a str,
    pub gateway_session_id: &'a str,
    pub hook_token: Option<&'a str>,
    pub run_id: &'a str,
}

#[must_use]
pub fn claude_args(spec: &CommandSpec<'_>) -> Vec<String> {
    let mut args = vec![
        "-p".to_owned(),
        spec.prompt.to_owned(),
        "--output-format".to_owned(),
        "stream-json".to_owned(),
        "--verbose".to_owned(),
        "--agents".to_owned(),
        spec.agents_json.to_string_lossy().into_owned(),
        "--agent".to_owned(),
        spec.agent_id.to_owned(),
    ];
    for dir in spec.plugin_dirs {
        args.push("--plugin-dir".to_owned());
        args.push(dir.to_string_lossy().into_owned());
    }
    args.extend([
        "--permission-mode".to_owned(),
        spec.permission_mode.to_owned(),
    ]);
    if !spec.allowed_tools.is_empty() {
        args.push("--allowedTools".to_owned());
        args.push(spec.allowed_tools.join(","));
    }
    args.extend([
        "--max-turns".to_owned(),
        spec.max_turns.max(1).to_string(),
        "--max-budget-usd".to_owned(),
        format!("{:.2}", spec.max_budget_usd.max(0.01)),
        "--no-session-persistence".to_owned(),
        "--setting-sources".to_owned(),
        "user".to_owned(),
        "--settings".to_owned(),
        spec.settings_json.to_string_lossy().into_owned(),
        "--strict-mcp-config".to_owned(),
    ]);
    if let Some(model) = spec.model.filter(|m| !m.trim().is_empty()) {
        args.push("--model".to_owned());
        args.push(model.to_owned());
    }
    args
}

#[must_use]
pub fn child_env(spec: &CommandSpec<'_>) -> Vec<(String, String)> {
    let env = &spec.env;
    let base = env.internal_base.trim_end_matches('/');
    let mut vars: Vec<(String, String)> = INHERITED_ENV
        .iter()
        .filter_map(|key| std::env::var(key).ok().map(|v| ((*key).to_owned(), v)))
        .collect();
    let home = spec.home.to_string_lossy().into_owned();
    let pairs = [
        ("HOME", home.clone()),
        ("CLAUDE_CONFIG_DIR", format!("{home}/.claude")),
        ("TERM", "dumb".to_owned()),
        ("ANTHROPIC_BASE_URL", base.to_owned()),
        ("ANTHROPIC_AUTH_TOKEN", env.pat.to_owned()),
        (
            "ANTHROPIC_CUSTOM_HEADERS",
            format!("x-session-id: {}", env.gateway_session_id),
        ),
        ("SYSTEMPROMPT_URL", env.external_base.to_owned()),
        ("SYSTEMPROMPT_API_URL", base.to_owned()),
        ("SYSTEMPROMPT_FARM_RUN_ID", env.run_id.to_owned()),
        ("DISABLE_AUTOUPDATER", "1".to_owned()),
        ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1".to_owned()),
    ];
    vars.extend(pairs.into_iter().map(|(k, v)| (k.to_owned(), v)));
    if let Some(token) = env.hook_token {
        vars.push(("SYSTEMPROMPT_TOKEN".to_owned(), token.to_owned()));
    }
    vars
}

#[must_use]
pub fn build_claude_command(spec: &CommandSpec<'_>) -> Command {
    let mut command = Command::new(spec.claude_bin);
    command
        .args(claude_args(spec))
        .env_clear()
        .envs(child_env(spec))
        .current_dir(spec.workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);
    command
}
