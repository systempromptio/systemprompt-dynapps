-- Fact tables the analysis pages read, refreshed after the seed commits.
-- Separate from the load transaction: fresh statistics make the refresh
-- functions' lateral joins plan sanely on a just-loaded table.
ANALYZE ai_requests, governance_decisions, plugin_usage_events, mcp_tool_executions,
        ai_request_tool_calls, conversation_session_contexts, ai_safety_findings;
SELECT refresh_trace_facts(ARRAY(
    SELECT DISTINCT r.trace_id::text FROM ai_requests r JOIN users u ON u.id = r.user_id
     WHERE u.email LIKE '%@vetgroup-demo.example'));
SELECT refresh_conversation_facts(ARRAY(
    SELECT DISTINCT r.context_id::text FROM ai_requests r JOIN users u ON u.id = r.user_id
     WHERE u.email LIKE '%@vetgroup-demo.example'));
