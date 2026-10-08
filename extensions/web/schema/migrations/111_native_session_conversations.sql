-- Group retained and future native-session requests without altering audit rows.
-- @cost: rows=4000 measured=4.120s triggers=live
-- PostgreSQL 18 isolated release-0.62.0 upgrade: 10,000 requests, 1,000
-- native sessions; fixture tests/fixtures/schema/native-session-cost.sql.
-- Wrote 1,000 session mappings, 1,000 aliases, deleted 1,000 old facts and
-- rebuilt 1,000 canonical facts. Slowest statement: refresh, 4119.325 ms.
-- Native sessions group request threads without rewriting the audit ledger.
CREATE TABLE IF NOT EXISTS conversation_session_contexts (
    user_id text NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_session_id text NOT NULL,
    context_id text NOT NULL DEFAULT gen_random_uuid()::text UNIQUE,
    PRIMARY KEY (user_id, client_session_id)
);
CREATE TABLE IF NOT EXISTS conversation_context_aliases (
    original_context_id text NOT NULL,
    context_id text NOT NULL REFERENCES conversation_session_contexts(context_id) ON DELETE CASCADE,
    PRIMARY KEY (original_context_id, context_id)
);

CREATE OR REPLACE FUNCTION register_conversation_session()
RETURNS trigger LANGUAGE plpgsql AS $function$
DECLARE canonical text;
BEGIN
    IF NEW.client_session_id IS NULL OR NEW.user_id IS NULL
       OR NEW.actor_kind = 'job'
       OR NEW.context_id = '00000000-0000-0000-0000-4c4547414359' THEN
        RETURN NEW;
    END IF;
    INSERT INTO conversation_session_contexts(user_id, client_session_id)
    VALUES (NEW.user_id, NEW.client_session_id) ON CONFLICT DO NOTHING;
    SELECT context_id INTO canonical FROM conversation_session_contexts
    WHERE user_id = NEW.user_id AND client_session_id = NEW.client_session_id;
    INSERT INTO conversation_context_aliases(original_context_id, context_id)
    VALUES (NEW.context_id, canonical) ON CONFLICT DO NOTHING;
    RETURN NEW;
END
$function$;
DROP TRIGGER IF EXISTS ai_requests_conversation_session ON ai_requests;
CREATE TRIGGER ai_requests_conversation_session
AFTER INSERT OR UPDATE OF user_id, client_session_id, context_id ON ai_requests
FOR EACH ROW EXECUTE FUNCTION register_conversation_session();

INSERT INTO conversation_session_contexts(user_id, client_session_id)
SELECT DISTINCT user_id, client_session_id FROM ai_requests
WHERE client_session_id IS NOT NULL AND user_id IS NOT NULL AND actor_kind <> 'job'
  AND context_id <> '00000000-0000-0000-0000-4c4547414359'
ON CONFLICT DO NOTHING;
INSERT INTO conversation_context_aliases(original_context_id, context_id)
SELECT DISTINCT r.context_id, s.context_id FROM ai_requests r
JOIN conversation_session_contexts s
  ON s.user_id = r.user_id AND s.client_session_id = r.client_session_id
WHERE r.actor_kind <> 'job' AND r.context_id <> '00000000-0000-0000-0000-4c4547414359'
ON CONFLICT DO NOTHING;

-- An ambiguous legacy link never chooses between separate native sessions.
CREATE OR REPLACE FUNCTION canonical_conversation_context(context_key text)
RETURNS text LANGUAGE sql STABLE AS $function$
SELECT COALESCE((SELECT MIN(context_id) FROM conversation_context_aliases
                WHERE original_context_id = context_key HAVING COUNT(*) = 1), context_key)
$function$;

-- Shared classification: request labels take precedence over thread shape.
--
-- A side call is a probe, an explicitly labelled utility call, or — only
-- inside a harness-bound context (one keyed on a client session, so the
-- harness can make title-generation calls beside the conversation) — a
-- single-request thread that offered no tools. A hash-derived context has no
-- harness behind it: every request there is a turn, whatever its shape, so an
-- SDK or OpenCode thread of one prompt is a conversation and not hidden.
CREATE OR REPLACE FUNCTION conversation_request_kind(kind text, has_tools boolean, thread_size bigint, harness_bound boolean)
RETURNS text LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $function$
SELECT CASE WHEN kind = 'probe' THEN 'probe' WHEN kind = 'utility' THEN 'utility'
            WHEN harness_bound AND NOT has_tools AND thread_size = 1 THEN 'utility' ELSE 'turn' END
