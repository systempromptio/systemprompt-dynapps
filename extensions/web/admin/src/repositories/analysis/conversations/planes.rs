//! The other planes of one conversation, read beside its fact row.
//!
//! For `/admin/analysis/conversations/{id}`: the turn ledger from the request
//! log, tool calls from the ledger (intent → execution → artifact),
//! governance decisions, the skills invoked with the marketplace version
//! served at the time, and the safety scanner's findings.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ArtifactId, ContextId, MarketplaceId, PluginId};

/// One gateway request of the conversation, in order.
#[derive(Debug, Clone)]
pub struct ConversationTurnRow {
    pub request_id: String,
    pub created_at: DateTime<Utc>,
    pub effective_kind: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub requested_model: Option<String>,
    pub route_match: Option<String>,
    pub status: String,
    pub finish_reason: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_tokens: i64,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub upstream_latency_ms: Option<i32>,
    pub is_streaming: bool,
    pub max_tokens: Option<i32>,
    pub tool_calls: i64,
    pub tool_names: Vec<String>,
    pub safety_findings: i64,
    pub safety_blocked: i64,
    pub error_message: Option<String>,
}

pub async fn list_conversation_turns(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ConversationTurnRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT cr.id AS "request_id!", cr.created_at AS "created_at!",
                  cr.effective_kind AS "effective_kind!", cr.provider, cr.model, ar.requested_model,
                  ar.route_match, cr.status AS "status!", ar.finish_reason,
                  COALESCE(cr.input_tokens, 0)::bigint AS "input_tokens!",
                  COALESCE(cr.output_tokens, 0)::bigint AS "output_tokens!",
                  COALESCE(ar.cache_read_tokens, 0)::bigint AS "cache_read_tokens!",
                  COALESCE(ar.cache_creation_tokens, 0)::bigint AS "cache_creation_tokens!",
                  COALESCE(ar.reasoning_tokens, 0)::bigint AS "reasoning_tokens!",
                  cr.cost_microdollars AS "cost_microdollars!", cr.latency_ms, ar.upstream_latency_ms,
                  COALESCE(ar.is_streaming, false) AS "is_streaming!", cr.max_tokens,
                  (SELECT COUNT(*)::bigint FROM ai_request_tool_calls t WHERE t.request_id = cr.id) AS "tool_calls!",
                  ARRAY(SELECT t.tool_name FROM ai_request_tool_calls t WHERE t.request_id = cr.id
                        ORDER BY t.sequence_number) AS "tool_names!",
                  (SELECT COUNT(*)::bigint FROM ai_safety_findings s WHERE s.ai_request_id = cr.id) AS "safety_findings!",
                  (SELECT COUNT(*)::bigint FROM ai_safety_findings s WHERE s.ai_request_id = cr.id AND s.blocked) AS "safety_blocked!",
                  ar.error_message
           FROM conversation_requests cr
           JOIN ai_requests ar ON ar.id = cr.id
           WHERE cr.context_id = $1
           ORDER BY cr.created_at, cr.id
           LIMIT 500"#,
        context_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationTurnRow {
            request_id: r.request_id,
            created_at: r.created_at,
            effective_kind: r.effective_kind,
            provider: r.provider,
            model: r.model,
            requested_model: r.requested_model,
            route_match: r.route_match,
            status: r.status,
            finish_reason: r.finish_reason,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_read_tokens: r.cache_read_tokens,
            cache_creation_tokens: r.cache_creation_tokens,
            reasoning_tokens: r.reasoning_tokens,
            cost_microdollars: r.cost_microdollars,
            latency_ms: r.latency_ms,
            upstream_latency_ms: r.upstream_latency_ms,
            is_streaming: r.is_streaming,
            max_tokens: r.max_tokens,
            tool_calls: r.tool_calls,
            tool_names: r.tool_names,
            safety_findings: r.safety_findings,
            safety_blocked: r.safety_blocked,
            error_message: r.error_message,
        })
        .collect())
}

/// One tool call from the ledger — intent, execution and artifact folded.
#[derive(Debug, Clone)]
pub struct ConversationToolCallRow {
    pub tool_name: Option<String>,
    pub server_name: Option<String>,
    pub state: String,
    pub source: Option<String>,
    pub execution_status: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub artifact_id: Option<ArtifactId>,
    pub artifact_type: Option<String>,
    pub artifact_title: Option<String>,
    pub is_error: Option<bool>,
    pub error_message: Option<String>,
    pub request_id: Option<String>,
}

pub async fn list_conversation_tool_calls(
    pool: &PgPool,
    context_id: &ContextId,
    client_session_id: Option<&str>,
) -> Result<Vec<ConversationToolCallRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT l.tool_name, l.server_name, l.state AS "state!", l.source, l.execution_status,
                  l.execution_time_ms::bigint AS execution_time_ms, l.occurred_at, l.artifact_id AS "artifact_id?: ArtifactId",
                  l.artifact_type, l.artifact_title, l.is_error, l.error_message, l.request_id
           FROM tool_call_ledger l
           WHERE l.context_id = $1
              OR ($2::text IS NOT NULL AND (l.trace_id = $2 OR l.session_id = $2))
           ORDER BY l.occurred_at NULLS LAST
           LIMIT 500"#,
        context_id.as_str(),
        client_session_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationToolCallRow {
            tool_name: r.tool_name,
            server_name: r.server_name,
            state: r.state,
            source: r.source,
            execution_status: r.execution_status,
            execution_time_ms: r.execution_time_ms,
            occurred_at: r.occurred_at,
            artifact_id: r.artifact_id,
            artifact_type: r.artifact_type,
            artifact_title: r.artifact_title,
            is_error: r.is_error,
            error_message: r.error_message,
            request_id: r.request_id,
        })
        .collect())
}

/// One governance decision on a tool call of the conversation.
#[derive(Debug, Clone)]
pub struct ConversationDecisionRow {
    pub tool_name: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub plugin_id: Option<PluginId>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_conversation_decisions(
    pool: &PgPool,
    context_id: &ContextId,
    client_session_id: Option<&str>,
) -> Result<Vec<ConversationDecisionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT d.tool_name, d.decision AS "decision!", d.policy, d.reason,
                  d.plugin_id AS "plugin_id?: PluginId",
                  d.created_at AS "created_at!"
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
        r#"SELECT v.plugin_id AS "plugin_id?: PluginId", v.skill AS "skill!",
                  v.marketplace_id AS "marketplace_id?: MarketplaceId", v.marketplace_hash,
                  v.invocations AS "invocations!", v.first_invoked_at AS "first_invoked_at!"
           FROM conversation_skill_versions v
           WHERE v.context_id = $1
           ORDER BY v.first_invoked_at"#,
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
}

pub async fn list_conversation_safety_findings(
    pool: &PgPool,
    context_id: &ContextId,
) -> Result<Vec<ConversationSafetyRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT s.phase AS "phase!", s.category AS "category!", s.severity AS "severity!",
                  s.scanner, s.blocked AS "blocked!", s.created_at AS "created_at!",
                  s.ai_request_id AS "request_id!"
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
        })
        .collect())
}
