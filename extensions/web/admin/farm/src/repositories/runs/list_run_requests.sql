SELECT request_id, model, latency_ms, input_tokens, output_tokens, cost_microdollars, created_at
FROM ai_requests
WHERE session_id = $1
ORDER BY created_at, id
