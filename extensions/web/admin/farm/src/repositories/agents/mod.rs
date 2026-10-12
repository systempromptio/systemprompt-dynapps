//! Queries over the agents a farm sync projected from the farm's agent files.

use sqlx::PgPool;
use systemprompt::identifiers::PluginId;

use super::filters::NewAgent;
use super::rows::AgentRow;
use crate::types::{FarmAgentId, FarmId};

pub async fn list_farm_agents(
    pool: &PgPool,
    farm_id: &FarmId,
) -> Result<Vec<AgentRow>, sqlx::Error> {
    sqlx::query_file_as!(
        AgentRow,
        "src/repositories/agents/list_farm_agents.sql",
        farm_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_all_agents(pool: &PgPool) -> Result<Vec<AgentRow>, sqlx::Error> {
    sqlx::query_file_as!(AgentRow, "src/repositories/agents/list_all_agents.sql")
        .fetch_all(pool)
        .await
}

pub async fn find_agent(
    pool: &PgPool,
    farm_id: &FarmId,
    id: &FarmAgentId,
) -> Result<Option<AgentRow>, sqlx::Error> {
    sqlx::query_file_as!(
        AgentRow,
        "src/repositories/agents/find_agent.sql",
        farm_id.as_str(),
        id.as_str(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn upsert_agent(pool: &PgPool, agent: &NewAgent) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/agents/upsert_agent.sql",
        agent.farm_id.as_str(),
        agent.id.as_str(),
        agent.name,
        agent.description,
        agent.model,
        &agent.tools,
        agent.plugin_id.as_str(),
        &agent.skills,
        agent.entry_prompt,
        agent.fixture,
        agent.max_turns,
        agent.max_budget_usd,
        agent.output_path,
        agent.output_format,
        agent.source_path,
        agent.body,
        agent.synced_commit_sha,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_agents_not_in(
    pool: &PgPool,
    farm_id: &FarmId,
    keep: &[String],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/agents/delete_agents_not_in.sql",
        farm_id.as_str(),
        keep,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn count_agents(pool: &PgPool, farm_id: Option<&FarmId>) -> Result<i64, sqlx::Error> {
    sqlx::query_file_scalar!(
        "src/repositories/agents/count_agents.sql",
        farm_id.map(FarmId::as_str),
    )
    .fetch_one(pool)
    .await
}
