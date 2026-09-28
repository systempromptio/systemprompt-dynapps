//! The groups listing's numbers, under exclusive attribution.
//!
//! The spend columns are not computed here. They come from
//! [`crate::repositories::people_usage::totals::list_scope_totals`], which
//! groups every request in the window and files what no primary group covers
//! under `unattributed` — so the rows this returns provably sum to the instance
//! total. Recomputing the same figures from a second statement would be one
//! refactor away from quietly disagreeing with the bucket that is supposed to
//! complete them.
//!
//! Everything else — membership, the projects a group's people reach, its
//! busiest model — is metadata the totals cannot carry, and is read here.
//! Membership counts read `user_groups` (a person in two groups is in both);
//! `active_members` and the busiest model read the request-stamped scope, so
//! they describe the same traffic as the spend columns beside them.
//!
//! The remainder is a row of the list, not a second return value: the rows
//! partition the instance, and a caller handed it separately could drop it.
//! `MAX_GROUPS` guards against a runaway directory; the handler pages.

use std::collections::HashMap;

use sqlx::PgPool;

use crate::repositories::people_usage::totals::list_scope_totals;
use crate::repositories::scope::membership::UNATTRIBUTED;

pub use crate::repositories::scope::membership::UNATTRIBUTED as UNATTRIBUTED_ROW;
use crate::repositories::scope::{Attribution, ScopeKind};

const MAX_GROUPS: i64 = 500;

/// One group on the listing: who is in it, what it reaches, what it spent.
#[derive(Debug, Clone, Default)]
pub struct GroupUsageRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub source: String,
    pub member_count: i64,
    pub project_count: i64,
    pub active_members: i64,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub top_model: Option<String>,
    pub top_model_requests: i64,
}

pub async fn list_groups_with_usage(
    pool: &PgPool,
    window_days: i32,
) -> Result<Vec<GroupUsageRow>, sqlx::Error> {
    let totals = list_scope_totals(pool, ScopeKind::Group, Attribution::Exclusive, window_days)
        .await?
        .into_iter()
        .map(|row| (row.scope_id.clone(), row))
        .collect::<HashMap<_, _>>();

    let meta = list_group_metadata(pool, window_days).await?;

    let mut rows: Vec<GroupUsageRow> = meta
        .into_iter()
        .map(|mut row| {
            if let Some(total) = totals.get(&row.id) {
                row.requests = total.requests;
                row.tokens = total.tokens;
                row.cost_microdollars = total.cost_microdollars;
            }
            row
        })
        .collect();

    let remainder = totals.get(UNATTRIBUTED);
    rows.push(GroupUsageRow {
        id: UNATTRIBUTED.to_owned(),
        name: "Unattributed".to_owned(),
        is_system: true,
        source: "system".to_owned(),
        requests: remainder.map_or(0, |t| t.requests),
        tokens: remainder.map_or(0, |t| t.tokens),
        cost_microdollars: remainder.map_or(0, |t| t.cost_microdollars),
        ..GroupUsageRow::default()
    });
    Ok(rows)
}

async fn list_group_metadata(
    pool: &PgPool,
    window_days: i32,
) -> Result<Vec<GroupUsageRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#", win AS (
               SELECT rs.scope_id, w.user_id, w.model
               FROM request_scope rs
               JOIN ai_requests w ON w.id = rs.request_id
               WHERE w.created_at >= NOW() - make_interval(days => $3)
                 AND rs.scope_id IS NOT NULL
           ), act AS (
               SELECT w.scope_id, COUNT(DISTINCT w.user_id) AS active_members
               FROM win w
               GROUP BY 1
           ), model_rank AS (
               SELECT t.scope_id, t.model, t.requests,
                      ROW_NUMBER() OVER (
                          PARTITION BY t.scope_id ORDER BY t.requests DESC, t.model
                      ) AS rn
               FROM (
                   SELECT w.scope_id, w.model, COUNT(*) AS requests
                   FROM win w
                   WHERE w.model IS NOT NULL
                   GROUP BY 1, 2
               ) t
           )
           SELECT g.id AS "id!",
                  g.name AS "name!",
                  g.description,
                  g.is_system AS "is_system!",
                  g.source AS "source!",
                  COALESCE(mc.member_count, 0)::BIGINT AS "member_count!",
                  COALESCE(pc.project_count, 0)::BIGINT AS "project_count!",
                  COALESCE(act.active_members, 0)::BIGINT AS "active_members!",
                  model_rank.model AS "top_model?",
                  COALESCE(model_rank.requests, 0)::BIGINT AS "top_model_requests!"
           FROM groups g
           LEFT JOIN (
               SELECT ug.group_id, COUNT(DISTINCT ug.user_id) AS member_count
               FROM user_groups ug GROUP BY 1
           ) mc ON mc.group_id = g.id
           LEFT JOIN (
               SELECT ug.group_id, COUNT(DISTINCT pm.project_id) AS project_count
               FROM user_groups ug
               JOIN project_members pm ON pm.user_id = ug.user_id
               GROUP BY 1
           ) pc ON pc.group_id = g.id
           LEFT JOIN act ON act.scope_id = g.id
           LEFT JOIN model_rank ON model_rank.scope_id = g.id AND model_rank.rn = 1
           ORDER BY g.is_system, g.name
           LIMIT $4"#,
        ScopeKind::Group.as_str(),
        Attribution::Exclusive.is_exclusive(),
        window_days,
        MAX_GROUPS
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| GroupUsageRow {
            id: row.id,
            name: row.name,
            description: row.description,
            is_system: row.is_system,
            source: row.source,
            member_count: row.member_count,
            project_count: row.project_count,
            active_members: row.active_members,
            top_model: row.top_model,
            top_model_requests: row.top_model_requests,
            ..GroupUsageRow::default()
        })
        .collect())
}
