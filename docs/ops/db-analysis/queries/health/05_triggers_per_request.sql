-- Row-level triggers on the per-request write path (statements per AI request).
SELECT c.relname AS table_name, t.tgname, p.proname AS function,
       CASE WHEN t.tgtype & 1 = 1 THEN 'ROW' ELSE 'STATEMENT' END AS level
FROM pg_trigger t
JOIN pg_class c ON c.oid = t.tgrelid
JOIN pg_proc p ON p.oid = t.tgfoid
WHERE NOT t.tgisinternal
  AND c.relname IN ('ai_requests','ai_request_messages','ai_request_payloads','ai_request_tool_calls',
                    'ai_safety_findings','governance_decisions','user_sessions','logs','mcp_tool_executions',
                    'analytics_events','plugin_usage_events','ai_request_scopes','event_outbox')
ORDER BY c.relname, t.tgname;
