//! From the farm's rows to core's [`RunRequest`], and the run directory the
//! request points at.
//!
//! [`build_request`] runs at launch: the request is stored verbatim as
//! `agent_runs.request` and the executor claims it blind, so every path in
//! it is fixed then, under the run's [`RunLayout`]. [`materialise`] runs at
//! claim time and fills those paths in: the fixture seeded into the
//! workspace and the agent's plugin tree composed from the kit, on a first
//! attempt only: a resumed attempt keeps the workspace the earlier one
//! edited, which re-seeding the fixture would overwrite.
//!
//! `credential_purpose` (`farm:<run id>`) feeds `IssueRunCredentials.purpose`;
//! the caller is bound to the credential at issue, never sent as a forgeable
//! scope header. `credential_user` names the user a run's credentials are
//! issued to: the agent's own service account
//! (`agent_farm_agents.run_as_user_id`), else the farm's
//! (`agent_farms.run_as_user_id`), else the launching user. The run's farm is
//! bound as its `farm` attribution scope, verified at the gateway against the
//! credential user's active runs.
//!
//! Every MCP server an agent's `mcp__<server>` tools name is handed to the
//! host as this instance's HTTP endpoint for it. The request carries no
//! bearer: it is stored verbatim, so [`with_mcp_bearer`] adds the run's MCP
//! token (whose signed `purpose` is the caller) only at claim time.

use std::path::{Path, PathBuf};
use std::time::Duration;

use std::collections::BTreeMap;

use systemprompt::agent_runs::{HostKind, McpServerSpec, PermissionMode, RunLayout, RunRequest};
use systemprompt::identifiers::{AgentRunId, ScopeDimension, UserId};
use systemprompt::models::attribution::ScopeBinding;
use walkdir::WalkDir;

use super::plugin::materialise_plugin;
use crate::attribution::FARM_DIMENSION;
use crate::error::{FarmError, FarmResult};
use crate::repositories::rows::{AgentRow, RunRow};
use crate::repositories::{farms, fixtures};
use crate::services::farm_sync::render_entry_prompt;
use crate::services::farm_sync::validate::mcp_server_of;
use crate::state::FarmState;
use crate::types::{FarmId, FarmRunId};

const SKILL_TOOL: &str = "Skill";
const PLUGIN_DIR: &str = "plugin";

/// What a launch knows before the request exists. `prompt_template` is the
/// run's own prompt or the agent's entry prompt, not yet rendered.
#[derive(Debug, Clone, Copy)]
pub struct RequestSpec<'a> {
    pub agent: &'a AgentRow,
    pub farm_id: &'a FarmId,
    pub farm_run_id: &'a FarmRunId,
    pub layout: &'a RunLayout,
    pub mcp_base: &'a str,
    pub prompt_template: &'a str,
    pub case_prompt: Option<&'a str>,
    pub model: Option<&'a str>,
    pub permission_mode: &'a str,
    pub max_turns: i32,
    pub max_budget_usd: f64,
    pub timeout_seconds: i32,
}

#[must_use]
pub fn run_layout(state: &FarmState, agent_run_id: &AgentRunId) -> RunLayout {
    RunLayout::new(state.paths.run_dir(agent_run_id.as_str()))
}

#[must_use]
pub fn credential_user(
    agent_run_as: Option<&UserId>,
    farm_run_as: Option<&UserId>,
    run_user: &UserId,
) -> UserId {
    agent_run_as.or(farm_run_as).unwrap_or(run_user).clone()
}

#[must_use]
pub fn credential_purpose(run_id: &FarmRunId) -> String {
    format!("farm:{run_id}")
}

#[must_use]
pub fn mcp_servers(agent_tools: &[String], base: &str) -> BTreeMap<String, McpServerSpec> {
    agent_tools
        .iter()
        .filter_map(|tool| mcp_server_of(tool))
        .map(|server| {
            let spec = McpServerSpec::Http {
                url: format!("{base}/api/v1/mcp/{server}/mcp"),
                headers: BTreeMap::new(),
            };
            (server.to_owned(), spec)
        })
        .collect()
}

#[must_use]
pub fn with_mcp_bearer(request: &RunRequest, token: Option<&str>) -> RunRequest {
    let mut request = request.clone();
    let Some(token) = token else {
        return request;
    };
    for spec in request.mcp_servers.values_mut() {
        if let McpServerSpec::Http { headers, .. } = spec {
            headers.insert("Authorization".to_owned(), format!("Bearer {token}"));
        }
    }
    request
}

