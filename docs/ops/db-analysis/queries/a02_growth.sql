-- A2: rows per day for the spine tables over the data's lifetime (dump spans 2026-09-02..22)
\echo == A2 daily rows: ai_requests / messages / governance / executions / hook events / logs / outbox
WITH d AS (SELECT generate_series(date '2026-09-02', date '2026-09-22', '1 day')::date AS day)
SELECT d.day,
  (SELECT count(*) FROM ai_requests r WHERE r.created_at::date = d.day) AS ai_requests,
  (SELECT count(*) FROM ai_request_messages m WHERE m.created_at::date = d.day) AS ai_msgs,
  (SELECT count(*) FROM governance_decisions g WHERE g.created_at::date = d.day) AS gov,
  (SELECT count(*) FROM mcp_tool_executions e WHERE e.started_at::date = d.day) AS mcp_exec,
  (SELECT count(*) FROM plugin_usage_events p WHERE p.created_at::date = d.day) AS hook_ev,
  (SELECT count(*) FROM logs l WHERE l.timestamp::date = d.day) AS logs,
  (SELECT count(*) FROM event_outbox o WHERE o.created_at::date = d.day) AS outbox,
  (SELECT count(*) FROM user_sessions s WHERE s.started_at::date = d.day) AS sessions
FROM d ORDER BY 1;
\echo == A2 bytes per ai_request (amortised across dependent tables) and 12-month projection at last-7-day rate
WITH r7 AS (SELECT count(*)::numeric / 7 AS per_day FROM ai_requests WHERE created_at >= now() - interval '7 days'),
     sz AS (SELECT sum(pg_total_relation_size(relid)) AS bytes FROM pg_stat_user_tables
            WHERE relname IN ('ai_requests','ai_request_messages','ai_request_payloads','ai_request_tool_calls','ai_request_scopes','ai_safety_findings','governance_decisions','mcp_tool_executions','plugin_usage_events','event_outbox','feedback_ingestion_outbox','logs'))
SELECT r7.per_day AS req_per_day, pg_size_pretty(sz.bytes) AS spine_bytes_now,
       (SELECT count(*) FROM ai_requests) AS requests_now,
       pg_size_pretty((sz.bytes / (SELECT count(*) FROM ai_requests))::bigint) AS bytes_per_request,
       pg_size_pretty(((sz.bytes / (SELECT count(*) FROM ai_requests)) * r7.per_day * 365)::bigint) AS projected_12m_at_current_rate
FROM r7, sz;
\echo == A2 logs per day by level (logs is the largest table)
SELECT timestamp::date AS day, count(*) FILTER (WHERE level='INFO') info, count(*) FILTER (WHERE level='WARN') warn, count(*) FILTER (WHERE level='ERROR') error FROM logs GROUP BY 1 ORDER BY 1;
