-- C16: governance_decisions vs ai_requests / mcp_tool_executions; scope coverage
\echo == C16 decisions by policy/decision
SELECT policy, decision, count(*), count(*) FILTER (WHERE trace_id IS NULL) no_trace, count(*) FILTER (WHERE tool_use_id IS NOT NULL) has_tool_use, count(DISTINCT trace_id) traces FROM governance_decisions GROUP BY 1,2 ORDER BY 3 DESC;
\echo == C16 authz vs authz_rule_based: identical pairs (same trace, tool, second)
SELECT count(*) AS pairs,
       count(*) FILTER (WHERE a.decision = b.decision AND a.reason IS NOT DISTINCT FROM b.reason) AS same_decision_and_reason,
       count(*) FILTER (WHERE a.evaluated_rules = b.evaluated_rules) AS same_rules
FROM governance_decisions a JOIN governance_decisions b ON a.trace_id = b.trace_id AND a.tool_name = b.tool_name
 AND a.policy = 'authz' AND b.policy = 'authz_rule_based' AND abs(extract(epoch FROM a.created_at - b.created_at)) < 1;
\echo == C16 the `authz` rows: what distinguishes them from authz_rule_based (reason shapes, agent_scope)
SELECT policy, agent_scope, left(reason,60) reason, count(*) FROM governance_decisions WHERE policy IN ('authz','authz_rule_based') GROUP BY 1,2,3 ORDER BY 4 DESC LIMIT 8;
\echo == C16 per-trace stage multiplicity for traced decisions
SELECT stages, count(*) traces FROM (SELECT trace_id, string_agg(policy, '+' ORDER BY policy) stages FROM governance_decisions WHERE trace_id IS NOT NULL GROUP BY 1) x GROUP BY 1 ORDER BY 2 DESC;
\echo == C16 decisions whose trace joins a request / an execution / neither
SELECT count(*) total,
  count(*) FILTER (WHERE EXISTS (SELECT 1 FROM ai_requests r WHERE r.trace_id = g.trace_id)) on_request,
  count(*) FILTER (WHERE EXISTS (SELECT 1 FROM mcp_tool_executions e WHERE e.trace_id = g.trace_id)) on_execution,
  count(*) FILTER (WHERE trace_id IS NULL) no_trace,
  count(*) FILTER (WHERE trace_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM ai_requests r WHERE r.trace_id = g.trace_id) AND NOT EXISTS (SELECT 1 FROM mcp_tool_executions e WHERE e.trace_id = g.trace_id)) traced_but_unjoined
FROM governance_decisions g;
\echo == C16 requests without any decision (should be 0: every /v1/messages passes the prompt gate)
SELECT count(*) requests, count(*) FILTER (WHERE NOT EXISTS (SELECT 1 FROM governance_decisions g WHERE g.trace_id = r.trace_id)) without_decision,
       count(*) FILTER (WHERE status = 'rejected') rejected FROM ai_requests r;
SELECT status, request_kind, count(*) FROM ai_requests r WHERE NOT EXISTS (SELECT 1 FROM governance_decisions g WHERE g.trace_id = r.trace_id) GROUP BY 1,2;
\echo == C16 executions without a decision
SELECT count(*) FROM mcp_tool_executions e WHERE NOT EXISTS (SELECT 1 FROM governance_decisions g WHERE g.trace_id = e.trace_id);
\echo == C16 hook (no-trace) decisions: session id namespace and what they carry
SELECT left(tool_name, 14) tool, count(*), count(*) FILTER (WHERE session_id LIKE 'sess\_%') gateway_session, count(*) FILTER (WHERE context_id IS NULL) ctx_null, count(*) FILTER (WHERE plugin_id IS NULL) plugin_null, count(*) FILTER (WHERE act_chain IS NULL) act_null
FROM governance_decisions WHERE trace_id IS NULL GROUP BY 1 ORDER BY 2 DESC LIMIT 10;
\echo == C16 hook decisions vs PostToolUse events: same CC session, tool_use available on the event but not on the decision
SELECT (SELECT count(*) FROM governance_decisions WHERE trace_id IS NULL) hook_decisions,
       (SELECT count(*) FROM plugin_usage_events WHERE event_type IN ('PostToolUse','PostToolUseFailure')) post_tool_events,
       (SELECT count(*) FROM governance_decisions g WHERE trace_id IS NULL AND EXISTS (SELECT 1 FROM plugin_usage_events p WHERE p.session_id = g.session_id AND p.tool_name = g.tool_name AND p.event_type LIKE 'PostToolUse%' AND p.created_at BETWEEN g.created_at AND g.created_at + interval '10 min')) paired_by_session_tool_time;
\echo == C16 decisions per day: with trace / without
SELECT created_at::date d, count(*) total, count(*) FILTER (WHERE trace_id IS NULL) hook_no_trace, count(*) FILTER (WHERE policy='authz') authz_dup FROM governance_decisions GROUP BY 1 ORDER BY 1;
\echo == C16 ai_request_scopes coverage and scope validity
SELECT (SELECT count(*) FROM ai_requests) requests, (SELECT count(*) FROM ai_request_scopes) scopes,
       (SELECT count(*) FROM ai_requests r WHERE NOT EXISTS (SELECT 1 FROM ai_request_scopes s WHERE s.request_id = r.id)) uncovered,
       (SELECT count(*) FROM ai_request_scopes WHERE group_id IS NULL) no_group, (SELECT count(*) FROM ai_request_scopes WHERE project_id IS NULL) no_project,
       (SELECT string_agg(source||'='||n, ', ') FROM (SELECT source, count(*) n FROM ai_request_scopes GROUP BY 1) s) sources;
