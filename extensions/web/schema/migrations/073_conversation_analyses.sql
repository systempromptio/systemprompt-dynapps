-- Conversation purpose classification. Twin of
-- schema/37_conversation_analyses.sql, plus: `conversation_requests` and
-- `conversation_metrics_for` from schema 27 re-declared with scheduled-job
-- traffic (`actor_kind = 'job'`) excluded, so the classifier's own judge
-- calls never surface as a conversation. Column lists and return types are
-- unchanged, so the dependent `conversation_rollups` view stays valid.

CREATE OR REPLACE VIEW conversation_requests AS
WITH threads AS (
    SELECT context_id, gateway_conversation_id, COUNT(*)::bigint AS thread_requests
    FROM ai_requests
    GROUP BY context_id, gateway_conversation_id
)
SELECT r.id, r.user_id, r.session_id, r.client_session_id, r.context_id,
       r.gateway_conversation_id, r.trace_id, r.provider, r.model, r.client_kind,
       r.client_attestation, r.status,
       r.max_tokens, r.input_tokens, r.output_tokens, r.cost_microdollars,
       r.latency_ms, r.created_at, r.completed_at,
       conversation_request_kind(r.request_kind, p.offered_tools IS NOT NULL, t.thread_requests, r.client_session_id IS NOT NULL) AS effective_kind
FROM ai_requests r
LEFT JOIN ai_request_payloads p ON p.ai_request_id = r.id
JOIN threads t ON t.context_id = r.context_id
   AND t.gateway_conversation_id IS NOT DISTINCT FROM r.gateway_conversation_id
WHERE r.context_id <> '00000000-0000-0000-0000-4c4547414359'
  AND r.actor_kind <> 'job';

-- Numeric summaries have no prompt/message dependency. An explicit context set
-- lets detail and scoped list callers aggregate complete conversations without
-- walking unrelated history. NULL means all contexts; an empty array means none.
CREATE OR REPLACE FUNCTION conversation_metrics_for(context_ids text[])
RETURNS TABLE (
    context_id text, user_id text, display_name text, session_id text,
    client_session_id text, group_name text, project_name text, model text,
    client_kind text, client_attestation text, turn_count bigint, side_call_count bigint,
    side_call_cost_microdollars bigint,
    tool_call_count bigint, error_count bigint, total_input_tokens bigint,
    total_output_tokens bigint, total_cost_microdollars bigint,
    first_at timestamptz, last_at timestamptz, status text
) LANGUAGE sql STABLE AS $function$
WITH scoped AS (
    SELECT ar.id, ar.context_id, ar.gateway_conversation_id, ar.user_id,
           ar.session_id, ar.client_session_id, ar.model, ar.client_kind, ar.client_attestation,
           ar.request_kind,
           ar.status, ar.input_tokens, ar.output_tokens, ar.cost_microdollars, ar.created_at,
           COUNT(*) OVER (PARTITION BY ar.context_id, ar.gateway_conversation_id) AS thread_size
    FROM ai_requests ar
    WHERE ar.context_id <> '00000000-0000-0000-0000-4c4547414359'
      AND ar.actor_kind <> 'job'
      AND (context_ids IS NULL OR ar.context_id IN (SELECT unnest(context_ids)))
), requests AS MATERIALIZED (
    SELECT r.*, conversation_request_kind(r.request_kind, p.offered_tools IS NOT NULL, r.thread_size, r.client_session_id IS NOT NULL) AS effective_kind
    FROM scoped r LEFT JOIN ai_request_payloads p ON p.ai_request_id = r.id
), agg AS (
    SELECT r.context_id,
        COUNT(*) FILTER (WHERE effective_kind = 'turn')::bigint AS turn_count,
        COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint AS side_call_count,
        COALESCE(SUM(cost_microdollars) FILTER (WHERE effective_kind <> 'turn'), 0)::bigint AS side_call_cost_microdollars,
        COUNT(*) FILTER (WHERE status = 'failed')::bigint AS error_count,
        COALESCE(SUM(input_tokens), 0)::bigint AS total_input_tokens,
        COALESCE(SUM(output_tokens), 0)::bigint AS total_output_tokens,
        COALESCE(SUM(cost_microdollars), 0)::bigint AS total_cost_microdollars,
        MIN(created_at) AS first_at, MAX(created_at) AS last_request_at,
        (ARRAY_AGG(user_id ORDER BY created_at DESC, id DESC))[1] AS user_id,
        (ARRAY_AGG(session_id ORDER BY created_at DESC, id DESC)
            FILTER (WHERE session_id IS NOT NULL))[1] AS session_id,
        (ARRAY_AGG(client_session_id ORDER BY created_at DESC, id DESC)
            FILTER (WHERE client_session_id IS NOT NULL))[1] AS client_session_id,
        (ARRAY_AGG(model ORDER BY created_at DESC, id DESC)
            FILTER (WHERE effective_kind = 'turn' AND model IS NOT NULL))[1] AS model,
        (ARRAY_AGG(client_kind ORDER BY created_at DESC, id DESC))[1] AS client_kind,
        (ARRAY_AGG(client_attestation ORDER BY created_at DESC, id DESC))[1] AS client_attestation
    FROM requests r GROUP BY r.context_id
), tools AS (
    SELECT r.context_id, COUNT(*)::bigint AS tool_call_count
    FROM requests r JOIN ai_request_tool_calls t ON t.request_id = r.id
    WHERE r.effective_kind = 'turn' GROUP BY r.context_id
)
SELECT a.context_id, a.user_id, u.display_name, a.session_id, a.client_session_id,
       g.name, p.name, a.model, a.client_kind, a.client_attestation, a.turn_count, a.side_call_count,
       a.side_call_cost_microdollars, COALESCE(t.tool_call_count, 0), a.error_count,
       a.total_input_tokens, a.total_output_tokens, a.total_cost_microdollars,
       a.first_at, GREATEST(a.last_request_at, c.updated_at), s.status
