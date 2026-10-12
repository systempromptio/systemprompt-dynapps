//! Farm sync: capturing a farm's Git source at a ref, parsing every
//! definition in it, writing its fixtures under the commit's directory, and
//! upserting agents, fixtures and eval suites.
//!
//! A failure at any step records `sync_error` on the farm and leaves the
//! previously synced rows in place.

pub mod agent_md;
pub mod capture;
pub mod eval_yaml;
pub mod farm_yaml;
pub mod fixtures;
mod persist;
pub mod tree;
pub mod validate;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, UserId};

use crate::config::{FarmDef, read_farms_doc};
use crate::error::{FarmError, FarmResult};
use crate::repositories::farms;
use crate::repositories::filters::NewFarm;
use crate::repositories::rows::FarmRow;
use crate::state::FarmState;
use crate::types::FarmId;

pub use agent_md::{
    AgentDefinition, AgentOutput, DEFAULT_HOST, HOSTS, parse_agent_markdown, render_entry_prompt,
};
pub use capture::{
    CapturedFarm, CapturedFile, FarmSourceCapture, InMemoryCapture, NativeCapture, github_https_url,
};
pub use eval_yaml::{Assertion, EvalCaseDefinition, EvalSuiteDefinition, parse_eval_suite};
pub use farm_yaml::{FarmManifest, ManifestDefaults, check_agents_listed, parse_farm_manifest};
pub use fixtures::{
    FixtureFile, FixtureManifest, fixture_manifest, group_fixtures, materialise_fixture,
};
pub use persist::eval_suite_id;
pub use tree::{ParsedFarm, parse_farm_tree};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FarmSyncReport {
    pub commit: String,
    pub agents: usize,
    pub fixtures: usize,
    pub suites: usize,
    pub cases: usize,
    pub warnings: Vec<String>,
}

pub async fn sync_farm(
    state: &FarmState,
    capture: Arc<dyn FarmSourceCapture>,
    farm_id: &FarmId,
    actor: Option<&UserId>,
) -> FarmResult<FarmSyncReport> {
    let pool: &PgPool = &state.pool;
    let farm = farms::find_farm(pool, farm_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("farm {}", farm_id.as_str())))?;
    match sync_captured(state, capture, &farm).await {
        Ok(report) => {
            farms::update_farm_sync_ok(pool, farm_id, &report.commit, actor).await?;
            tracing::info!(farm = farm_id.as_str(), commit = %report.commit, "farm synced");
            Ok(report)
        },
        Err(err) => {
            if let Err(record) =
                farms::update_farm_sync_failed(pool, farm_id, &err.to_string()).await
            {
                tracing::warn!(farm = farm_id.as_str(), error = %record, "recording sync failure");
            }
            Err(err)
        },
    }
}

async fn sync_captured(
    state: &FarmState,
    capture: Arc<dyn FarmSourceCapture>,
    farm: &FarmRow,
) -> FarmResult<FarmSyncReport> {
    let url = github_https_url(&farm.source_repo);
    let reference = farm.source_ref.clone();
    let subdir = farm.source_subdir.clone();
    let captured = tokio::task::spawn_blocking(move || capture.capture(&url, &reference, &subdir))
        .await
        .map_err(|source| FarmError::Task {
            step: "capture",
            source,
        })??;
    let parsed = parse_farm_tree(&captured.files)?;
    if parsed.manifest.marketplace != farm.marketplace_id.as_str() {
        return Err(FarmError::Validation(format!(
            "farm.yaml marketplace {:?} does not match the configured {:?}",
            parsed.manifest.marketplace,
            farm.marketplace_id.as_str()
        )));
    }
    let commit = captured.commit;
    let farm_key = farm.id.as_str();
    let dirs: Vec<PathBuf> = parsed
        .fixtures
        .keys()
        .map(|name| state.paths.fixture_dir(farm_key, &commit, name))
        .collect();
    let manifests = write_fixtures(&parsed, &dirs).await?;
    let stored: Vec<persist::StoredFixture<'_>> = parsed
        .fixtures
        .keys()
        .zip(dirs.iter())
        .zip(manifests)
        .map(|((name, dir), manifest)| persist::StoredFixture {
            name,
            manifest,
            dir: dir.as_path(),
        })
        .collect();
    let cases = persist::persist_farm(&state.pool, &farm.id, &parsed, &stored, &commit).await?;
    Ok(FarmSyncReport {
        commit,
        agents: parsed.agents.len(),
        fixtures: stored.len(),
        suites: parsed.suites.len(),
        cases,
        warnings: parsed.warnings,
    })
}

async fn write_fixtures(parsed: &ParsedFarm, dirs: &[PathBuf]) -> FarmResult<Vec<FixtureManifest>> {
    let jobs: Vec<(PathBuf, Vec<FixtureFile>)> = dirs
        .iter()
        .cloned()
        .zip(parsed.fixtures.values().cloned())
        .collect();
    tokio::task::spawn_blocking(move || {
        jobs.iter()
            .map(|(dir, files)| materialise_fixture(dir, files))
            .collect::<FarmResult<Vec<_>>>()
    })
    .await
    .map_err(|e| FarmError::Io(std::io::Error::other(e)))?
}

fn new_farm(def: &FarmDef) -> NewFarm {
    NewFarm {
        id: FarmId::new(def.id.clone()),
        name: def.name.clone(),
        description: def.description.clone(),
        marketplace_id: MarketplaceId::new(def.marketplace.clone()),
        source_repo: def.source.github.clone(),
        source_ref: def.source.r#ref.clone(),
        source_subdir: def.source.subdir.clone(),
        permission_mode: def.defaults.permission_mode.clone(),
        timeout_seconds: i32::try_from(def.defaults.timeout_seconds).unwrap_or(i32::MAX),
        max_turns: i32::try_from(def.defaults.max_turns).unwrap_or(i32::MAX),
        max_budget_usd: def.defaults.max_budget_usd,
        model: def.defaults.model.clone(),
    }
}

pub async fn seed_farms_from_yaml(pool: &PgPool, services_path: &Path) -> FarmResult<usize> {
    let Some(doc) = read_farms_doc(services_path).await? else {
        return Ok(0);
    };
    for def in &doc.farms {
        farms::upsert_farm(pool, &new_farm(def)).await?;
    }
    Ok(doc.farms.len())
}
