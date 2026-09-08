-- One row per gateway request, classified for the conversation pages.
--
-- Rule: a side call is a probe, or a single-request thread that offered no
-- tools. Everything else is a turn. `effective_kind` is one of
-- 'turn', 'probe', 'utility'.
--
-- A thread is one `gateway_conversation_id` inside a context (sub-agents and
-- post-compaction histories get their own). The legacy sentinel context that
-- pools every context-less request is excluded here so no page has to.
CREATE OR REPLACE VIEW conversation_requests AS
WITH threads AS (
    SELECT context_id, gateway_conversation_id, COUNT(*)::bigint AS thread_requests
    FROM ai_requests
    GROUP BY context_id, gateway_conversation_id
)
SELECT r.id, r.user_id, r.session_id, r.client_session_id, r.context_id,
       r.gateway_conversation_id, r.trace_id, r.provider, r.model, r.status,
       r.max_tokens, r.input_tokens, r.output_tokens, r.cost_microdollars,
       r.latency_ms, r.created_at, r.completed_at,
       CASE WHEN r.request_kind = 'probe'   THEN 'probe'
            WHEN r.request_kind = 'utility' THEN 'utility'
            WHEN p.offered_tools IS NULL AND t.thread_requests = 1 THEN 'utility'
            ELSE 'turn' END AS effective_kind
FROM ai_requests r
LEFT JOIN ai_request_payloads p ON p.ai_request_id = r.id
JOIN threads t ON t.context_id = r.context_id
   AND t.gateway_conversation_id IS NOT DISTINCT FROM r.gateway_conversation_id
WHERE r.context_id <> '00000000-0000-0000-0000-4c4547414359';

-- One row per conversation (context), rolled up from `conversation_requests`.
--
-- Title resolution, in order: the hook pipeline's generated title (joined via
-- the Claude Code session uuid), the context's own name unless it is the
-- gateway's placeholder, the first human prompt of the earliest turn, and
-- finally the context id. The owner is the user of the latest request.
CREATE OR REPLACE VIEW conversation_rollups AS
WITH agg AS (
    SELECT
        context_id,
        COUNT(*) FILTER (WHERE effective_kind = 'turn')::bigint             AS turn_count,
        COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint            AS side_call_count,
        COALESCE(SUM(cost_microdollars) FILTER (WHERE effective_kind <> 'turn'), 0)::bigint
                                                                            AS side_call_cost_microdollars,
        COUNT(*) FILTER (WHERE status = 'failed')::bigint                   AS error_count,
        COALESCE(SUM(input_tokens), 0)::bigint                              AS total_input_tokens,
        COALESCE(SUM(output_tokens), 0)::bigint                             AS total_output_tokens,
        COALESCE(SUM(cost_microdollars), 0)::bigint                         AS total_cost_microdollars,
        MIN(created_at)                                                     AS first_at,
        MAX(created_at)                                                     AS last_request_at,
        (ARRAY_AGG(user_id ORDER BY created_at DESC))[1]                    AS user_id,
        (ARRAY_AGG(session_id ORDER BY created_at DESC)
            FILTER (WHERE session_id IS NOT NULL))[1]                       AS session_id,
        (ARRAY_AGG(client_session_id ORDER BY created_at DESC)
            FILTER (WHERE client_session_id IS NOT NULL))[1]                AS client_session_id,
        (ARRAY_AGG(model ORDER BY created_at DESC)
            FILTER (WHERE effective_kind = 'turn' AND model IS NOT NULL))[1] AS model
    FROM conversation_requests
    GROUP BY context_id
),
tools AS (
    SELECT cr.context_id, COUNT(*)::bigint AS tool_call_count
    FROM ai_request_tool_calls t
    JOIN conversation_requests cr ON cr.id = t.request_id
    WHERE cr.effective_kind = 'turn'
    GROUP BY cr.context_id
),
first_prompt AS (
    SELECT DISTINCT ON (cr.context_id)
        cr.context_id,
        LEFT(
            NULLIF(
                BTRIM(regexp_replace(
                    regexp_replace(
                        split_part(m.content, '=== ASSISTANT ANSWER ===', 1),
                        '=== USER PROMPT ===', '', 'g'),
                    '\s+', ' ', 'g')),
                ''),
            160) AS prompt
    FROM ai_request_messages m
    JOIN conversation_requests cr ON cr.id = m.request_id
    WHERE cr.effective_kind = 'turn' AND m.role = 'user'
    ORDER BY cr.context_id, cr.created_at ASC, m.sequence_number ASC
)
SELECT
    a.context_id,
    COALESCE(
        s.ai_title,
        NULLIF(c.name, 'Gateway conversation'),
        fp.prompt,
        'Conversation ' || LEFT(a.context_id, 12) || '…')                  AS title,
    a.user_id,
    u.display_name,
    a.session_id,
    a.client_session_id,
    g.name                                                                  AS group_name,
    p.name                                                                  AS project_name,
    a.model,
    a.turn_count,
    a.side_call_count,
    a.side_call_cost_microdollars,
    COALESCE(t.tool_call_count, 0)::bigint                                  AS tool_call_count,
    a.error_count,
    a.total_input_tokens,
    a.total_output_tokens,
    a.total_cost_microdollars,
    a.first_at,
    GREATEST(a.last_request_at, c.updated_at)                               AS last_at,
    s.status
FROM agg a
LEFT JOIN user_contexts c ON c.context_id = a.context_id
LEFT JOIN plugin_session_summaries s ON s.session_id = a.client_session_id
LEFT JOIN tools t ON t.context_id = a.context_id
LEFT JOIN first_prompt fp ON fp.context_id = a.context_id
LEFT JOIN users u ON u.id = a.user_id
LEFT JOIN user_scope_defaults d ON d.user_id = a.user_id
LEFT JOIN groups g ON g.id = d.primary_group_id
LEFT JOIN projects p ON p.id = d.primary_project_id;