FROM agg a
LEFT JOIN tools t ON t.context_id = a.context_id
LEFT JOIN user_contexts c ON c.context_id = a.context_id
LEFT JOIN plugin_session_summaries s ON s.session_id = a.client_session_id
LEFT JOIN users u ON u.id = a.user_id
LEFT JOIN user_scope_defaults d ON d.user_id = a.user_id
LEFT JOIN groups g ON g.id = d.primary_group_id
LEFT JOIN projects p ON p.id = d.primary_project_id
$function$;

CREATE TABLE IF NOT EXISTS conversation_analyses (
    context_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'classified', 'failed')),
    category TEXT CHECK (category IN ('development', 'business-analysis', 'operations',
        'admin-config', 'writing-comms', 'research-learning', 'other')),
    summary TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    skills_used TEXT[] NOT NULL DEFAULT '{}',
    outcome TEXT CHECK (outcome IN ('achieved', 'partial', 'abandoned', 'unclear')),
    confidence REAL CHECK (confidence BETWEEN 0 AND 1),
    provider TEXT,
    model TEXT,
    ai_request_id TEXT,
    source_request_count BIGINT NOT NULL DEFAULT 0,
    source_last_at TIMESTAMPTZ,
    classified_at TIMESTAMPTZ,
    attempts INT NOT NULL DEFAULT 0,
    next_attempt TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    lease_token TEXT,
    lease_until TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_pending
    ON conversation_analyses(next_attempt) WHERE status <> 'classified';
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_category
    ON conversation_analyses(category, classified_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_user
    ON conversation_analyses(user_id, classified_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_skills
    ON conversation_analyses USING gin(skills_used);

-- Skills the harness hooks reported per client session, so a conversation
-- shows what it invoked even before the judge has seen it.
CREATE OR REPLACE VIEW conversation_skill_uses AS
SELECT e.session_id AS client_session_id, e.skill,
       COUNT(*)::bigint AS invocations, MIN(e.invoked_at) AS first_at
FROM analysis_skill_events e
WHERE e.skill IS NOT NULL
GROUP BY e.session_id, e.skill;

-- One row per conversation with its metrics, classification and hooked
-- skills. For single-row reads and ad-hoc SQL only: it aggregates every
-- context, so a page filters `conversation_analyses` first and passes the ids
-- to conversation_metrics_for.
CREATE OR REPLACE VIEW conversation_analysis_rows AS
SELECT r.*, a.status AS analysis_status, a.category, a.summary, a.tags, a.outcome, a.confidence,
       a.skills_used, a.classified_at, a.model AS judge_model,
       COALESCE((SELECT array_agg(s.skill ORDER BY s.first_at) FROM conversation_skill_uses s
                 WHERE s.client_session_id = r.client_session_id), '{}'::text[]) AS skills_hooked
FROM conversation_metrics_for(NULL) r
LEFT JOIN conversation_analyses a ON a.context_id = r.context_id;