pub fn build_request(spec: RequestSpec<'_>) -> FarmResult<RunRequest> {
    let agent = spec.agent;
    let host: HostKind = agent.host.parse().map_err(|source| FarmError::AgentHost {
        agent: agent.id.to_string(),
        source: Box::new(source),
    })?;
    let permission_mode: PermissionMode = spec.permission_mode.parse()?;
    let workspace = spec.layout.workspace.to_string_lossy().into_owned();
    let prompt = render_entry_prompt(spec.prompt_template, &workspace, spec.case_prompt);
    let positive = |name: &str| FarmError::Validation(format!("{name} must be positive"));
    let max_turns = u32::try_from(spec.max_turns)
        .ok()
        .filter(|turns| *turns > 0)
        .ok_or_else(|| positive("max_turns"))?;
    let wall_clock = u64::try_from(spec.timeout_seconds)
        .ok()
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
        .ok_or_else(|| positive("timeout_seconds"))?;
    let farm_scope = ScopeBinding {
        dimension: ScopeDimension::try_new(FARM_DIMENSION).map_err(|e| FarmError::Identifier {
            at: "farm scope dimension",
            source: Box::new(e),
        })?,
        value: spec.farm_id.as_str().to_owned(),
    };
    Ok(RunRequest {
        host,
        prompt,
        system_prompt_append: None,
        persona: (!agent.body.trim().is_empty()).then(|| agent.body.clone()),
        model: spec.model.map(ToOwned::to_owned),
        allowed_tools: allowed_tools(&agent.tools),
        disallowed_tools: Vec::new(),
        permission_mode,
        permission_prompts_none: false,
        workspace: spec.layout.workspace.clone(),
        add_dirs: Vec::new(),
        mcp_servers: mcp_servers(&agent.tools, spec.mcp_base),
        output_schema: None,
        max_turns: Some(max_turns),
        max_budget_usd: Some(spec.max_budget_usd),
        wall_clock,
        resume_session_id: None,
        attempt: 1,
        attribution_scopes: vec![farm_scope],
        plugin_id: Some(agent.plugin_id.clone()),
        plugin_dirs: vec![plugin_root(spec.layout).join(agent.plugin_id.as_str())],
    })
}

fn plugin_root(layout: &RunLayout) -> PathBuf {
    layout.root.join(PLUGIN_DIR)
}

fn allowed_tools(agent_tools: &[String]) -> Vec<String> {
    let mut tools = agent_tools.to_vec();
    if !tools.is_empty() && !tools.iter().any(|t| t == SKILL_TOOL) {
        tools.push(SKILL_TOOL.to_owned());
    }
    tools
}

pub async fn materialise(
    state: &FarmState,
    run: &RunRow,
    agent: &AgentRow,
    layout: &RunLayout,
) -> FarmResult<Vec<String>> {
    layout.create().await?;
    let fixture = match run.fixture.as_deref() {
        Some(name) => Some(resolve_fixture(state, run, name).await?),
        None => None,
    };
    let services_root = state.paths.composed_services_root.clone();
    let plugin_id = agent.plugin_id.clone();
    let workspace = layout.workspace.clone();
    let plugins = plugin_root(layout);
    tokio::task::spawn_blocking(move || -> FarmResult<Vec<String>> {
        if let Some(source) = fixture {
            copy_tree(&source, &workspace)?;
        }
        let plugin = materialise_plugin(&services_root, &plugin_id, &plugins)?;
        Ok(plugin
            .missing_skills
            .iter()
            .map(|skill| format!("skill {skill} is not in the composed services tree"))
            .collect())
    })
    .await
    .map_err(|source| FarmError::Task {
        step: "materialise",
        source,
    })?
}

async fn resolve_fixture(state: &FarmState, run: &RunRow, name: &str) -> FarmResult<PathBuf> {
    if let Some(row) = fixtures::find_fixture(state.pool.as_ref(), &run.farm_id, name).await? {
        let stored = PathBuf::from(&row.stored_path);
        if stored.is_dir() {
            return Ok(stored);
        }
    }
    let commit = match run.source_commit_sha.clone() {
        Some(sha) => sha,
        None => farms::find_farm(state.pool.as_ref(), &run.farm_id)
            .await?
            .and_then(|farm| farm.synced_commit_sha)
            .ok_or_else(|| FarmError::NotFound(format!("farm {} is not synced", run.farm_id)))?,
    };
    let dir = state.paths.fixture_dir(run.farm_id.as_str(), &commit, name);
    if dir.is_dir() {
        Ok(dir)
    } else {
        Err(FarmError::NotFound(format!(
            "fixture {name} ({})",
            dir.display()
        )))
    }
}

fn copy_tree(source: &Path, target: &Path) -> FarmResult<()> {
    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry.map_err(|err| FarmError::Io(std::io::Error::other(err)))?;
        let Ok(relative) = entry.path().strip_prefix(source) else {
            continue;
        };
        let dest = target.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}
