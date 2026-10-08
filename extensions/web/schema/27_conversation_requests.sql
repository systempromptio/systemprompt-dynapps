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
RETURNS trigger LANGUAGE plpgsql AS $$
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
$$;
DROP TRIGGER IF EXISTS ai_requests_conversation_session ON ai_requests;
CREATE TRIGGER ai_requests_conversation_session
AFTER INSERT OR UPDATE OF user_id, client_session_id, context_id ON ai_requests
FOR EACH ROW EXECUTE FUNCTION register_conversation_session();

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
