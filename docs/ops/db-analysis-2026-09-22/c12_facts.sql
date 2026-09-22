BEGIN;
SELECT md5(string_agg(f::text, '' ORDER BY context_id)) before_hash, count(*) FROM conversation_facts f \gset
CREATE TEMP TABLE _facts_before AS SELECT * FROM conversation_facts;
SELECT refresh_conversation_facts(NULL) AS rows_written;
SELECT count(*) after_rows, (SELECT count(*) FROM _facts_before) before_rows,
       (SELECT count(*) FROM conversation_facts n LEFT JOIN _facts_before b USING (context_id) WHERE b.context_id IS NULL) new_contexts,
       (SELECT count(*) FROM _facts_before b LEFT JOIN conversation_facts n USING (context_id) WHERE n.context_id IS NULL) gone_contexts
FROM conversation_facts;
SELECT col, count(*) FILTER (WHERE differs) n_diff, round(100.0*count(*) FILTER (WHERE differs)/count(*),1) pct, (array_agg(context_id) FILTER (WHERE differs))[1:3] examples FROM (
 SELECT b.context_id, 'request_count' col, b.request_count IS DISTINCT FROM n.request_count differs FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'turn_count', b.turn_count IS DISTINCT FROM n.turn_count FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'cost_microdollars', b.cost_microdollars IS DISTINCT FROM n.cost_microdollars FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'input_tokens', b.input_tokens IS DISTINCT FROM n.input_tokens FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'output_tokens', b.output_tokens IS DISTINCT FROM n.output_tokens FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'first_at', b.first_at IS DISTINCT FROM n.first_at FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'last_at', b.last_at IS DISTINCT FROM n.last_at FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'tool_calls_intended', b.tool_calls_intended IS DISTINCT FROM n.tool_calls_intended FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'tool_calls_executed', b.tool_calls_executed IS DISTINCT FROM n.tool_calls_executed FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'gov_allow', b.gov_allow IS DISTINCT FROM n.gov_allow FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'hook_event_count', b.hook_event_count IS DISTINCT FROM n.hook_event_count FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'skill_invocations', b.skill_invocations IS DISTINCT FROM n.skill_invocations FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'error_count', b.error_count IS DISTINCT FROM n.error_count FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'group_id', b.group_id IS DISTINCT FROM n.group_id FROM _facts_before b JOIN conversation_facts n USING (context_id)
 UNION ALL SELECT b.context_id, 'safety_findings', b.safety_findings IS DISTINCT FROM n.safety_findings FROM _facts_before b JOIN conversation_facts n USING (context_id)
) d GROUP BY col ORDER BY n_diff DESC;
SELECT b.context_id, b.last_at b_last, n.last_at n_last, b.request_count b_rc, n.request_count n_rc, b.tool_calls_executed b_tx, n.tool_calls_executed n_tx, b.gov_allow b_ga, n.gov_allow n_ga FROM _facts_before b JOIN conversation_facts n USING (context_id) WHERE b.last_at IS DISTINCT FROM n.last_at OR b.request_count<>n.request_count OR b.tool_calls_executed<>n.tool_calls_executed LIMIT 5;
-- stale-ness: last request vs refreshed_at
SELECT count(*) FILTER (WHERE b.refreshed_at < r.max_created) stale_facts, count(*) total FROM _facts_before b JOIN (SELECT context_id, max(created_at) max_created FROM ai_requests GROUP BY 1) r USING (context_id);
-- overall: tool calls intended vs executed vs mcp_tool_executions rows
SELECT sum(tool_calls_intended) intended, sum(tool_calls_executed) executed, (SELECT count(*) FROM ai_request_tool_calls) tool_call_rows, (SELECT count(*) FROM mcp_tool_executions) mcp_exec_rows, sum(gov_allow) gov_allow, (SELECT count(*) FROM governance_decisions WHERE decision='allow') gov_allow_rows FROM conversation_facts;
ROLLBACK;
SELECT md5(string_agg(f::text, '' ORDER BY context_id)) = :'before_hash' AS unchanged_after_rollback FROM conversation_facts f;
