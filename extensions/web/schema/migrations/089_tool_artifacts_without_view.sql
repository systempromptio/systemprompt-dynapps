-- Re-issue of 087_tool_artifacts without its `CREATE VIEW tool_activity`:
-- that view selects from core's `tool_call_ledger`, a declarative view the
-- installer only creates after migrations, so 087 failed every upgrade from
-- a database that had never booted on core >= 0.55 ("relation
-- tool_call_ledger does not exist"). `tool_activity` is declarative
-- (schema/46_tool_artifacts.sql) and is created in the dependent phase;
-- the rest of 087 is repeated verbatim so a database that never ran it
-- still gains the columns, the functions and `analysis_reports`, and one
-- that did is unchanged (every statement is idempotent).

ALTER TABLE conversation_facts ADD COLUMN IF NOT EXISTS artifact_files BIGINT NOT NULL DEFAULT 0;
ALTER TABLE conversation_facts ADD COLUMN IF NOT EXISTS artifact_cards BIGINT NOT NULL DEFAULT 0;

-- The one artifact rule. Immutable so it can sit in a view and an index.
CREATE OR REPLACE FUNCTION artifact_kind(tool_name text, artifact_type text, has_ui boolean,
    is_structured boolean, payload_sha256 text, input text, is_builtin boolean)
RETURNS text LANGUAGE sql IMMUTABLE AS $$
SELECT CASE
    WHEN tool_name IN ('Edit', 'Write', 'MultiEdit', 'NotebookEdit', 'Read')
         AND input IS NOT NULL AND position('"file_path"' IN input) > 0 THEN 'file'
    WHEN COALESCE(is_builtin, false) THEN NULL
    WHEN COALESCE(has_ui, false) THEN 'ui'
    WHEN artifact_type IS NOT NULL AND artifact_type <> 'tool_result' THEN 'card'
    WHEN payload_sha256 IS NOT NULL AND COALESCE(is_structured, false) THEN 'body'
    ELSE NULL END
$$;

-- What a call was about, in one line: the file path, the command, the
-- search pattern or query, else the first key of the input object. A body
-- that is not a JSON object is clipped; a body that only looks like one
-- falls back to the clip rather than failing the row.
CREATE OR REPLACE FUNCTION tool_input_summary(input text)
RETURNS text LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE body jsonb;
BEGIN
    IF input IS NULL OR input = '' THEN RETURN NULL; END IF;
    IF left(ltrim(input), 1) <> '{' THEN RETURN left(input, 160); END IF;
    body := input::jsonb;
    RETURN COALESCE(
        NULLIF(body->>'file_path', ''), NULLIF(body->>'command', ''),
        NULLIF(body->>'pattern', ''), NULLIF(body->>'query', ''),
        NULLIF(body->>'path', ''), NULLIF(body->>'url', ''),
        NULLIF(body->>'skill', ''), NULLIF(body->>'name', ''),
        (SELECT k FROM jsonb_object_keys(body) k LIMIT 1));
EXCEPTION WHEN others THEN
    RETURN left(input, 160);
END $$;

CREATE TABLE IF NOT EXISTS analysis_reports (
    id TEXT PRIMARY KEY,
    scope_kind TEXT NOT NULL CHECK (scope_kind IN ('global', 'marketplace', 'skill', 'filter')),
    scope_id TEXT,
    scope_label TEXT,
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'generated', 'failed')),
    requested_by TEXT NOT NULL,
    provider TEXT,
    model TEXT,
    ai_request_id TEXT,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost_microdollars BIGINT,
    inputs JSONB NOT NULL DEFAULT '{}'::jsonb,
    findings JSONB,
    attempts INTEGER NOT NULL DEFAULT 0,
    lease_token TEXT,
    lease_until TIMESTAMPTZ,
    next_attempt TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    last_error TEXT,
    generated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS idx_analysis_reports_created ON analysis_reports(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_analysis_reports_scope ON analysis_reports(scope_kind, scope_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_analysis_reports_pending ON analysis_reports(next_attempt)
    WHERE status = 'pending';

-- Rebuilds the fact row for each context given (NULL = every context that
-- has a conversation request). Returns the number of rows written. The
-- request classification is `conversation_requests` (27_), so a turn here is
-- a turn everywhere; the group and project are the ones stamped on the latest
-- request, so a conversation stays filed where it was made.
CREATE OR REPLACE FUNCTION refresh_conversation_facts(context_ids TEXT[])
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE written BIGINT;
BEGIN
    WITH requests AS MATERIALIZED (
        SELECT r.*, ar.wire_protocol, ar.cache_read_tokens, ar.cache_creation_tokens,
               ar.reasoning_tokens, ar.is_streaming
        FROM conversation_requests r
        JOIN ai_requests ar ON ar.id = r.id
        WHERE context_ids IS NULL OR r.context_id = ANY(context_ids)
    ), agg AS (
        SELECT r.context_id,
            (ARRAY_AGG(r.user_id ORDER BY r.created_at DESC, r.id DESC))[1] AS user_id,
            (ARRAY_AGG(r.id ORDER BY r.created_at DESC, r.id DESC))[1] AS last_request_id,
            (ARRAY_AGG(r.session_id ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.session_id IS NOT NULL))[1] AS session_id,
            (ARRAY_AGG(r.client_session_id ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.client_session_id IS NOT NULL))[1] AS client_session_id,
            (ARRAY_AGG(r.client_kind ORDER BY r.created_at DESC, r.id DESC))[1] AS client_kind,
            (ARRAY_AGG(r.client_attestation ORDER BY r.created_at DESC, r.id DESC))[1] AS client_attestation,
            (ARRAY_AGG(r.wire_protocol ORDER BY r.created_at DESC, r.id DESC))[1] AS wire_protocol,
            (ARRAY_AGG(r.model ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.effective_kind = 'turn' AND r.model IS NOT NULL))[1] AS model,
            (ARRAY_AGG(r.provider ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.effective_kind = 'turn' AND r.provider IS NOT NULL))[1] AS provider,
            COUNT(*)::bigint AS request_count,
            COUNT(*) FILTER (WHERE r.effective_kind = 'turn')::bigint AS turn_count,
            COUNT(*) FILTER (WHERE r.effective_kind <> 'turn')::bigint AS side_call_count,
            COALESCE(SUM(r.cost_microdollars) FILTER (WHERE r.effective_kind <> 'turn'), 0)::bigint AS side_call_cost,
            COUNT(*) FILTER (WHERE r.status = 'failed')::bigint AS error_count,
            COUNT(*) FILTER (WHERE r.status = 'rejected')::bigint AS rejected_count,
            COUNT(*) FILTER (WHERE r.is_streaming)::bigint AS streaming_count,
            COALESCE(SUM(r.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(r.output_tokens), 0)::bigint AS output_tokens,
            COALESCE(SUM(r.cache_read_tokens), 0)::bigint AS cache_read_tokens,
            COALESCE(SUM(r.cache_creation_tokens), 0)::bigint AS cache_creation_tokens,
            COALESCE(SUM(r.reasoning_tokens), 0)::bigint AS reasoning_tokens,
            COALESCE(SUM(r.cost_microdollars), 0)::bigint AS cost_microdollars,
            percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.latency_ms IS NOT NULL AND r.effective_kind = 'turn') AS p50,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.latency_ms IS NOT NULL AND r.effective_kind = 'turn') AS p95,
            MAX(r.latency_ms) FILTER (WHERE r.effective_kind = 'turn') AS max_latency_ms,
            MIN(r.created_at) AS first_at,
            MAX(COALESCE(r.completed_at, r.created_at)) AS last_at
        FROM requests r
        GROUP BY r.context_id
    ), used AS (
        SELECT r.context_id,
            ARRAY(SELECT m.model FROM (
                SELECT model, MIN(created_at) AS first_use FROM requests q
                WHERE q.context_id = r.context_id AND q.model IS NOT NULL GROUP BY model) m
                ORDER BY m.first_use, m.model) AS models,
            ARRAY(SELECT DISTINCT q.provider FROM requests q
                WHERE q.context_id = r.context_id AND q.provider IS NOT NULL ORDER BY q.provider) AS providers
        FROM (SELECT DISTINCT context_id FROM requests) r
    ), intents AS (
        SELECT r.context_id, COUNT(*)::bigint AS tool_calls_intended
        FROM requests r JOIN ai_request_tool_calls t ON t.request_id = r.id
        GROUP BY r.context_id
    ), safety AS (
        SELECT r.context_id, COUNT(*)::bigint AS findings,
               COUNT(*) FILTER (WHERE f.blocked)::bigint AS blocked
        FROM requests r JOIN ai_safety_findings f ON f.ai_request_id = r.id
        GROUP BY r.context_id
    ), executions AS (
        SELECT a.context_id,
               COUNT(t.mcp_execution_id)::bigint AS executed,
               COUNT(t.mcp_execution_id) FILTER (WHERE t.execution_status IN ('failed', 'timeout'))::bigint AS failed,
               COUNT(t.mcp_execution_id) FILTER (WHERE t.artifact_kind IS NOT NULL)::bigint AS artifacts,
               COUNT(t.mcp_execution_id) FILTER (WHERE t.artifact_kind = 'file')::bigint AS artifact_files,
               COUNT(t.mcp_execution_id) FILTER (WHERE t.artifact_kind IN ('ui', 'card', 'body'))::bigint AS artifact_cards
        FROM agg a
        LEFT JOIN tool_activity t ON t.mcp_execution_id IS NOT NULL
            AND (t.execution_context_id = a.context_id
                 OR (a.client_session_id IS NOT NULL AND t.execution_trace_id = a.client_session_id))
        GROUP BY a.context_id
    ), governance AS (
        SELECT a.context_id,
               COUNT(*) FILTER (WHERE d.decision = 'allow')::bigint AS allow,
               COUNT(*) FILTER (WHERE d.decision = 'warn')::bigint AS warn,
               COUNT(*) FILTER (WHERE d.decision = 'deny')::bigint AS deny
        FROM agg a
        JOIN governance_decisions d ON d.context_id = a.context_id
            OR (a.client_session_id IS NOT NULL AND d.session_id = a.client_session_id)
        GROUP BY a.context_id
    ), hooks AS (
        SELECT a.context_id,
               COUNT(*)::bigint AS hook_events,
               COUNT(*) FILTER (WHERE e.event_type = 'UserPromptSubmit')::bigint AS prompts
        FROM agg a JOIN plugin_usage_events e ON e.session_id = a.client_session_id
        WHERE a.client_session_id IS NOT NULL
        GROUP BY a.context_id
    ), skills AS (
        SELECT a.context_id, COUNT(*)::bigint AS invocations,
               ARRAY(SELECT x.skill FROM (
                   SELECT skill, MIN(invoked_at) AS first_at FROM analysis_skill_events q
                   WHERE q.session_id = a.client_session_id AND q.skill IS NOT NULL GROUP BY skill) x
                   ORDER BY x.first_at, x.skill) AS skills
        FROM agg a JOIN analysis_skill_events s ON s.session_id = a.client_session_id AND s.skill IS NOT NULL
        WHERE a.client_session_id IS NOT NULL
        GROUP BY a.context_id, a.client_session_id
    ), upsert AS (
        INSERT INTO conversation_facts AS f (
            context_id, user_id, session_id, client_session_id, group_id, project_id,
            client_kind, client_attestation, wire_protocol, model, provider, models, providers,
            request_count, turn_count, side_call_count, side_call_cost_microdollars,
            error_count, rejected_count, streaming_count,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, reasoning_tokens,
            cost_microdollars, p50_latency_ms, p95_latency_ms, max_latency_ms,
            tool_calls_intended, tool_calls_executed, tool_calls_failed, artifact_count,
            artifact_files, artifact_cards,
            safety_findings, safety_blocked, gov_allow, gov_warn, gov_deny,
            prompt_count, hook_event_count, hook_status, skill_invocations, skills,
            first_at, last_at, duration_seconds, refreshed_at)
        SELECT a.context_id, a.user_id, a.session_id, a.client_session_id, rs.group_id, rs.project_id,
               COALESCE(a.client_kind, 'unknown'), COALESCE(a.client_attestation, 'unknown'),
               COALESCE(a.wire_protocol, 'unknown'), a.model, a.provider,
               COALESCE(u.models, '{}'), COALESCE(u.providers, '{}'),
               a.request_count, a.turn_count, a.side_call_count, a.side_call_cost,
               a.error_count, a.rejected_count, a.streaming_count,
               a.input_tokens, a.output_tokens, a.cache_read_tokens, a.cache_creation_tokens, a.reasoning_tokens,
               a.cost_microdollars, a.p50::int, a.p95::int, a.max_latency_ms,
               COALESCE(i.tool_calls_intended, 0), COALESCE(x.executed, 0), COALESCE(x.failed, 0), COALESCE(x.artifacts, 0),
               COALESCE(x.artifact_files, 0), COALESCE(x.artifact_cards, 0),
               COALESCE(s.findings, 0), COALESCE(s.blocked, 0),
               COALESCE(g.allow, 0), COALESCE(g.warn, 0), COALESCE(g.deny, 0),
               COALESCE(h.prompts, 0), COALESCE(h.hook_events, 0), ps.status,
               COALESCE(k.invocations, 0), COALESCE(k.skills, '{}'),
               a.first_at, GREATEST(a.last_at, c.updated_at),
               GREATEST(EXTRACT(EPOCH FROM GREATEST(a.last_at, c.updated_at) - a.first_at)::bigint, 0),
               clock_timestamp()
        FROM agg a
        LEFT JOIN used u ON u.context_id = a.context_id
        LEFT JOIN intents i ON i.context_id = a.context_id
        LEFT JOIN safety s ON s.context_id = a.context_id
        LEFT JOIN executions x ON x.context_id = a.context_id
        LEFT JOIN governance g ON g.context_id = a.context_id
        LEFT JOIN hooks h ON h.context_id = a.context_id
        LEFT JOIN skills k ON k.context_id = a.context_id
        LEFT JOIN ai_request_scopes rs ON rs.request_id = a.last_request_id
        LEFT JOIN user_contexts c ON c.context_id = a.context_id
        LEFT JOIN plugin_session_summaries ps ON ps.session_id = a.client_session_id
        ON CONFLICT (context_id) DO UPDATE SET
            user_id = EXCLUDED.user_id, session_id = EXCLUDED.session_id,
            client_session_id = EXCLUDED.client_session_id,
            group_id = EXCLUDED.group_id, project_id = EXCLUDED.project_id,
            client_kind = EXCLUDED.client_kind, client_attestation = EXCLUDED.client_attestation,
            wire_protocol = EXCLUDED.wire_protocol, model = EXCLUDED.model, provider = EXCLUDED.provider,
            models = EXCLUDED.models, providers = EXCLUDED.providers,
            request_count = EXCLUDED.request_count, turn_count = EXCLUDED.turn_count,
            side_call_count = EXCLUDED.side_call_count,
            side_call_cost_microdollars = EXCLUDED.side_call_cost_microdollars,
            error_count = EXCLUDED.error_count, rejected_count = EXCLUDED.rejected_count,
            streaming_count = EXCLUDED.streaming_count,
            input_tokens = EXCLUDED.input_tokens, output_tokens = EXCLUDED.output_tokens,
            cache_read_tokens = EXCLUDED.cache_read_tokens,
            cache_creation_tokens = EXCLUDED.cache_creation_tokens,
            reasoning_tokens = EXCLUDED.reasoning_tokens, cost_microdollars = EXCLUDED.cost_microdollars,
            p50_latency_ms = EXCLUDED.p50_latency_ms, p95_latency_ms = EXCLUDED.p95_latency_ms,
            max_latency_ms = EXCLUDED.max_latency_ms,
            tool_calls_intended = EXCLUDED.tool_calls_intended,
            tool_calls_executed = EXCLUDED.tool_calls_executed,
            tool_calls_failed = EXCLUDED.tool_calls_failed, artifact_count = EXCLUDED.artifact_count,
            artifact_files = EXCLUDED.artifact_files, artifact_cards = EXCLUDED.artifact_cards,
            safety_findings = EXCLUDED.safety_findings, safety_blocked = EXCLUDED.safety_blocked,
            gov_allow = EXCLUDED.gov_allow, gov_warn = EXCLUDED.gov_warn, gov_deny = EXCLUDED.gov_deny,
            prompt_count = EXCLUDED.prompt_count, hook_event_count = EXCLUDED.hook_event_count,
            hook_status = EXCLUDED.hook_status, skill_invocations = EXCLUDED.skill_invocations,
            skills = EXCLUDED.skills, first_at = EXCLUDED.first_at, last_at = EXCLUDED.last_at,
            duration_seconds = EXCLUDED.duration_seconds, refreshed_at = EXCLUDED.refreshed_at
        RETURNING 1)
    SELECT COUNT(*) INTO written FROM upsert;
    IF context_ids IS NOT NULL THEN
        DELETE FROM conversation_facts f WHERE f.context_id = ANY(context_ids)
            AND NOT EXISTS (SELECT 1 FROM conversation_requests r WHERE r.context_id = f.context_id);
    END IF;
    RETURN COALESCE(written, 0);
END $$;

