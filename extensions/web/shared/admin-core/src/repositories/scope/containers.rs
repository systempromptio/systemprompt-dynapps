//! Every group and every project, for the scope-filter selects a list page
//! offers and the pages that enumerate containers.

use sqlx::PgPool;
use systemprompt_web_shared::{GroupId, ProjectId};

use crate::types::groups::{GroupRecord, GroupSummary};
use crate::types::projects::{ProjectRow, ProjectSummary};

pub async fn list_group_options(pool: &PgPool) -> Result<Vec<GroupRecord>, sqlx::Error> {
    sqlx::query_as!(
        GroupRecord,
        r#"SELECT id AS "id: GroupId", name, description, is_system, source FROM groups ORDER BY is_system, name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_project_options(pool: &PgPool) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as!(
        ProjectRow,
        r#"SELECT id AS "id: ProjectId", name, description, source FROM projects ORDER BY name"#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_group_summaries(pool: &PgPool) -> Result<Vec<GroupSummary>, sqlx::Error> {
    sqlx::query_as!(
        GroupSummary,
        r#"
        SELECT
            g.id AS "id!: GroupId",
            g.name AS "name!",
            g.description,
            g.is_system AS "is_system!",
            COALESCE(m.member_count, 0)::BIGINT AS "member_count!",
            COALESCE(pc.project_count, 0)::BIGINT AS "project_count!",
            COALESCE(u.active_members, 0)::BIGINT AS "active_members_30d!",
            COALESCE(u.requests, 0)::BIGINT AS "requests_30d!",
            COALESCE(u.cost_microdollars, 0)::BIGINT AS "cost_30d_microdollars!"
        FROM groups g
        LEFT JOIN (
            SELECT group_id, COUNT(DISTINCT user_id) AS member_count
            FROM user_groups GROUP BY group_id
        ) m ON m.group_id = g.id
        LEFT JOIN (
            SELECT ug.group_id, COUNT(DISTINCT pm.project_id) AS project_count
            FROM user_groups ug
            JOIN user_projects pm ON pm.user_id = ug.user_id
            GROUP BY ug.group_id
        ) pc ON pc.group_id = g.id
        LEFT JOIN (
            SELECT ug.group_id,
                   COUNT(DISTINCT r.user_id) AS active_members,
                   COUNT(r.id) AS requests,
                   COALESCE(SUM(r.cost_microdollars), 0) AS cost_microdollars
            FROM user_groups ug
            JOIN ai_requests r
              ON r.user_id = ug.user_id
             AND r.created_at >= NOW() - INTERVAL '30 days'
            GROUP BY ug.group_id
        ) u ON u.group_id = g.id
        ORDER BY g.is_system, g.name
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_project_summaries(pool: &PgPool) -> Result<Vec<ProjectSummary>, sqlx::Error> {
    sqlx::query_as!(
        ProjectSummary,
        r#"
        SELECT
            p.id AS "id!: ProjectId",
            p.name AS "name!",
            p.description,
            COALESCE(m.member_count, 0)::BIGINT AS "member_count!",
            COALESCE(u.active_members, 0)::BIGINT AS "active_members_30d!",
            COALESCE(u.requests, 0)::BIGINT AS "requests_30d!",
            COALESCE(u.cost_microdollars, 0)::BIGINT AS "cost_30d_microdollars!"
        FROM projects p
        LEFT JOIN (
            SELECT project_id, COUNT(*) AS member_count
            FROM user_projects GROUP BY project_id
        ) m ON m.project_id = p.id
        LEFT JOIN (
            SELECT pm.project_id,
                   COUNT(DISTINCT r.user_id) AS active_members,
                   COUNT(r.id) AS requests,
                   COALESCE(SUM(r.cost_microdollars), 0) AS cost_microdollars
            FROM user_projects pm
            JOIN ai_requests r
              ON r.user_id = pm.user_id
             AND r.created_at >= NOW() - INTERVAL '30 days'
            GROUP BY pm.project_id
        ) u ON u.project_id = p.id
        ORDER BY p.name
        "#
    )
    .fetch_all(pool)
    .await
}
