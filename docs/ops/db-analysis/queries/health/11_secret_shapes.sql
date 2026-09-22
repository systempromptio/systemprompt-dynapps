-- Secret-looking values stored in clear: counts only, never values.
SELECT 'ai_request_payloads.request_body' AS col, count(*) FROM ai_request_payloads WHERE request_body::text ~ '(sk-ant-|sk-[A-Za-z0-9]{20}|BEGIN PRIVATE KEY|eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.)'
UNION ALL SELECT 'ai_request_messages.content', count(*) FROM ai_request_messages WHERE content ~ '(sk-ant-|sk-[A-Za-z0-9]{20}|BEGIN PRIVATE KEY|eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.)'
UNION ALL SELECT 'logs.metadata', count(*) FROM logs WHERE metadata::text ~ '(sk-ant-|Bearer [A-Za-z0-9._-]{20,}|BEGIN PRIVATE KEY)'
UNION ALL SELECT 'mcp_tool_executions.input', count(*) FROM mcp_tool_executions WHERE input::text ~ '(sk-ant-|Bearer [A-Za-z0-9._-]{20,}|BEGIN PRIVATE KEY)'
UNION ALL SELECT 'plugin_usage_events.metadata', count(*) FROM plugin_usage_events WHERE metadata::text ~ '(sk-ant-|Bearer [A-Za-z0-9._-]{20,}|BEGIN PRIVATE KEY)';
