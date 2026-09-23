-- C10: user_sessions counters vs recomputation from ai_requests / agent_tasks / task_messages
WITH src AS (
  SELECT s.session_id, s.ai_request_count, s.total_tokens_used, s.total_ai_cost_microdollars, s.request_count, s.task_count, s.message_count, s.error_count,
         COALESCE(r.n,0) r_n, COALESCE(r.tok,0) r_tok, COALESCE(r.cost,0) r_cost, COALESCE(r.err,0) r_err, COALESCE(t.n,0) t_n, COALESCE(m.n,0) m_n
  FROM user_sessions s
  LEFT JOIN (SELECT session_id, count(*) n, sum(tokens_used) tok, sum(cost_microdollars) cost, count(*) FILTER (WHERE status='failed') err FROM ai_requests GROUP BY 1) r USING (session_id)
  LEFT JOIN (SELECT session_id, count(*) n FROM agent_tasks GROUP BY 1) t USING (session_id)
  LEFT JOIN (SELECT t.session_id, count(*) n FROM task_messages m JOIN agent_tasks t USING (task_id) GROUP BY 1) m USING (session_id))
SELECT 'ai_request_count' counter, count(*) FILTER (WHERE ai_request_count IS DISTINCT FROM r_n) mismatched, count(*) sessions, count(*) FILTER (WHERE r_n>0) sessions_with_requests,
       sum(ai_request_count) stored_sum, sum(r_n) actual_sum, (array_agg(session_id) FILTER (WHERE ai_request_count IS DISTINCT FROM r_n))[1:3] examples FROM src
UNION ALL SELECT 'total_tokens_used', count(*) FILTER (WHERE total_tokens_used IS DISTINCT FROM r_tok), count(*), count(*) FILTER (WHERE r_n>0), sum(total_tokens_used), sum(r_tok), (array_agg(session_id) FILTER (WHERE total_tokens_used IS DISTINCT FROM r_tok))[1:3] FROM src
UNION ALL SELECT 'total_ai_cost_microdollars', count(*) FILTER (WHERE total_ai_cost_microdollars IS DISTINCT FROM r_cost), count(*), count(*) FILTER (WHERE r_n>0), sum(total_ai_cost_microdollars), sum(r_cost), (array_agg(session_id) FILTER (WHERE total_ai_cost_microdollars IS DISTINCT FROM r_cost))[1:3] FROM src
UNION ALL SELECT 'error_count', count(*) FILTER (WHERE error_count IS DISTINCT FROM r_err), count(*), count(*) FILTER (WHERE r_n>0), sum(error_count), sum(r_err), (array_agg(session_id) FILTER (WHERE error_count IS DISTINCT FROM r_err))[1:3] FROM src
UNION ALL SELECT 'task_count', count(*) FILTER (WHERE task_count IS DISTINCT FROM t_n), count(*), count(*) FILTER (WHERE r_n>0), sum(task_count), sum(t_n), (array_agg(session_id) FILTER (WHERE task_count IS DISTINCT FROM t_n))[1:3] FROM src
UNION ALL SELECT 'message_count', count(*) FILTER (WHERE message_count IS DISTINCT FROM m_n), count(*), count(*) FILTER (WHERE r_n>0), sum(message_count), sum(m_n), (array_agg(session_id) FILTER (WHERE message_count IS DISTINCT FROM m_n))[1:3] FROM src;
-- request_count semantics: HTTP hits? compare to ai requests and analytics_events
SELECT sum(request_count) request_count_sum, (SELECT count(*) FROM analytics_events) analytics_events, (SELECT count(*) FROM ai_requests) ai_requests FROM user_sessions;
-- how far off are the mismatched ones (direction)
SELECT sign(s.ai_request_count - COALESCE(r.n,0)) dir, count(*), sum(abs(s.ai_request_count - COALESCE(r.n,0))) abs_delta FROM user_sessions s LEFT JOIN (SELECT session_id, count(*) n FROM ai_requests GROUP BY 1) r USING (session_id) GROUP BY 1;
SELECT sign(s.total_ai_cost_microdollars - COALESCE(r.c,0)) dir, count(*), sum(abs(s.total_ai_cost_microdollars - COALESCE(r.c,0))) abs_delta_micro FROM user_sessions s LEFT JOIN (SELECT session_id, sum(cost_microdollars) c FROM ai_requests GROUP BY 1) r USING (session_id) GROUP BY 1;
