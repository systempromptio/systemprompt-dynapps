-- user_sessions counters vs recomputed values from ai_requests.
WITH r AS (
  SELECT session_id, count(*) AS n, sum(tokens_used) AS tok, sum(cost_microdollars) AS cost,
         count(*) FILTER (WHERE status <> 'completed') AS errs
  FROM ai_requests GROUP BY session_id)
SELECT count(*) AS sessions_with_requests,
       count(*) FILTER (WHERE s.ai_request_count <> r.n) AS request_count_drift,
       count(*) FILTER (WHERE s.total_tokens_used <> r.tok) AS tokens_drift,
       count(*) FILTER (WHERE s.total_ai_cost_microdollars <> r.cost) AS cost_drift,
       count(*) FILTER (WHERE s.error_count <> r.errs) AS error_drift
FROM r JOIN user_sessions s ON s.session_id = r.session_id;
