-- C14/C15: ingestion producers, fact checkpoints, analytics projection + outbox (evidence of the known rebuild loop)
\echo == C14 analytics_ingestion_producers.pending_count vs actual pending feedback_ingestion_outbox rows
SELECT p.producer, p.pending_count AS stored, (SELECT count(*) FROM feedback_ingestion_outbox o WHERE o.pending AND o.kind = p.producer) AS actual_by_kind, p.oldest_pending_at::timestamp(0), p.last_drained_at::timestamp(0) FROM analytics_ingestion_producers p ORDER BY 1;
SELECT kind, pending, count(*), min(created_at)::timestamp(0) oldest, max(attempts) max_attempts, count(*) FILTER (WHERE last_error IS NOT NULL) errored FROM feedback_ingestion_outbox GROUP BY 1,2 ORDER BY 1,2;
\echo == C14 analytics_fact_checkpoints vs analytics_fact_changes
SELECT (SELECT count(*) FROM analytics_fact_checkpoints) checkpoints, (SELECT count(*) FROM analytics_fact_changes) changes, (SELECT string_agg(state||'='||n, ', ') FROM (SELECT state, count(*) n FROM analytics_fact_changes GROUP BY 1) s) states;
\echo == C15 analytics projection state
SELECT * FROM analytics_projection_state;
SELECT count(*) revisions FROM analytics_projection_revisions;
\echo == C15 analytics_report_* row counts vs sources
SELECT 'users' src, (SELECT count(*) FROM users) source_rows, (SELECT count(*) FROM analytics_report_users) report_rows
UNION ALL SELECT 'user_sessions', (SELECT count(*) FROM user_sessions), (SELECT count(*) FROM analytics_report_user_sessions)
UNION ALL SELECT 'ai_requests', (SELECT count(*) FROM ai_requests), (SELECT count(*) FROM analytics_report_ai_requests)
UNION ALL SELECT 'mcp_tool_executions', (SELECT count(*) FROM mcp_tool_executions), (SELECT count(*) FROM analytics_report_mcp_tool_executions)
UNION ALL SELECT 'user_contexts', (SELECT count(*) FROM user_contexts), (SELECT count(*) FROM analytics_report_user_contexts)
UNION ALL SELECT 'analytics_events', (SELECT count(*) FROM analytics_events), (SELECT count(*) FROM analytics_report_analytics_events)
UNION ALL SELECT 'ai_request_messages', (SELECT count(*) FROM ai_request_messages), (SELECT count(*) FROM analytics_report_ai_request_messages)
UNION ALL SELECT 'logs', (SELECT count(*) FROM logs), (SELECT count(*) FROM analytics_report_logs);
\echo == C15 event_outbox: pending by consumer/source, age of oldest, all created in one burst?
SELECT consumer, fact->>'source' src, count(*) n, count(*) FILTER (WHERE processed_at IS NULL) pending, min(created_at)::timestamp(0) oldest, max(created_at)::timestamp(0) newest FROM event_outbox GROUP BY 1,2 ORDER BY 3 DESC;
SELECT date_trunc('minute', created_at) AS minute, count(*) FROM event_outbox GROUP BY 1 ORDER BY 2 DESC LIMIT 5;
\echo == C15 outbox facts per source vs source row counts (is the backlog a full snapshot, i.e. the rebuild's own emission?)
SELECT fact->>'source' src, count(*) facts, count(DISTINCT fact->>'entity_key') entities FROM event_outbox GROUP BY 1 ORDER BY 2 DESC;
\echo == C15 sample fact shape (keys only)
SELECT DISTINCT jsonb_object_keys(fact) FROM event_outbox;