$function$;

-- One row per gateway request, classified for the conversation pages.
--
-- Rule: see conversation_request_kind above. `effective_kind` is one of
-- 'turn', 'probe', 'utility'.
--
-- A thread is one `gateway_conversation_id` inside a context (sub-agents and
-- post-compaction histories get their own). The legacy sentinel context that
-- pools every context-less request is excluded here so no page has to, and so
-- is every request a scheduled job made (`actor_kind = 'job'`): the
-- conversation classifier's own judge calls are gateway requests under the
-- job owner, and a job is never a conversation.
CREATE OR REPLACE VIEW conversation_requests AS
WITH threads AS (
    SELECT COALESCE(s.context_id, r.context_id) AS context_id,
           r.gateway_conversation_id, COUNT(*)::bigint AS thread_requests
    FROM ai_requests r LEFT JOIN conversation_session_contexts s
      ON s.user_id = r.user_id AND s.client_session_id = r.client_session_id
    GROUP BY COALESCE(s.context_id, r.context_id), r.gateway_conversation_id
)
SELECT r.id, r.user_id, r.session_id, r.client_session_id, COALESCE(s.context_id, r.context_id)::varchar(255) AS context_id,
       r.gateway_conversation_id, r.trace_id, r.provider, r.model, r.client_kind,
       r.client_attestation, r.status,
       r.max_tokens, r.input_tokens, r.output_tokens, r.cost_microdollars,
       r.latency_ms, r.created_at, r.completed_at,
       conversation_request_kind(r.request_kind, p.offered_tools_sha256 IS NOT NULL, t.thread_requests, r.client_session_id IS NOT NULL) AS effective_kind
FROM ai_requests r
LEFT JOIN conversation_session_contexts s
  ON s.user_id = r.user_id AND s.client_session_id = r.client_session_id
LEFT JOIN ai_request_payloads p ON p.ai_request_id = r.id
JOIN threads t ON t.context_id = COALESCE(s.context_id, r.context_id)
   AND t.gateway_conversation_id IS NOT DISTINCT FROM r.gateway_conversation_id
WHERE r.context_id <> '00000000-0000-0000-0000-4c4547414359'
  AND r.actor_kind <> 'job';

