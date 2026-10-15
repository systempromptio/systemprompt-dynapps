//! Projects `agent-farms.yaml` into rows: the clients with the service
//! account `client` mode acts as, each farm with its client and run-as mode
//! and account, and each agent's own run-as mode and account.
//!
//! An agent's run-as lives only here, never in the farm's Git source, so the
//! sync never writes those columns; [`apply_agent_run_as`] sets them for every
//! agent of a farm (clearing an override the file no longer names) after a
//! seed and after each sync. Only `service_account` mode names an account to
//! resolve here; `user` and `client` are resolved per launch.

use std::path::Path;

use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, UserId};
use systemprompt_web_shared::ProjectId;

use crate::config::{ClientDef, FarmDef, RunAs, RunAsMode, read_farms_doc};
use crate::error::{FarmError, FarmResult};
use crate::repositories::filters::NewFarm;
use crate::repositories::{agents, clients, farms};
use crate::types::{ClientId, FarmId};

fn new_farm(def: &FarmDef, run_as_user_id: Option<UserId>) -> NewFarm {
    let run_as_mode = def.run_as.as_ref().map(|r| r.mode);
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
        run_as_user_id,
        run_as_mode,
        project_id: def
            .project
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(ProjectId::new),
        client_id: def.client.clone().map(ClientId::new),
    }
}

async fn resolve_service_account(pool: &PgPool, owner: &str, run_as: &str) -> FarmResult<UserId> {
    farms::find_service_account(pool, run_as.trim())
        .await?
        .ok_or_else(|| {
            FarmError::Config(format!(
                "{owner}: run_as {run_as:?} is not an active service account"
            ))
        })
}

async fn resolve_run_as(
    pool: &PgPool,
    owner: &str,
    run_as: Option<&RunAs>,
) -> FarmResult<Option<UserId>> {
    match run_as.and_then(RunAs::account) {
        Some(account) => Ok(Some(resolve_service_account(pool, owner, account).await?)),
        None => Ok(None),
    }
}

async fn resolve_client_account(pool: &PgPool, client: &ClientDef) -> FarmResult<Option<UserId>> {
    let Some(account) = client.service_account.as_deref() else {
        return Ok(None);
    };
    let owner = format!("client {:?}", client.slug);
    Ok(Some(resolve_service_account(pool, &owner, account).await?))
}

pub async fn apply_agent_run_as(pool: &PgPool, def: &FarmDef) -> FarmResult<()> {
    let farm_id = FarmId::new(def.id.clone());
    for agent in agents::list_farm_agents(pool, &farm_id).await? {
        let declared = def
            .agents
            .get(agent.id.as_str())
            .and_then(|o| o.run_as.as_ref());
        let owner = format!("farm {:?} agent {:?}", def.id, agent.id.as_str());
        let run_as = resolve_run_as(pool, &owner, declared).await?;
        let mode = declared.map(|r| r.mode);
        let stored_mode = agent.run_as_mode.as_deref().and_then(RunAsMode::parse);
        if run_as != agent.run_as_user_id || mode != stored_mode {
            agents::update_agent_run_as(pool, &farm_id, agent.id.as_str(), run_as.as_ref(), mode)
                .await?;
        }
    }
    Ok(())
}

pub async fn apply_agent_run_as_from_yaml(
    pool: &PgPool,
    services_path: &Path,
    farm_id: &FarmId,
) -> FarmResult<()> {
    let Some(doc) = read_farms_doc(services_path).await? else {
        return Ok(());
    };
    match doc.farms.iter().find(|f| f.id == farm_id.as_str()) {
        Some(def) => apply_agent_run_as(pool, def).await,
        None => Ok(()),
    }
}

pub async fn seed_farms_from_yaml(pool: &PgPool, services_path: &Path) -> FarmResult<usize> {
    let Some(doc) = read_farms_doc(services_path).await? else {
        return Ok(0);
    };
    for client in &doc.clients {
        let account = resolve_client_account(pool, client).await?;
        clients::upsert_client(pool, &client.slug, &client.name, account.as_ref()).await?;
    }
    for def in &doc.farms {
        let owner = format!("farm {:?}", def.id);
        let run_as_user_id = resolve_run_as(pool, &owner, def.run_as.as_ref()).await?;
        farms::upsert_farm(pool, &new_farm(def, run_as_user_id)).await?;
        apply_agent_run_as(pool, def).await?;
    }
    Ok(doc.farms.len())
}
