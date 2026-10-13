//! Writes a parsed farm into the `agent_farm_*` tables at one commit:
//! upserts what the commit ships and deletes what it no longer does.

use std::path::Path;

use sqlx::PgPool;
use systemprompt::identifiers::PluginId;

use super::agent_md::AgentDefinition;
use super::eval_yaml::EvalSuiteDefinition;
use super::fixtures::FixtureManifest;
use super::tree::ParsedFarm;
use crate::error::{FarmError, FarmResult};
use crate::repositories::filters::{NewAgent, NewEvalCase, NewEvalSuite, NewFixture};
use crate::repositories::{agents, evals, fixtures};
use crate::types::{EvalSuiteId, FarmAgentId, FarmId};

pub(super) struct StoredFixture<'a> {
    pub(super) name: &'a str,
    pub(super) manifest: FixtureManifest,
    pub(super) dir: &'a Path,
}

fn new_agent(
    farm_id: &FarmId,
    parsed: &ParsedFarm,
    agent: &AgentDefinition,
    commit: &str,
) -> NewAgent {
    let defaults = &parsed.manifest.defaults;
    NewAgent {
        farm_id: farm_id.clone(),
        id: FarmAgentId::new(agent.name.clone()),
        name: agent.name.clone(),
        description: agent.description.clone(),
        model: agent.model.clone().or_else(|| defaults.model.clone()),
        host: agent.host.clone(),
        tools: agent.tools.clone(),
        plugin_id: PluginId::new(agent.plugin.clone()),
        skills: agent.skills.clone(),
        entry_prompt: agent.entry_prompt.clone(),
        fixture: Some(agent.fixture.clone()),
        max_turns: agent.max_turns.or(defaults.max_turns).map(|t| t as i32),
        max_budget_usd: agent.max_budget_usd.or(defaults.max_budget_usd),
        output_path: agent.output.as_ref().map(|o| o.path.clone()),
        output_format: agent.output.as_ref().map(|o| o.format.clone()),
        source_path: agent.source_path.clone(),
        body: agent.body.clone(),
        synced_commit_sha: commit.to_owned(),
    }
}

fn new_cases(suite: &EvalSuiteDefinition) -> FarmResult<Vec<NewEvalCase>> {
    suite
        .cases
        .iter()
        .map(|case| {
            let assertions =
                serde_json::to_value(&case.assertions).map_err(|source| FarmError::Json {
                    at: suite.source_path.clone(),
                    source,
                })?;
            Ok(NewEvalCase {
                id: case.id.clone(),
                prompt: case.prompt.clone(),
                fixture: case.fixture.clone(),
                timeout_secs: case.timeout_secs.map(|t| t as i32),
                assertions,
                judge_rubric: case.judge_rubric.clone(),
                judge_threshold: case.effective_judge_threshold().map(f64::from),
            })
        })
        .collect()
}

#[must_use]
pub fn eval_suite_id(farm_id: &FarmId, agent: &str) -> EvalSuiteId {
    EvalSuiteId::new(format!("{}.{agent}", farm_id.as_str()))
}

pub(super) async fn persist_farm(
    pool: &PgPool,
    farm_id: &FarmId,
    parsed: &ParsedFarm,
    stored: &[StoredFixture<'_>],
    commit: &str,
) -> FarmResult<usize> {
    for fixture in stored {
        fixtures::upsert_fixture(
            pool,
            &NewFixture {
                farm_id: farm_id.clone(),
                name: fixture.name.to_owned(),
                file_count: fixture.manifest.file_count as i32,
                bytes: fixture.manifest.bytes as i64,
                stored_path: fixture.dir.display().to_string(),
                synced_commit_sha: commit.to_owned(),
            },
        )
        .await?;
    }
    let fixture_names: Vec<String> = stored.iter().map(|f| f.name.to_owned()).collect();
    fixtures::delete_fixtures_not_in(pool, farm_id, &fixture_names).await?;

    for agent in &parsed.agents {
        agents::upsert_agent(pool, &new_agent(farm_id, parsed, agent, commit)).await?;
    }
    let agent_names: Vec<String> = parsed.agents.iter().map(|a| a.name.clone()).collect();
    agents::delete_agents_not_in(pool, farm_id, &agent_names).await?;

    let mut case_total = 0;
    for suite in &parsed.suites {
        let cases = new_cases(suite)?;
        let suite_id = eval_suite_id(farm_id, &suite.agent);
        evals::upsert_eval_suite(
            pool,
            &NewEvalSuite {
                id: suite_id.clone(),
                farm_id: farm_id.clone(),
                agent_id: FarmAgentId::new(suite.agent.clone()),
                source_path: suite.source_path.clone(),
                case_count: cases.len() as i32,
                synced_commit_sha: commit.to_owned(),
            },
        )
        .await?;
        evals::replace_eval_cases(pool, &suite_id, &cases).await?;
        case_total += cases.len();
    }
    let suite_ids: Vec<String> = parsed
        .suites
        .iter()
        .map(|s| eval_suite_id(farm_id, &s.agent).into_inner())
        .collect();
    evals::delete_eval_suites_not_in(pool, farm_id, &suite_ids).await?;
    Ok(case_total)
}
