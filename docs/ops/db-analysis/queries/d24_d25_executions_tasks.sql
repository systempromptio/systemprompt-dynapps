-- D24: mcp_tool_executions; D25: agent_tasks / task_execution_steps
\echo == D24 execution anomalies
SELECT count(*) executions, count(*) FILTER (WHERE completed_at IS NULL AND started_at < now() - interval '1 hour') stuck,
       count(*) FILTER (WHERE status='pending') pending, count(*) FILTER (WHERE execution_time_ms IS NULL) no_duration,
       percentile_cont(0.5) WITHIN GROUP (ORDER BY execution_time_ms)::int p50_ms, percentile_cont(0.95) WITHIN GROUP (ORDER BY execution_time_ms)::int p95_ms, max(execution_time_ms) max_ms,
       count(*) FILTER (WHERE execution_time_ms > 60000) over_1min, count(*) FILTER (WHERE completed_at < started_at) completed_before_started
FROM mcp_tool_executions;
\echo == D24 server_name values vs services/mcp/*.yaml (checked offline: services/mcp/ has atlassian, github, knowledge-bank?, systemprompt …)
SELECT server_name, source, count(*), count(DISTINCT user_id) users FROM mcp_tool_executions GROUP BY 1,2 ORDER BY 3 DESC;
\echo == D24 error shapes
SELECT server_name, left(regexp_replace(error_message, '[0-9]+', '#', 'g'), 100) shape, count(*) FROM mcp_tool_executions WHERE status='failed' GROUP BY 1,2 ORDER BY 3 DESC LIMIT 10;
\echo == D24 request_method / actor
SELECT request_method, request_source, actor_kind, count(*) FROM mcp_tool_executions GROUP BY 1,2,3 ORDER BY 4 DESC;
\echo == D25 agent tasks (this instance ships no A2A agents)
SELECT (SELECT count(*) FROM agent_tasks) tasks, (SELECT count(*) FROM task_execution_steps) steps, (SELECT count(*) FROM task_messages) messages,
       (SELECT count(*) FROM task_execution_steps s WHERE NOT EXISTS (SELECT 1 FROM agent_tasks t WHERE t.task_id = s.task_id)) steps_without_task;
