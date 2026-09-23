-- C17: ai_safety_findings vs ai_requests; ai_request_tool_calls vs mcp_tool_executions; the three tool-call planes
\echo == C17 safety findings: orphans, per phase/category/blocked
SELECT count(*) findings, count(*) FILTER (WHERE NOT EXISTS (SELECT 1 FROM ai_requests r WHERE r.id = f.ai_request_id)) orphan_findings,
       count(DISTINCT ai_request_id) requests_with_findings FROM ai_safety_findings f;
SELECT phase, category, blocked, count(*) FROM ai_safety_findings GROUP BY 1,2,3 ORDER BY 4 DESC LIMIT 10;
\echo == C17 the three tool-call planes and their links
SELECT (SELECT count(*) FROM ai_request_tool_calls) intents,
       (SELECT count(*) FROM ai_request_tool_calls WHERE mcp_execution_id IS NOT NULL) intents_linked_to_execution,
       (SELECT count(*) FROM mcp_tool_executions) executions,
       (SELECT count(*) FROM mcp_tool_executions WHERE ai_tool_call_id IS NOT NULL) executions_linked_to_intent,
       (SELECT count(*) FROM mcp_tool_executions WHERE correlation IS NOT NULL) executions_with_correlation,
       (SELECT count(*) FROM plugin_usage_events WHERE event_type LIKE 'PostToolUse%') hook_tool_events,
       (SELECT count(*) FROM plugin_usage_events p WHERE event_type LIKE 'PostToolUse%' AND EXISTS (SELECT 1 FROM ai_request_tool_calls t WHERE t.ai_tool_call_id = p.tool_use_id)) hook_events_matching_intent,
       (SELECT count(*) FROM plugin_usage_events WHERE mcp_execution_id IS NOT NULL) hook_events_linked_to_execution,
       (SELECT count(*) FROM tool_call_ledger) ledger_rows,
       (SELECT string_agg(state||'='||n, ', ') FROM (SELECT state, count(*) n FROM tool_call_ledger GROUP BY 1) s) ledger_states;
\echo == C17 executions by source/correlation/status
SELECT source, correlation, status, count(*), count(*) FILTER (WHERE ai_tool_call_id IS NOT NULL) linked FROM mcp_tool_executions GROUP BY 1,2,3 ORDER BY 4 DESC;
\echo == C17 intents whose tool is an MCP tool (mcp__server__tool) vs executions by server: are executions a subset of intents?
SELECT count(*) intents_mcp, count(DISTINCT request_id) requests FROM ai_request_tool_calls WHERE tool_name LIKE 'mcp\_\_%';
SELECT split_part(tool_name,'__',2) AS server, count(*) intents FROM ai_request_tool_calls WHERE tool_name LIKE 'mcp\_\_%' GROUP BY 1 ORDER BY 2 DESC LIMIT 8;
SELECT server_name, count(*) executions FROM mcp_tool_executions GROUP BY 1 ORDER BY 2 DESC;
\echo == C17 can an execution be paired to an intent by (user, tool, time)? — what claim_unclaimed_intent should have done
SELECT count(*) executions,
  count(*) FILTER (WHERE EXISTS (SELECT 1 FROM ai_request_tool_calls t JOIN ai_requests r ON r.id = t.request_id
     WHERE r.user_id = e.user_id AND t.tool_name = 'mcp__' || e.server_name || '__' || e.tool_name
       AND r.created_at BETWEEN e.started_at - interval '10 min' AND e.started_at + interval '2 min')) pairable_by_user_tool_time,
  count(*) FILTER (WHERE EXISTS (SELECT 1 FROM ai_request_tool_calls t JOIN ai_requests r ON r.id = t.request_id
     WHERE r.session_id = e.session_id AND t.tool_name = 'mcp__' || e.server_name || '__' || e.tool_name)) pairable_by_session_tool
FROM mcp_tool_executions e;
\echo == C17 intent tool-name shapes vs execution tool-name shapes (namespace mismatch?)
SELECT left(tool_name, 6) prefix, count(*) FROM ai_request_tool_calls GROUP BY 1 ORDER BY 2 DESC LIMIT 6;
SELECT server_name, left(tool_name, 20) tool, count(*) FROM mcp_tool_executions GROUP BY 1,2 ORDER BY 3 DESC LIMIT 6;
\echo == C17 duplicate ai_tool_call_id among intents (ledger fan-out hazard)
SELECT count(*) dup_ids, sum(n) rows_involved FROM (SELECT ai_tool_call_id, count(*) n FROM ai_request_tool_calls GROUP BY 1 HAVING count(*) > 1) d;
\echo == C17 hook PostToolUse events with tool_use_id not matching any intent, by tool
SELECT tool_name, count(*) FROM plugin_usage_events p WHERE event_type LIKE 'PostToolUse%' AND NOT EXISTS (SELECT 1 FROM ai_request_tool_calls t WHERE t.ai_tool_call_id = p.tool_use_id) GROUP BY 1 ORDER BY 2 DESC LIMIT 8;
\echo == C17 per-day: intents / executions / hook tool events
SELECT d, (SELECT count(*) FROM ai_request_tool_calls t WHERE t.created_at::date = d) intents,
          (SELECT count(*) FROM mcp_tool_executions e WHERE e.started_at::date = d) executions,
          (SELECT count(*) FROM plugin_usage_events p WHERE p.event_type LIKE 'PostToolUse%' AND p.created_at::date = d) hook_tool_events
FROM generate_series(date '2026-09-03', date '2026-09-22', '1 day') d ORDER BY 1;
