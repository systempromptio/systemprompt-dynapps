//! The other planes of one conversation, read beside its fact row.
//!
//! For `/admin/analysis/conversations/{id}`: the turn ledger (`turns`), tool
//! calls (`tool_calls`), and here the governance decisions, the skills invoked
//! with the marketplace version served at the time, and the safety scanner's
//! findings.

mod tool_calls;
mod turns;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, MarketplaceId, PluginId};

pub use tool_calls::{ConversationToolCallRow, list_conversation_tool_calls};
pub use turns::{ConversationTurnRow, list_conversation_turns};

/// One governance decision on a tool call of the conversation.
#[derive(Debug, Clone)]
pub struct ConversationDecisionRow {
    pub tool_name: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub plugin_id: Option<PluginId>,
    pub created_at: DateTime<Utc>,
    // JSON: the chain's per-rule evaluations, shaped by each policy stage.
    pub evaluated_rules: Option<serde_json::Value>,
    pub tool_use_id: Option<String>,
    pub trace_id: Option<String>,
}

pub async fn list_conversation_decisions(
    pool: &PgPool,
    context_id: &ContextId,
    client_session_id: Option<&str>,
) -> Result<Vec<ConversationDecisionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT d.tool_name, d.decision AS "decision!", d.policy, d.reason,
                  d.plugin_id AS "plugin_id?: PluginId",
                  d.created_at AS "created_at!",
                  d.evaluated_rules, d.tool_use_id, d.trace_id
           FROM governance_decisions d
           WHERE d.context_id = $1 OR ($2::text IS NOT NULL AND d.session_id = $2)
           ORDER BY d.created_at
           LIMIT 500"#,
        context_id.as_str(),
        client_session_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationDecisionRow {
            tool_name: r.tool_name,
            decision: r.decision,
            policy: r.policy,
            reason: r.reason,
            plugin_id: r.plugin_id,
            created_at: r.created_at,
            evaluated_rules: r.evaluated_rules,
            tool_use_id: r.tool_use_id,
            trace_id: r.trace_id,
        })
        .collect())
}

/// One skill the conversation invoked and the version served at the time.
#[derive(Debug, Clone)]
pub struct ConversationSkillRow {
    pub plugin_id: Option<PluginId>,
    pub skill: String,
    pub marketplace_id: Option<MarketplaceId>,
    pub marketplace_hash: Option<String>,
    pub invocations: i64,
    pub first_invoked_at: DateTime<Utc>,
}

pub async fn list_conversation_skills(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ConversationSkillRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT s.plugin_id AS "plugin_id?: PluginId", s.skill,
                  s.marketplace_id AS "marketplace_id?: MarketplaceId", s.marketplace_hash,
                  s.invocations, s.first_invoked_at
           FROM conversation_skill_facts s
           WHERE s.context_id = $1
           ORDER BY s.first_invoked_at"#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationSkillRow {
            plugin_id: r.plugin_id,
            skill: r.skill,
            marketplace_id: r.marketplace_id,
            marketplace_hash: r.marketplace_hash,
            invocations: r.invocations,
            first_invoked_at: r.first_invoked_at,
        })
        .collect())
}

/// One safety scanner finding on a request of the conversation.
#[derive(Debug, Clone)]
pub struct ConversationSafetyRow {
    pub phase: String,
    pub category: String,
    pub severity: String,
    pub scanner: String,
    pub blocked: bool,
    pub created_at: DateTime<Utc>,
    pub request_id: String,
    pub excerpt: Option<String>,
}

pub async fn list_conversation_safety_findings(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ConversationSafetyRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT s.phase AS "phase!", s.category AS "category!", s.severity AS "severity!",
                  s.scanner, s.blocked AS "blocked!", s.created_at AS "created_at!",
                  s.ai_request_id AS "request_id!", s.excerpt
           FROM ai_safety_findings s
           JOIN conversation_requests cr ON cr.id = s.ai_request_id
           WHERE cr.context_id = $1
           ORDER BY s.created_at
           LIMIT 200"#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationSafetyRow {
            phase: r.phase,
            category: r.category,
            severity: r.severity,
            scanner: r.scanner,
            blocked: r.blocked,
            created_at: r.created_at,
            request_id: r.request_id,
            excerpt: r.excerpt,
        })
        .collect())
}
