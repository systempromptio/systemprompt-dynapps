\timing on
-- F33 logs page: level+time filter with message LIKE (admin logs view / infra logs view --level error --since 1h)
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT COUNT(*) FROM logs WHERE ('ERROR'::VARCHAR IS NULL OR level = 'ERROR') AND (NULL::VARCHAR IS NULL OR module = NULL) AND (NULL::VARCHAR IS NULL OR message LIKE NULL) AND ('2026-09-21'::TIMESTAMPTZ IS NULL OR timestamp >= '2026-09-21');
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT id, timestamp, level, module, message FROM logs WHERE message ILIKE '%bridge%' AND (timestamp >= '2026-09-15') AND (UPPER(level) = 'ERROR') ORDER BY timestamp DESC LIMIT 50;
-- trace list (list_queries.rs) with no filters, limit 20
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT trace_id, MIN(timestamp) FROM logs WHERE trace_id IS NOT NULL GROUP BY trace_id ORDER BY 2 DESC LIMIT 20;
-- module top-N
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT module, COUNT(*) FROM logs WHERE timestamp >= '2026-09-15' GROUP BY module ORDER BY 2 DESC LIMIT 20;
-- distinct user ids (orphan log cleanup in database_cleanup)
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT DISTINCT user_id FROM logs WHERE user_id IS NOT NULL;
-- conversations list realistic: last 30 days, page 1
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT f.*, u.name FROM conversation_facts f JOIN users u ON u.id=f.user_id WHERE f.last_at >= now()-interval '30 days' ORDER BY f.last_at DESC LIMIT 50;
-- costs by user 30d
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT user_id, count(*), sum(cost_microdollars), sum(tokens_used), count(DISTINCT context_id) FROM ai_requests WHERE created_at >= now()-interval '30 days' GROUP BY 1 ORDER BY 3 DESC LIMIT 20;
-- governance decisions page 30d
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT * FROM governance_decisions WHERE created_at >= now()-interval '30 days' ORDER BY created_at DESC LIMIT 50;
-- transcript of a conversation: last request's messages
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT m.role, m.content FROM ai_request_messages m WHERE m.request_id = (SELECT id FROM ai_requests WHERE context_id=(SELECT context_id FROM ai_requests GROUP BY 1 ORDER BY count(*) DESC LIMIT 1) ORDER BY created_at DESC LIMIT 1) ORDER BY m.sequence_number;
-- request audit: payload fetch
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) SELECT pg_column_size(request_body), pg_column_size(offered_tools) FROM ai_request_payloads WHERE ai_request_id = (SELECT id FROM ai_requests ORDER BY created_at DESC LIMIT 1);
-- indexes on logs
SELECT indexname, pg_size_pretty(pg_relation_size(indexname::regclass)) FROM pg_indexes WHERE tablename='logs' ORDER BY 2;
