//! `find_run_facts` and `list_run_facts`: one statement (`facts.sql`) for
//! both, narrowed to one id for the find.
//!
//! A workflow step's project is read after the fact through `workflow`,
//! since core's row carries no project and the table that does is not on
//! every instance. A workflow step's attempt is the last segment of its
//! `<run>/<step>/<attempt>` origin id; every other origin reports core's
//! own attempt counter.

use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{AgentRunId, UserId};
use systemprompt_web_shared::ProjectId;

use super::MAX_LIST_LIMIT;
use super::rows::{RunFacts, RunFilter, RunScope};
use super::validity::output_schema_valid;
use super::workflow::{WORKFLOW_STEP_ORIGIN, list_workflow_projects, workflow_run_id};

pub(super) struct ScopeParams {
    pub user_id: Option<UserId>,
    pub project_ids: Option<Vec<String>>,
}

impl From<&RunScope> for ScopeParams {
    fn from(scope: &RunScope) -> Self {
        match scope {
            RunScope::Own(user) => Self {
                user_id: Some(user.clone()),
                project_ids: None,
            },
            RunScope::Projects(ids) => Self {
                user_id: None,
                project_ids: Some(ids.iter().map(|p| p.as_str().to_owned()).collect()),
            },
            RunScope::All => Self {
                user_id: None,
                project_ids: None,
            },
        }
    }
}

pub async fn find_run_facts(
    pool: &PgPool,
    run_id: &AgentRunId,
) -> Result<Option<RunFacts>, sqlx::Error> {
    let scope = ScopeParams::from(&RunScope::All);
    let mut rows = list_rows(
        pool,
        Some(run_id.as_str()),
        &scope,
        &RunFilter::default(),
        1,
    )
    .await?;
    Ok(rows.pop())
}

pub async fn list_run_facts(
    pool: &PgPool,
    scope: &RunScope,
    filter: &RunFilter,
) -> Result<Vec<RunFacts>, sqlx::Error> {
    let limit = filter
        .limit
        .unwrap_or(MAX_LIST_LIMIT)
        .clamp(1, MAX_LIST_LIMIT);
    list_rows(pool, None, &ScopeParams::from(scope), filter, limit).await
}

async fn list_rows(
    pool: &PgPool,
    run_id: Option<&str>,
    scope: &ScopeParams,
    filter: &RunFilter,
    limit: i64,
) -> Result<Vec<RunFacts>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/agent_run_facts/facts.sql",
        run_id,
        scope.user_id.as_ref().map(UserId::as_str),
        scope.project_ids.as_deref(),
        filter.origin_kind,
        filter.origin_id,
        filter.status,
        filter.since,
        filter.until,
        filter.farm_id,
        limit,
    )
    .fetch_all(pool)
    .await?;

    let mut facts: Vec<RunFacts> = rows
        .into_iter()
        .map(|row| {
            let output_schema_valid =
                output_schema_valid(row.output_schema.as_ref(), row.structured_output.as_ref());
            let Json(tools_by_name) = row.tools_by_name;
            RunFacts {
                attempt: attempt_of(&row.origin_kind, &row.origin_id, row.attempt),
                budget_exceeded: row.error_class.as_deref() == Some("budget_exceeded"),
                timed_out: row.status == "timed_out"
                    || row.error_class.as_deref() == Some("timeout"),
                run_id: row.run_id,
                origin_kind: row.origin_kind,
                origin_id: row.origin_id,
                user_id: row.user_id,
                project_id: row.farm_project_id,
                status: row.status,
                error_class: row.error_class,
                started_at: row.started_at,
                finished_at: row.finished_at,
                duration_ms: row.duration_ms,
                turns: row.turns,
                input_tokens: row.input_tokens,
                output_tokens: row.output_tokens,
                cost_microdollars_reported: row.cost_microdollars_reported,
                cost_microdollars_metered: row.cost_microdollars_metered,
                tool_use_count: row.tool_use_count,
                tool_result_count: row.tool_result_count,
                tool_result_error_count: row.tool_result_error_count,
                tools_by_name,
                hook_allow: row.hook_allow,
                hook_deny: row.hook_deny,
                governance_allow: row.governance_allow,
                governance_deny: row.governance_deny,
                structured_output: row.structured_output,
                output_schema_valid,
                diff_summary: row.diff_summary,
            }
        })
        .collect();
    attach_workflow_projects(pool, &mut facts).await?;
    Ok(facts)
}

fn attempt_of(origin_kind: &str, origin_id: &str, row_attempt: i32) -> i32 {
    if origin_kind != WORKFLOW_STEP_ORIGIN {
        return row_attempt;
    }
    origin_id
        .rsplit('/')
        .next()
        .and_then(|segment| segment.parse().ok())
        .unwrap_or(row_attempt)
}

async fn attach_workflow_projects(
    pool: &PgPool,
    facts: &mut [RunFacts],
) -> Result<(), sqlx::Error> {
    let run_ids: Vec<String> = facts
        .iter()
        .filter(|f| f.origin_kind == WORKFLOW_STEP_ORIGIN && f.project_id.is_none())
        .filter_map(|f| workflow_run_id(&f.origin_id).map(ToOwned::to_owned))
        .collect();
    if run_ids.is_empty() {
        return Ok(());
    }
    let projects = list_workflow_projects(pool, &run_ids).await?;
    for fact in facts
        .iter_mut()
        .filter(|f| f.origin_kind == WORKFLOW_STEP_ORIGIN)
    {
        if let Some(project) = workflow_run_id(&fact.origin_id).and_then(|id| projects.get(id)) {
            fact.project_id = Some(project.clone());
        }
    }
    Ok(())
}
