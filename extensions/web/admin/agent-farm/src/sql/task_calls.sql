-- The model calls of one farm task: every ai_requests row of its gateway
-- session ($1 = usage.gateway_session_id), oldest first, at most $2.
SELECT id,
       created_at,
       model,
       status,
       request_kind,
       input_tokens,
       output_tokens,
       cache_read_tokens,
       cache_creation_tokens,
       cost_microdollars,
       latency_ms
FROM ai_requests
WHERE session_id = $1
ORDER BY created_at, id
LIMIT $2
