-- Logical references without a parent row.
SELECT 'ai_requests.user_id' AS ref, count(*) FROM ai_requests a LEFT JOIN users u ON u.id = a.user_id WHERE u.id IS NULL
UNION ALL SELECT 'ai_requests.session_id', count(*) FROM ai_requests a LEFT JOIN user_sessions s ON s.session_id = a.session_id WHERE s.session_id IS NULL
UNION ALL SELECT 'ai_requests.context_id', count(*) FROM ai_requests a LEFT JOIN user_contexts c ON c.context_id = a.context_id WHERE a.context_id IS NOT NULL AND c.context_id IS NULL
UNION ALL SELECT 'governance_decisions.user_id', count(*) FROM governance_decisions g LEFT JOIN users u ON u.id = g.user_id WHERE u.id IS NULL
UNION ALL SELECT 'governance_decisions.trace_id->ai_requests', count(*) FROM governance_decisions g WHERE g.trace_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests a WHERE a.trace_id = g.trace_id) AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions m WHERE m.trace_id = g.trace_id)
UNION ALL SELECT 'logs.user_id', count(*) FROM logs l LEFT JOIN users u ON u.id = l.user_id WHERE l.user_id IS NOT NULL AND u.id IS NULL
UNION ALL SELECT 'plugin_usage_events.user_id', count(*) FROM plugin_usage_events p LEFT JOIN users u ON u.id = p.user_id WHERE u.id IS NULL
UNION ALL SELECT 'ai_request_scopes.request', count(*) FROM ai_request_scopes s LEFT JOIN ai_requests a ON a.id = s.request_id WHERE a.id IS NULL
UNION ALL SELECT 'ai_request_tool_calls.mcp_execution_id', count(*) FROM ai_request_tool_calls t WHERE t.mcp_execution_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions m WHERE m.mcp_execution_id = t.mcp_execution_id)
UNION ALL SELECT 'user_sessions.user_id', count(*) FROM user_sessions s LEFT JOIN users u ON u.id = s.user_id WHERE u.id IS NULL;
