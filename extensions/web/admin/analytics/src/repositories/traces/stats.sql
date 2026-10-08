WITH trace_to_session AS (
    SELECT DISTINCT trace_id, session_id
    FROM ai_requests
    WHERE created_at >= $1 AND created_at < $2
      AND trace_id IS NOT NULL AND session_id IS NOT NULL
),
-- Hook rows carry Claude Code's uuid, gateway rows the bridge session; the
-- gateway row holds both, so the uuid folds into the session (see list.sql).
client_to_session AS (
    SELECT
        client_session_id,
        (ARRAY_AGG(session_id ORDER BY created_at DESC))[1] AS session_id
    FROM ai_requests
    WHERE created_at >= $1 AND created_at < $2
      AND client_session_id IS NOT NULL AND session_id IS NOT NULL
    GROUP BY client_session_id
),
all_sessions AS (
    SELECT COALESCE(c.session_id, e.session_id) AS session_id,
           e.created_at, NULL::text AS decision, NULL::text AS status,
           e.event_type, 'evt'::text AS source
    FROM plugin_usage_events e
    LEFT JOIN client_to_session c ON c.client_session_id = e.session_id
    WHERE e.created_at >= $1 AND e.created_at < $2 AND e.session_id IS NOT NULL
      AND ($3::TEXT[] IS NULL OR e.user_id = ANY($3))
    UNION ALL
    SELECT COALESCE(t.session_id, c.session_id, NULLIF(g.session_id, ''), g.trace_id) AS session_id,
           g.created_at, g.decision, NULL::text, NULL::text AS event_type,
           'gov'::text AS source
    FROM governance_decisions g
    LEFT JOIN trace_to_session t ON t.trace_id = g.trace_id
    LEFT JOIN client_to_session c ON c.client_session_id = NULLIF(g.session_id, '')
    WHERE g.created_at >= $1 AND g.created_at < $2
      AND (NULLIF(g.session_id, '') IS NOT NULL OR g.trace_id IS NOT NULL)
      AND ($3::TEXT[] IS NULL OR g.user_id = ANY($3))
    UNION ALL
    SELECT session_id, created_at, NULL::text, status::text, NULL::text,
           'ai'::text AS source
    FROM ai_requests
    WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
      AND ($3::TEXT[] IS NULL OR ai_requests.user_id = ANY($3))
),
per_session AS (
    SELECT
        session_id,
        BOOL_OR(decision = 'deny') AS has_deny,
        BOOL_OR(status NOT IN ('ok','success','completed','pending') AND status IS NOT NULL)
          AS has_error,
        -- Why: the same three kinds `list.sql` classifies, so the cards count
        -- the population the table lists rather than a wider one.
        CASE
            WHEN COUNT(*) FILTER (WHERE source = 'ai') > 0 THEN 'gateway'
            WHEN COUNT(*) FILTER (
                     WHERE source = 'gov'
                        OR (source = 'evt' AND event_type NOT IN (
                                'SessionStart', 'SessionEnd', 'Stop',
                                'SubagentStart', 'SubagentStop', 'Notification'))
                 ) > 0 THEN 'hook'
            ELSE 'lifecycle'
        END AS kind
    FROM all_sessions
    GROUP BY session_id
),
listed AS (
    SELECT * FROM per_session WHERE $4::BOOLEAN OR kind <> 'lifecycle'
),
-- Percentiles and totals come from the request rows only: a
-- governance-only trace has no latency, and counting it as 0 ms would
-- pin p50 to zero however slow the real traffic was.
active AS (
    SELECT
        session_id,
        COALESCE(SUM(latency_ms), 0)::bigint       AS active_ms,
        COALESCE(SUM(cost_microdollars), 0)::bigint AS cost_microdollars,
        COALESCE(SUM(tokens_used), 0)::bigint       AS tokens
    FROM ai_requests
    WHERE created_at >= $1 AND created_at < $2 AND session_id IS NOT NULL
      AND ($3::TEXT[] IS NULL OR ai_requests.user_id = ANY($3))
    GROUP BY session_id
)
SELECT
    (SELECT COUNT(*) FROM listed)::bigint                           AS "total_traces!",
    (SELECT COUNT(*) FROM listed WHERE has_error)::bigint           AS "error_count!",
    (SELECT COUNT(*) FROM listed WHERE has_deny)::bigint            AS "deny_count!",
    -- Why: cost, tokens and the percentiles below come from `active`, which is
    -- request rows only. Reporting its size beside them stops the cards from
    -- reading as one population when they are two.
    (SELECT COUNT(*) FROM active)::bigint                           AS "active_traces!",
    (SELECT COUNT(*) FROM per_session WHERE kind = 'lifecycle')::bigint
                                                                    AS "lifecycle_count!",
    COALESCE(percentile_disc(0.50) WITHIN GROUP (ORDER BY active_ms), 0)::bigint
                                                                    AS "p50!",
    COALESCE(percentile_disc(0.95) WITHIN GROUP (ORDER BY active_ms), 0)::bigint
                                                                    AS "p95!",
    COALESCE(percentile_disc(0.99) WITHIN GROUP (ORDER BY active_ms), 0)::bigint
                                                                    AS "p99!",
    COALESCE(SUM(cost_microdollars), 0)::bigint                     AS "total_cost!",
    COALESCE(SUM(tokens), 0)::bigint                                AS "total_tokens!"
FROM active
