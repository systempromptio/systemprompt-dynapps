//! Lays out one run directory before Claude Code starts.
//!
//! `home/.claude/settings.json` so the host `~/.claude` is never read,
//! `workspace/` seeded from the fixture, `logs/`, `agents.json`,
//! `settings.json`, and the agent's plugin materialised under `plugin/`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::WalkDir;

use super::diff::{FileStat, snapshot};
use super::plugin_materialise::materialise_plugin;
use crate::error::{FarmError, FarmResult};
use crate::repositories::rows::{AgentRow, RunRow};
use crate::repositories::{agents, farms, fixtures};
use crate::state::FarmState;

#[derive(Debug, Clone)]
pub struct RunLayout {
    pub root: PathBuf,
    pub home: PathBuf,
    pub workspace: PathBuf,
    pub logs: PathBuf,
}

impl RunLayout {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            home: root.join("home"),
            workspace: root.join("workspace"),
            logs: root.join("logs"),
            root,
        }
    }

    #[must_use]
    pub fn agents_json(&self) -> PathBuf {
        self.root.join("agents.json")
    }

    #[must_use]
    pub fn settings_json(&self) -> PathBuf {
        self.root.join("settings.json")
    }
}

#[derive(Debug, Clone)]
pub struct PreparedRun {
    pub layout: RunLayout,
    pub agent: AgentRow,
    pub plugin_dir: PathBuf,
    pub prompt: String,
    pub before: Vec<FileStat>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AgentDef<'a> {
    description: &'a str,
    prompt: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct Settings<'a> {
    permissions: Permissions<'a>,
    #[serde(rename = "includeCoAuthoredBy")]
    include_co_authored_by: bool,
}

#[derive(Debug, Serialize)]
struct Permissions<'a> {
    #[serde(rename = "defaultMode")]
    default_mode: &'a str,
}

#[must_use]
pub fn render_entry_prompt(template: &str, fixture_path: &str, case_prompt: &str) -> String {
    template
        .replace("{{fixture_path}}", fixture_path)
        .replace("{{ fixture_path }}", fixture_path)
        .replace("{{case.prompt}}", case_prompt)
        .replace("{{ case.prompt }}", case_prompt)
}

pub async fn prepare_run(state: &FarmState, run: &RunRow) -> FarmResult<PreparedRun> {
    let pool = state.pool.as_ref();
    let agent = agents::find_agent(pool, &run.farm_id, &run.agent_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("agent {}/{}", run.farm_id, run.agent_id)))?;
    let fixture_src = match run.fixture.as_deref() {
        Some(name) => Some(resolve_fixture(state, run, name).await?),
        None => None,
    };

    let layout = RunLayout::new(state.paths.run_dir(run.id.as_str()));
    let services_root = state.paths.composed_services_root.clone();
    let permission_mode = run.permission_mode.clone();
    let blocking_agent = agent.clone();
    let blocking_layout = layout.clone();
    let (plugin_dir, before, notes) = tokio::task::spawn_blocking(move || {
        lay_out(
            &blocking_layout,
            &blocking_agent,
            fixture_src.as_deref(),
            &services_root,
            &permission_mode,
        )
    })
    .await
    .map_err(|source| FarmError::Task {
        step: "prepare",
        source,
    })??;

    let workspace = layout.workspace.to_string_lossy().into_owned();
    let prompt = render_entry_prompt(&run.prompt, &workspace, "");
    Ok(PreparedRun {
        layout,
        agent,
        plugin_dir,
        prompt,
        before,
        notes,
    })
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

fn lay_out(
    layout: &RunLayout,
    agent: &AgentRow,
    fixture: Option<&Path>,
    services_root: &Path,
    permission_mode: &str,
) -> FarmResult<(PathBuf, Vec<FileStat>, Vec<String>)> {
    let claude_dir = layout.home.join(".claude");
    for dir in [&claude_dir, &layout.workspace, &layout.logs] {
        std::fs::create_dir_all(dir)?;
    }
    if let Some(source) = fixture {
        copy_tree(source, &layout.workspace)?;
    }

    let mut defs = BTreeMap::new();
    defs.insert(
        agent.id.as_str(),
        AgentDef {
            description: &agent.description,
            prompt: &agent.body,
            tools: (!agent.tools.is_empty()).then_some(agent.tools.as_slice()),
            model: agent.model.as_deref(),
        },
    );
    std::fs::write(layout.agents_json(), to_json(&defs)?)?;

    let settings = to_json(&Settings {
        permissions: Permissions {
            default_mode: permission_mode,
        },
        include_co_authored_by: false,
    })?;
    std::fs::write(layout.settings_json(), &settings)?;
    std::fs::write(claude_dir.join("settings.json"), &settings)?;

    let plugin = materialise_plugin(services_root, &agent.plugin_id, &layout.root.join("plugin"))?;
    let notes = plugin
        .missing_skills
        .iter()
        .map(|skill| format!("skill {skill} is not in the composed services tree"))
        .collect();
    Ok((plugin.dir, snapshot(&layout.workspace), notes))
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

fn to_json<T: Serialize>(value: &T) -> FarmResult<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|source| FarmError::Json {
        at: "run layout".to_owned(),
        source,
    })
}
