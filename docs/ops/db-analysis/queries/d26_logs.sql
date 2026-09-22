-- D26: logs
\echo == D26 rows per level
SELECT level, count(*), min(timestamp)::date first, max(timestamp)::date last FROM logs GROUP BY 1 ORDER BY 2 DESC;
\echo == D26 module top 20
SELECT module, count(*), count(*) FILTER (WHERE level='ERROR') errors, count(*) FILTER (WHERE level='WARN') warns FROM logs GROUP BY 1 ORDER BY 2 DESC LIMIT 20;
\echo == D26 ERROR shapes top 20
SELECT module, left(regexp_replace(message, '[0-9a-f]{8}-[0-9a-f-]{27}|[0-9]+', '#', 'g'), 120) shape, count(*), min(timestamp)::date first, max(timestamp)::date last FROM logs WHERE level='ERROR' GROUP BY 1,2 ORDER BY 3 DESC LIMIT 20;
\echo == D26 WARN shapes top 15
SELECT module, left(regexp_replace(message, '[0-9a-f]{8}-[0-9a-f-]{27}|[0-9]+', '#', 'g'), 120) shape, count(*) FROM logs WHERE level='WARN' GROUP BY 1,2 ORDER BY 3 DESC LIMIT 15;
\echo == D26 gateway access-log lines by status class (the INFO bulk)
SELECT substring(message FROM '-> ([0-9]{3})') status, count(*), min(timestamp)::date, max(timestamp)::date FROM logs WHERE module='systemprompt_api::gateway' GROUP BY 1 ORDER BY 2 DESC;
\echo == D26 gateway lines by route (top 12)
SELECT regexp_replace(substring(message FROM '^[A-Z]+ ([^ ?]+)'), '[0-9a-f]{8}-[0-9a-f-]{27}|[0-9]{3,}', '#', 'g') route, count(*), count(*) FILTER (WHERE level='ERROR') errors FROM logs WHERE module='systemprompt_api::gateway' GROUP BY 1 ORDER BY 2 DESC LIMIT 12;
\echo == D26 /v1/bridge/latest by status and day
SELECT timestamp::date d, substring(message FROM '-> ([0-9]{3})') status, count(*) FROM logs WHERE message LIKE 'GET /v1/bridge/latest%' GROUP BY 1,2 ORDER BY 1,2;
\echo == D26 logs.user_id / session_id sentinel values and real-id coverage
SELECT count(*) FILTER (WHERE user_id='system') user_system, count(*) FILTER (WHERE session_id='system') session_system, count(*) FILTER (WHERE user_id <> 'system') user_real,
       count(*) FILTER (WHERE user_id <> 'system' AND NOT EXISTS (SELECT 1 FROM users u WHERE u.id = l.user_id)) user_real_orphan,
       count(*) FILTER (WHERE trace_id IS NOT NULL) with_trace, count(*) FILTER (WHERE context_id IS NOT NULL) with_context FROM logs l;
\echo == D26 logs with a trace_id: do they join ai_requests?
SELECT count(*), count(*) FILTER (WHERE EXISTS (SELECT 1 FROM ai_requests r WHERE r.trace_id = l.trace_id)) joins_request FROM logs l WHERE trace_id IS NOT NULL;
