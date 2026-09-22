-- Attribute session-scoped facts to one context instead of every context of
-- the session.
--
-- `refresh_conversation_facts` joined governance decisions, hook events,
-- skill invocations and hook-observed tool executions on
-- `session_id = client_session_id`. One Claude Code session opens many
-- gateway contexts (sub-agents, compaction, /clear — up to 16 in production),
-- so each of those rows was counted once per context: 14,923 hook events
-- recorded against 3,348 real ones, and the same multiple on `gov_*` and
-- `skill_invocations`. Each event now belongs to the single context whose
-- request window contains it, falling back to the nearest window of the same
-- session, so a session's events are partitioned across its contexts and the
-- per-context sums add up to the session's true total. Contexts outside the
-- refresh set still own their share, which is what keeps a partial refresh
-- from re-attributing another context's events to this one.
--
-- The function body is the declarative one in schema/45_conversation_facts.sql.

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
    ), sessions AS (
        SELECT DISTINCT r.client_session_id
        FROM requests r WHERE r.client_session_id IS NOT NULL
    ), windows AS (
        SELECT r.context_id, r.client_session_id,
               MIN(r.created_at) AS opened_at,
               MAX(COALESCE(r.completed_at, r.created_at)) AS closed_at
        FROM conversation_requests r
        WHERE r.client_session_id IN (SELECT s.client_session_id FROM sessions s)
        GROUP BY r.context_id, r.client_session_id
    ), executions AS (
        SELECT o.context_id,
               COUNT(*)::bigint AS executed,
               COUNT(*) FILTER (WHERE o.execution_status IN ('failed', 'timeout'))::bigint AS failed,
               COUNT(*) FILTER (WHERE o.artifact_kind IS NOT NULL)::bigint AS artifacts,
               COUNT(*) FILTER (WHERE o.artifact_kind = 'file')::bigint AS artifact_files,
               COUNT(*) FILTER (WHERE o.artifact_kind IN ('ui', 'card', 'body'))::bigint AS artifact_cards
        FROM (
            SELECT CASE WHEN EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                        THEN t.execution_context_id ELSE w.context_id END AS context_id,
                   t.execution_status, t.artifact_kind
            FROM tool_activity t
            LEFT JOIN LATERAL (
                SELECT x.context_id FROM windows x
                WHERE NOT EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                  AND x.client_session_id = t.execution_trace_id
                ORDER BY (COALESCE(t.executed_at, t.occurred_at) BETWEEN x.opened_at AND x.closed_at) DESC,
                         abs(EXTRACT(EPOCH FROM COALESCE(t.executed_at, t.occurred_at) - x.opened_at)), x.opened_at, x.context_id
                LIMIT 1) w ON TRUE
            WHERE t.mcp_execution_id IS NOT NULL
              AND (EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                   OR w.context_id IS NOT NULL)
        ) o
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = o.context_id)
        GROUP BY o.context_id
    ), governance AS (
        SELECT o.context_id,
               COUNT(*) FILTER (WHERE o.decision = 'allow')::bigint AS allow,
               COUNT(*) FILTER (WHERE o.decision = 'warn')::bigint AS warn,
               COUNT(*) FILTER (WHERE o.decision = 'deny')::bigint AS deny
        FROM (
            SELECT CASE WHEN EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
                        THEN g.context_id ELSE w.context_id END AS context_id, g.decision
            FROM governance_decisions g
            LEFT JOIN LATERAL (
                SELECT x.context_id FROM windows x
                WHERE NOT EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
                  AND x.client_session_id = g.session_id
                ORDER BY (g.created_at BETWEEN x.opened_at AND x.closed_at) DESC,
                         abs(EXTRACT(EPOCH FROM g.created_at - x.opened_at)), x.opened_at, x.context_id
                LIMIT 1) w ON TRUE
            WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
               OR w.context_id IS NOT NULL
        ) o
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = o.context_id)
        GROUP BY o.context_id
    ), hook_events AS (
        SELECT w.context_id, e.event_type, e.id
        FROM plugin_usage_events e
        JOIN LATERAL (
            SELECT x.context_id FROM windows x
            WHERE x.client_session_id = e.session_id
            ORDER BY (e.created_at BETWEEN x.opened_at AND x.closed_at) DESC,
                     abs(EXTRACT(EPOCH FROM e.created_at - x.opened_at)), x.opened_at, x.context_id
            LIMIT 1) w ON TRUE
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = w.context_id)
    ), hooks AS (
        SELECT h.context_id,
               COUNT(*)::bigint AS hook_events,
               COUNT(*) FILTER (WHERE h.event_type = 'UserPromptSubmit')::bigint AS prompts
        FROM hook_events h
        GROUP BY h.context_id
    ), skill_events AS (
        SELECT w.context_id, s.skill, s.invoked_at
        FROM analysis_skill_events s
        JOIN LATERAL (
            SELECT x.context_id FROM windows x
            WHERE x.client_session_id = s.session_id
            ORDER BY (s.invoked_at BETWEEN x.opened_at AND x.closed_at) DESC,
                     abs(EXTRACT(EPOCH FROM s.invoked_at - x.opened_at)), x.opened_at, x.context_id
            LIMIT 1) w ON TRUE
        WHERE s.skill IS NOT NULL AND EXISTS (SELECT 1 FROM agg a WHERE a.context_id = w.context_id)
    ), skills AS (
        SELECT k.context_id, COUNT(*)::bigint AS invocations,
               ARRAY(SELECT y.skill FROM (
                   SELECT q.skill, MIN(q.invoked_at) AS first_at FROM skill_events q
                   WHERE q.context_id = k.context_id GROUP BY q.skill) y
                   ORDER BY y.first_at, y.skill) AS skills
        FROM skill_events k
        GROUP BY k.context_id
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

SELECT refresh_conversation_facts(NULL);
