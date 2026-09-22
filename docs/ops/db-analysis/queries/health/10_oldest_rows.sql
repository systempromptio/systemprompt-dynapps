-- Oldest row per retention-managed table.
SELECT 'logs' AS t, min("timestamp") AS oldest, count(*) FROM logs
UNION ALL SELECT 'ai_requests', min(created_at), count(*) FROM ai_requests
UNION ALL SELECT 'ai_request_messages', min(created_at), count(*) FROM ai_request_messages
UNION ALL SELECT 'ai_request_payloads', min(created_at), count(*) FROM ai_request_payloads
UNION ALL SELECT 'governance_decisions', min(created_at), count(*) FROM governance_decisions
UNION ALL SELECT 'analytics_events', min("timestamp"), count(*) FROM analytics_events
UNION ALL SELECT 'event_outbox', min(created_at), count(*) FROM event_outbox
UNION ALL SELECT 'mcp_tool_executions', min(started_at), count(*) FROM mcp_tool_executions
UNION ALL SELECT 'user_sessions', min(started_at), count(*) FROM user_sessions
UNION ALL SELECT 'ai_quota_buckets', min(window_start), count(*) FROM ai_quota_buckets
UNION ALL SELECT 'user_rate_limit_buckets', min(window_start), count(*) FROM user_rate_limit_buckets
UNION ALL SELECT 'plugin_usage_events', min(created_at), count(*) FROM plugin_usage_events
UNION ALL SELECT 'engagement_events', min(created_at), count(*) FROM engagement_events;
