//! Projects `agent-farms.yaml` into rows: the clients, each farm with its
//! client and run-as account, and each agent's own run-as account.
//!
//! An agent's run-as lives only here, never in the farm's Git source, so the
//! sync never writes that column; [`apply_agent_run_as`] sets it for every
//! agent of a farm (clearing an override the file no longer names) after a
//! seed and after each sync.

use std::path::Path;

use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, UserId};
use systemprompt_web_shared::ProjectId;

use crate::config::{FarmDef, read_farms_doc};
use crate::error::{FarmError, FarmResult};
use crate::repositories::filters::NewFarm;
use crate::repositories::{agents, clients, farms};
use crate::types::{ClientId, FarmId};

fn new_farm(def: &FarmDef, run_as_user_id: Option<UserId>) -> NewFarm {
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

async fn resolve_run_as(pool: &PgPool, def: &FarmDef) -> FarmResult<Option<UserId>> {
    let Some(run_as) = def.run_as.as_deref() else {
        return Ok(None);
    };
    let owner = format!("farm {:?}", def.id);
    Ok(Some(resolve_service_account(pool, &owner, run_as).await?))
}

pub async fn apply_agent_run_as(pool: &PgPool, def: &FarmDef) -> FarmResult<()> {
    let farm_id = FarmId::new(def.id.clone());
    for agent in agents::list_farm_agents(pool, &farm_id).await? {
        let run_as = match def
            .agents
            .get(agent.id.as_str())
            .and_then(|o| o.run_as.as_deref())
        {
            Some(run_as) => {
                let owner = format!("farm {:?} agent {:?}", def.id, agent.id.as_str());
                Some(resolve_service_account(pool, &owner, run_as).await?)
            },
            None => None,
        };
        if run_as != agent.run_as_user_id {
            agents::update_agent_run_as(pool, &farm_id, agent.id.as_str(), run_as.as_ref()).await?;
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
        clients::upsert_client(pool, &client.slug, &client.name, client.odoo_partner_id).await?;
    }
    for def in &doc.farms {
        let run_as_user_id = resolve_run_as(pool, def).await?;
        farms::upsert_farm(pool, &new_farm(def, run_as_user_id)).await?;
        apply_agent_run_as(pool, def).await?;
    }
    Ok(doc.farms.len())
}
