//! Warn-mode reads: what governance and the safety scanners would have refused.
//!
//! A policy configured `mode: warn` writes a `governance_decisions` row with
//! `decision = 'warn'` carrying the reason it would have denied on, and the
//! gateway's scanners under `safety.mode: warn` write `ai_safety_findings`
//! rows whose `blocked` column stays false. Both are ordinary audit history;
//! this module is the read side that turns them into something an operator can
//! retune a threshold from.
//!
//! The rollup is grouped by policy, tool and user at once and re-aggregated in
//! Rust rather than issued three times with a different `GROUP BY`. The
//! combinations are bounded by the policy count times the tool count, not by
//! traffic, and one query keeps the three views of a window provably
//! consistent with each other.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct WarningRollupRow {
    pub policy: String,
    pub tool_name: String,
    pub user_id: UserId,
    pub count: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub example_reason: String,
}

#[derive(Debug, Clone)]
pub struct WarningDecisionRow {
    pub id: String,
    pub policy: String,
    pub tool_name: String,
    pub user_id: UserId,
    pub reason: String,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SafetyFindingRollupRow {
    pub category: String,
    pub scanner: String,
    pub severity: String,
    pub phase: String,
    pub count: i64,
    pub blocked_count: i64,
    pub last_seen: DateTime<Utc>,
}

pub async fn list_warning_rollup(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<WarningRollupRow>, sqlx::Error> {
    sqlx::query_as!(
        WarningRollupRow,
        r#"SELECT policy AS "policy!", tool_name AS "tool_name!",
                  user_id AS "user_id!: UserId", COUNT(*) AS "count!",
                  MIN(created_at) AS "first_seen!", MAX(created_at) AS "last_seen!",
                  (ARRAY_AGG(reason ORDER BY created_at DESC))[1] AS "example_reason!"
           FROM governance_decisions
           WHERE decision = 'warn' AND created_at >= $1 AND created_at < $2
           GROUP BY policy, tool_name, user_id
           ORDER BY COUNT(*) DESC, MAX(created_at) DESC
           LIMIT $3"#,
        from,
        to,
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn list_recent_warnings(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<WarningDecisionRow>, sqlx::Error> {
    sqlx::query_as!(
        WarningDecisionRow,
        r#"SELECT id, policy, tool_name, user_id AS "user_id!: UserId", reason,
                  trace_id, created_at
           FROM governance_decisions
           WHERE decision = 'warn' AND created_at >= $1 AND created_at < $2
           ORDER BY created_at DESC
           LIMIT $3"#,
        from,
        to,
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn list_safety_finding_rollup(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<SafetyFindingRollupRow>, sqlx::Error> {
    sqlx::query_as!(
        SafetyFindingRollupRow,
        r#"SELECT category AS "category!", scanner AS "scanner!", severity AS "severity!",
                  phase AS "phase!", COUNT(*) AS "count!",
                  COUNT(*) FILTER (WHERE blocked) AS "blocked_count!",
                  MAX(created_at) AS "last_seen!"
           FROM ai_safety_findings
           WHERE created_at >= $1 AND created_at < $2
           GROUP BY category, scanner, severity, phase
           ORDER BY COUNT(*) DESC, MAX(created_at) DESC
           LIMIT $3"#,
        from,
        to,
        limit
    )
    .fetch_all(pool)
    .await
}

/// Which dimension of the warn rollup to collapse onto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningGroupBy {
    Policy,
    Tool,
    User,
}

impl WarningGroupBy {
    #[must_use]
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("tool") => Self::Tool,
            Some("user") => Self::User,
            _ => Self::Policy,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Policy => "policy",
            Self::Tool => "tool",
            Self::User => "user",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Policy => "Policy",
            Self::Tool => "Tool",
            Self::User => "User",
        }
    }
}

/// One re-aggregated group of the warn rollup.
#[derive(Debug, Clone)]
pub struct WarningGroup {
    pub group: String,
    pub warnings: i64,
    pub tools: i64,
    pub users: i64,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub example_reason: String,
}

// Why: distinct tool and user counts accumulate into sets rather than summing
// the SQL rollup's per-combination counts. Summing would report one tool used
// by two users as two tools, which is the number an operator would act on.
#[derive(Default)]
struct Accumulator {
    warnings: i64,
    tools: BTreeSet<String>,
    users: BTreeSet<String>,
    first_seen: Option<DateTime<Utc>>,
    last_seen: Option<DateTime<Utc>>,
    example_reason: String,
}

#[must_use]
pub fn group_warning_rollup(rows: &[WarningRollupRow], by: WarningGroupBy) -> Vec<WarningGroup> {
    let mut acc: BTreeMap<String, Accumulator> = BTreeMap::new();
    for row in rows {
        let key = match by {
            WarningGroupBy::Policy => row.policy.clone(),
            WarningGroupBy::Tool => row.tool_name.clone(),
            WarningGroupBy::User => row.user_id.as_str().to_owned(),
        };
        let entry = acc.entry(key).or_default();
        entry.warnings += row.count;
        entry.tools.insert(row.tool_name.clone());
        entry.users.insert(row.user_id.as_str().to_owned());
        if entry.first_seen.is_none_or(|seen| row.first_seen < seen) {
            entry.first_seen = Some(row.first_seen);
        }
        if entry.last_seen.is_none_or(|seen| seen < row.last_seen) {
            entry.last_seen = Some(row.last_seen);
            entry.example_reason.clone_from(&row.example_reason);
        }
    }

    let mut out: Vec<WarningGroup> = acc
        .into_iter()
        .map(|(group, a)| WarningGroup {
            group,
            warnings: a.warnings,
            tools: i64::try_from(a.tools.len()).unwrap_or(i64::MAX),
            users: i64::try_from(a.users.len()).unwrap_or(i64::MAX),
            first_seen: a.first_seen,
            last_seen: a.last_seen,
            example_reason: a.example_reason,
        })
        .collect();
    out.sort_by(|a, b| {
        b.warnings
            .cmp(&a.warnings)
            .then_with(|| a.group.cmp(&b.group))
    });
    out
}