-- Numeric summaries have no prompt/message dependency. An explicit context set
-- lets detail and scoped list callers aggregate complete conversations without
-- walking unrelated history. NULL means all contexts; an empty array means none.
-- The group and project are the ones stamped on the latest request
-- (ai_request_scopes, 40_), not the person's current primaries, so a
-- conversation stays filed where it was made.
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
WITH requests AS MATERIALIZED (
    SELECT r.* FROM conversation_requests r
    WHERE context_ids IS NULL OR r.context_id IN (
        SELECT canonical_conversation_context(unnest(context_ids)))
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
        (ARRAY_AGG(id ORDER BY created_at DESC, id DESC))[1] AS last_request_id,
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
LEFT JOIN ai_request_scopes rs ON rs.request_id = a.last_request_id
LEFT JOIN groups g ON g.id = rs.group_id
LEFT JOIN projects p ON p.id = rs.project_id
$function$;

-- Parse only the first matching message, never every prompt in history.
CREATE OR REPLACE FUNCTION conversation_opening_prompt(context_key text, max_characters integer)
RETURNS text LANGUAGE sql STABLE AS $function$
SELECT LEFT(NULLIF(BTRIM(regexp_replace(regexp_replace(regexp_replace(
    split_part(m.content, '=== ASSISTANT ANSWER ===', 1),
    '=== USER PROMPT ===', '', 'g'), '<system-reminder>.*?</system-reminder>', '', 'g'),
    '\s+', ' ', 'g')), ''), max_characters)
FROM (
    SELECT m.content FROM conversation_requests cr
    JOIN ai_request_messages m ON m.request_id = cr.id
    WHERE cr.context_id = context_key AND cr.effective_kind = 'turn' AND m.role = 'user'
    ORDER BY cr.created_at, m.sequence_number, cr.id LIMIT 1
) m
$function$;

-- COALESCE avoids reading messages when a generated title/name is available.
CREATE OR REPLACE FUNCTION conversation_title(context_key text, client_session_key text)
RETURNS text LANGUAGE sql STABLE AS $function$
SELECT COALESCE(
    (SELECT s.ai_title FROM plugin_session_summaries s WHERE s.session_id = client_session_key),
    (SELECT NULLIF(c.name, 'Gateway conversation') FROM user_contexts c WHERE c.context_id = context_key),
    conversation_opening_prompt(context_key, 160),
    'Conversation ' || LEFT(context_key, 12) || '…')
$function$;

CREATE OR REPLACE FUNCTION refresh_conversation_facts(context_ids TEXT[])
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE written BIGINT; stamp TIMESTAMPTZ := clock_timestamp();
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
            COALESCE(SUM(r.latency_ms) FILTER (WHERE r.effective_kind = 'turn'), 0)::bigint AS active_ms,
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
        SELECT w.context_id, s.id, s.plugin_id, s.skill, s.invoked_at
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
    ), skill_upsert AS (
        INSERT INTO conversation_skill_facts AS t (
            context_id, plugin_id, skill, user_id, marketplace_id, marketplace_hash,
            invocations, failures, first_invoked_at, last_invoked_at, refreshed_at)
        SELECT k.context_id, k.plugin_id, k.skill, a.user_id, o.marketplace_id,
               marketplace_version_at(o.marketplace_id, MIN(k.invoked_at)),
               COUNT(*)::bigint,
               COUNT(*) FILTER (WHERE p.event_type = 'PostToolUseFailure')::bigint,
               MIN(k.invoked_at), MAX(k.invoked_at), stamp
        FROM skill_events k
        JOIN agg a ON a.context_id = k.context_id
        JOIN plugin_usage_events p ON p.id = k.id
        LEFT JOIN service_owned_ids o ON o.kind = 'plugin' AND o.id = k.plugin_id
        WHERE k.plugin_id IS NOT NULL
        GROUP BY k.context_id, k.plugin_id, k.skill, a.user_id, o.marketplace_id
        ON CONFLICT (context_id, plugin_id, skill) DO UPDATE SET
            user_id = EXCLUDED.user_id, marketplace_id = EXCLUDED.marketplace_id,
            marketplace_hash = EXCLUDED.marketplace_hash, invocations = EXCLUDED.invocations,
            failures = EXCLUDED.failures, first_invoked_at = EXCLUDED.first_invoked_at,
            last_invoked_at = EXCLUDED.last_invoked_at, refreshed_at = EXCLUDED.refreshed_at
        RETURNING 1
    ), upsert AS (
        INSERT INTO conversation_facts AS f (
            context_id, user_id, session_id, client_session_id, group_id, project_id,
            client_kind, client_attestation, wire_protocol, model, provider, models, providers,
            request_count, turn_count, side_call_count, side_call_cost_microdollars,
            error_count, rejected_count, streaming_count,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, reasoning_tokens,
            cost_microdollars, p50_latency_ms, p95_latency_ms, max_latency_ms, active_ms,
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
               a.cost_microdollars, a.p50::int, a.p95::int, a.max_latency_ms, a.active_ms,
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
            max_latency_ms = EXCLUDED.max_latency_ms, active_ms = EXCLUDED.active_ms,
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
    -- Why: a skill row this run did not rewrite belongs to a skill the
    -- conversation no longer shows. Only conversations rebuilt here are
    -- judged, so a conversation whose raw events expired keeps its rows.
    DELETE FROM conversation_skill_facts s
    USING conversation_facts f
    WHERE f.context_id = s.context_id AND f.refreshed_at >= stamp AND s.refreshed_at < stamp;
    IF context_ids IS NOT NULL THEN
        DELETE FROM conversation_facts f WHERE f.context_id = ANY(context_ids)
            AND NOT EXISTS (SELECT 1 FROM conversation_requests r WHERE r.context_id = f.context_id);
    END IF;
    RETURN COALESCE(written, 0);
END $$;

-- Every context touched in [from_at, to_at) on any plane that feeds the row: a
-- request settled, a hook event, a governance decision, a tool execution or
-- an artifact.
CREATE OR REPLACE FUNCTION refresh_conversation_facts_between(from_at TIMESTAMPTZ, to_at TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE sql AS $$
SELECT refresh_conversation_facts(ARRAY(
    SELECT DISTINCT canonical_conversation_context(context_id) FROM (
        SELECT r.context_id FROM ai_requests r
        WHERE (r.updated_at >= from_at AND r.updated_at < to_at)
           OR (r.created_at >= from_at AND r.created_at < to_at)
        UNION ALL
        SELECT r.context_id FROM ai_requests r
        WHERE r.client_session_id IN (
            SELECT e.session_id FROM plugin_usage_events e
            WHERE e.created_at >= from_at AND e.created_at < to_at
            UNION ALL
            SELECT d.session_id FROM governance_decisions d
            WHERE d.created_at >= from_at AND d.created_at < to_at
            UNION ALL
            SELECT x.trace_id FROM mcp_tool_executions x
            WHERE x.created_at >= from_at AND x.created_at < to_at AND x.trace_id IS NOT NULL)
        UNION ALL
        SELECT x.context_id FROM mcp_tool_executions x
        WHERE x.created_at >= from_at AND x.created_at < to_at AND x.context_id IS NOT NULL
        UNION ALL
        SELECT m.context_id FROM mcp_artifacts m
        WHERE m.created_at >= from_at AND m.created_at < to_at AND m.context_id IS NOT NULL
        UNION ALL
        SELECT d.context_id FROM governance_decisions d
        WHERE d.created_at >= from_at AND d.created_at < to_at
    ) touched
    WHERE context_id IS NOT NULL AND context_id <> '00000000-0000-0000-0000-4c4547414359'))
$$;

-- One rollup tick: every context touched between the stored watermark and a
-- new high-water mark, then the mark moves. Consecutive ticks cover disjoint
-- windows, so a conversation is rebuilt once per change, not once per tick.
--
-- Why the mark trails the clock: a row carries the `now()` of the transaction
-- that wrote it, so a transaction still open can commit a row stamped before
-- this tick. The mark stops at the oldest open transaction's start, less a
-- margin for timestamps the application stamps before it begins one; the
-- next tick picks up whatever that transaction commits. A transaction open
-- longer than ten minutes stops holding the mark back, so one stuck session
-- cannot freeze the rollup. The state row is held FOR UPDATE, so two ticks
-- never overlap.
CREATE OR REPLACE FUNCTION refresh_conversation_facts_pending()
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE from_at TIMESTAMPTZ; to_at TIMESTAMPTZ; written BIGINT;
BEGIN
    INSERT INTO conversation_rollup_state(id, watermark)
    VALUES (TRUE, now() - interval '3 minutes')
    ON CONFLICT (id) DO NOTHING;
    SELECT s.watermark INTO from_at FROM conversation_rollup_state s WHERE s.id FOR UPDATE;
    SELECT LEAST(now(), GREATEST(COALESCE(MIN(a.xact_start), now()), now() - interval '10 minutes'))
           - interval '30 seconds' INTO to_at
    FROM pg_stat_activity a
    WHERE a.datname = current_database() AND a.pid <> pg_backend_pid() AND a.xact_start IS NOT NULL;
    IF to_at <= from_at THEN
        RETURN 0;
    END IF;
    written := refresh_conversation_facts_between(from_at, to_at);
    UPDATE conversation_rollup_state SET watermark = to_at, updated_at = clock_timestamp() WHERE id;
    RETURN written;
END $$;

DELETE FROM conversation_facts f
WHERE EXISTS (SELECT 1 FROM conversation_context_aliases a
              WHERE a.original_context_id = f.context_id AND a.context_id <> f.context_id);
SELECT refresh_conversation_facts(ARRAY(SELECT context_id FROM conversation_session_contexts));
