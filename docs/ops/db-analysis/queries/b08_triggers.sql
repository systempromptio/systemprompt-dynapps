-- B8: every trigger on the spine tables, with level; per-row triggers on tables > 100k rows flagged.
\echo == B8 triggers on spine tables
SELECT c.relname AS table, t.tgname AS trigger,
       CASE WHEN t.tgtype & 2 > 0 THEN 'BEFORE' ELSE 'AFTER' END AS timing,
       concat_ws('/', CASE WHEN t.tgtype & 4 > 0 THEN 'INSERT' END, CASE WHEN t.tgtype & 8 > 0 THEN 'DELETE' END, CASE WHEN t.tgtype & 16 > 0 THEN 'UPDATE' END) AS events,
       CASE WHEN t.tgtype & 1 > 0 THEN 'ROW' ELSE 'STATEMENT' END AS level,
       p.proname AS function, s.n_live_tup AS rows
FROM pg_trigger t JOIN pg_class c ON c.oid = t.tgrelid JOIN pg_proc p ON p.oid = t.tgfoid
LEFT JOIN pg_stat_user_tables s ON s.relid = c.oid
WHERE NOT t.tgisinternal AND c.relname IN ('ai_requests','ai_request_messages','ai_request_tool_calls','user_sessions','logs','mcp_tool_executions','analytics_events','plugin_usage_events','conversation_analyses','governance_decisions','agent_tasks','task_messages','user_contexts','mcp_artifacts','users')
ORDER BY 1,2,3;
\echo == B8 per-row triggers on tables with > 100k rows
SELECT c.relname, t.tgname, s.n_live_tup FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_stat_user_tables s ON s.relid=c.oid
WHERE NOT t.tgisinternal AND t.tgtype & 1 > 0 AND s.n_live_tup > 100000 ORDER BY 3 DESC;
\echo == B8 all triggers in the database by level
SELECT CASE WHEN tgtype & 1 > 0 THEN 'ROW' ELSE 'STATEMENT' END AS level, count(*) FROM pg_trigger WHERE NOT tgisinternal GROUP BY 1;
