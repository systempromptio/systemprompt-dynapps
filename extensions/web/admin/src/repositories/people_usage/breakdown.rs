//! What a group or project spent its window on: models, skills, MCP tools,
//! and the other people container each one overlaps with.
//!
//! Same contract as the parent module — the membership relation is chosen by
//! a bound parameter, never interpolated — so every statement here is static
//! and macro-verified. Tools are read from `mcp_tool_executions` rather than
//! the pre-rolled daily table because a leaderboard an operator reads wants
//! the server name beside the tool name, and only the raw rows carry it.

use std::collections::HashMap;

use serde::Serialize;
use sqlx::PgPool;

use super::Scope;

/// One row of the model mix: the request and token split for one model.
#[derive(Debug, Clone, Serialize)]
pub struct ModelUsageRow {
    pub model: String,
    pub provider: String,
    pub requests: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_microdollars: i64,
}

/// One skill and how much of the window it accounted for.
#[derive(Debug, Clone, Serialize)]
pub struct SkillUsageRow {
    pub skill: String,
    pub invocations: i64,
    pub users: i64,
}

/// One MCP tool, named by its server, and how much of the window it took.
#[derive(Debug, Clone, Serialize)]
pub struct ToolUsageRow {
    pub server_name: String,
    pub tool_name: String,
    pub invocations: i64,
    pub users: i64,
}

/// A container on the other side of the membership overlap, with how many of
/// this container's people are in it.
#[derive(Debug, Clone, Serialize)]
pub struct LinkedScopeRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i64,
}

pub async fn list_scope_top_models(
    pool: &PgPool,
    scope: Scope,
    id: &str,
    window_days: i32,
    limit: i64,
) -> Result<Vec<ModelUsageRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH membership AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'group'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'project'
           )
           SELECT COALESCE(r.model, 'unknown') AS "model!",
                  COALESCE(r.provider, 'unknown') AS "provider!",
                  COUNT(*)::BIGINT AS "requests!",
                  COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "tokens_in!",
                  COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "tokens_out!",
                  COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost_microdollars!"
           FROM membership m
           JOIN ai_requests r ON r.user_id = m.user_id
           WHERE m.scope_id = $2
             AND r.created_at >= NOW() - make_interval(days => $3)
           GROUP BY 1, 2
           ORDER BY 3 DESC, 1
           LIMIT $4"#,
        scope.as_str(),
        id,
        window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ModelUsageRow {
            model: row.model,
            provider: row.provider,
            requests: row.requests,
            tokens_in: row.tokens_in,
            tokens_out: row.tokens_out,
            cost_microdollars: row.cost_microdollars,
        })
        .collect())
}

pub async fn list_scope_top_skills(
    pool: &PgPool,
    scope: Scope,
    id: &str,
    window_days: i32,
    limit: i64,
) -> Result<Vec<SkillUsageRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH membership AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'group'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'project'
           )
           SELECT e.skill AS "skill!",
                  COUNT(*)::BIGINT AS "invocations!",
                  COUNT(DISTINCT e.user_id)::BIGINT AS "users!"
           FROM membership m
           JOIN skill_invocation_events e ON e.user_id = m.user_id
           WHERE m.scope_id = $2
             AND e.skill IS NOT NULL
             AND e.invoked_at >= NOW() - make_interval(days => $3)
           GROUP BY 1
           ORDER BY 2 DESC, 1
           LIMIT $4"#,
        scope.as_str(),
        id,
        window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| SkillUsageRow {
            skill: row.skill,
            invocations: row.invocations,
            users: row.users,
        })
        .collect())
}

pub async fn list_scope_top_tools(
    pool: &PgPool,
    scope: Scope,
    id: &str,
    window_days: i32,
    limit: i64,
) -> Result<Vec<ToolUsageRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH membership AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'group'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'project'
           )
           SELECT x.server_name AS "server_name!",
                  x.tool_name AS "tool_name!",
                  COUNT(*)::BIGINT AS "invocations!",
                  COUNT(DISTINCT x.user_id)::BIGINT AS "users!"
           FROM membership m
           JOIN mcp_tool_executions x ON x.user_id = m.user_id
           WHERE m.scope_id = $2
             AND x.started_at >= NOW() - make_interval(days => $3)
           GROUP BY 1, 2
           ORDER BY 3 DESC, 2
           LIMIT $4"#,
        scope.as_str(),
        id,
        window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ToolUsageRow {
            server_name: row.server_name,
            tool_name: row.tool_name,
            invocations: row.invocations,
            users: row.users,
        })
        .collect())
}

pub async fn list_linked_scopes(
    pool: &PgPool,
    scope: Scope,
    id: &str,
) -> Result<Vec<LinkedScopeRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH membership AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'group'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'project'
           ), linked AS (
               SELECT ug.user_id, g.id, g.name, g.description
               FROM user_groups ug JOIN groups g ON g.id = ug.group_id
               WHERE $1 = 'project'
               UNION ALL
               SELECT DISTINCT pm.user_id, p.id, p.name, p.description
               FROM project_members pm JOIN projects p ON p.id = pm.project_id
               WHERE $1 = 'group'
           )
           SELECT linked.id AS "id!",
                  linked.name AS "name!",
                  linked.description AS "description?",
                  COUNT(DISTINCT linked.user_id)::BIGINT AS "member_count!"
           FROM linked
           WHERE linked.user_id IN (SELECT m.user_id FROM membership m WHERE m.scope_id = $2)
           GROUP BY 1, 2, 3
           ORDER BY 4 DESC, 2"#,
        scope.as_str(),
        id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| LinkedScopeRow {
            id: row.id,
            name: row.name,
            description: row.description,
            member_count: row.member_count,
        })
        .collect())
}

pub async fn count_linked_scopes(
    pool: &PgPool,
    scope: Scope,
) -> Result<HashMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH membership AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'group'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'project'
           ), linked AS (
               SELECT user_id, group_id AS scope_id FROM user_groups WHERE $1 = 'project'
               UNION ALL
               SELECT DISTINCT user_id, project_id AS scope_id FROM project_members WHERE $1 = 'group'
           )
           SELECT m.scope_id AS "id!", COUNT(DISTINCT l.scope_id)::BIGINT AS "n!"
           FROM membership m
           JOIN linked l ON l.user_id = m.user_id
           GROUP BY 1"#,
        scope.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| (row.id, row.n)).collect())
}
