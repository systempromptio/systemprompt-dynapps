-- C10: user_sessions counters vs recomputation from ai_requests / agent_tasks / task_messages
\echo == C10 sessions with ai_requests: stored counter vs recomputed
WITH a AS (SELECT session_id, count(*) n, sum(tokens_used) t, sum(cost_microdollars) c, count(*) FILTER (WHERE status <> 'completed') e FROM ai_requests GROUP BY 1)
SELECT count(*) AS sessions_with_requests,
       count(*) FILTER (WHERE s.ai_request_count <> a.n) AS ai_request_count_wrong,
       count(*) FILTER (WHERE coalesce(s.total_tokens_used,0) <> coalesce(a.t,0)) AS total_tokens_wrong,
       count(*) FILTER (WHERE coalesce(s.total_ai_cost_microdollars,0) <> coalesce(a.c,0)) AS total_cost_wrong,
       count(*) FILTER (WHERE coalesce(s.error_count,0) <> a.e) AS error_count_wrong,
       sum(a.n) AS actual_requests, sum(s.ai_request_count) AS stored_requests,
       sum(a.t) AS actual_tokens, sum(s.total_tokens_used) AS stored_tokens,
       sum(a.c) AS actual_cost_ud, sum(s.total_ai_cost_microdollars) AS stored_cost_ud
FROM user_sessions s JOIN a ON a.session_id = s.session_id;
\echo == C10 distribution of stored ai counters across all sessions
SELECT count(*) sessions, count(*) FILTER (WHERE ai_request_count > 0) ai_count_nonzero, count(*) FILTER (WHERE total_tokens_used > 0) tokens_nonzero,
       count(*) FILTER (WHERE total_ai_cost_microdollars > 0) cost_nonzero, count(*) FILTER (WHERE error_count > 0) error_nonzero,
       count(*) FILTER (WHERE request_count > 0) http_request_nonzero, count(*) FILTER (WHERE task_count > 0) task_nonzero, count(*) FILTER (WHERE message_count > 0) message_nonzero
FROM user_sessions;
\echo == C10 three example sessions (ids only)
WITH a AS (SELECT session_id, count(*) n, sum(tokens_used) t FROM ai_requests GROUP BY 1)
SELECT s.session_id, s.ai_request_count AS stored, a.n AS actual, s.total_tokens_used AS stored_tokens, a.t AS actual_tokens, s.session_source, s.client_type
FROM user_sessions s JOIN a USING (session_id) ORDER BY a.n DESC LIMIT 3;
\echo == C10 agent_tasks / task_messages (the task_count and message_count sources)
SELECT (SELECT count(*) FROM agent_tasks) agent_tasks, (SELECT count(*) FROM task_messages) task_messages, (SELECT sum(task_count) FROM user_sessions) stored_task_count, (SELECT sum(message_count) FROM user_sessions) stored_message_count;
\echo == C10 request_count (HTTP) vs analytics_events http_request rows per session
WITH h AS (SELECT session_id, count(*) n FROM analytics_events GROUP BY 1)
SELECT count(*) sessions, count(*) FILTER (WHERE s.request_count = h.n) exact_match, count(*) FILTER (WHERE s.request_count > h.n) stored_higher, count(*) FILTER (WHERE s.request_count < h.n) stored_lower
FROM user_sessions s JOIN h USING (session_id);
