SELECT
    COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
        AS "requests!",
    COUNT(*) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1)::BIGINT
        AS "prev_requests!",
    COALESCE(SUM(r.cost_microdollars)
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
        AS "cost!",
    COALESCE(SUM(r.cost_microdollars)
        FILTER (WHERE r.created_at >= $3 AND r.created_at < $1), 0)::BIGINT
        AS "prev_cost!",
    COUNT(DISTINCT r.user_id) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
        AS "active_users!",
    COUNT(DISTINCT r.user_id) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1)::BIGINT
        AS "prev_active_users!",
    COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms)
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
                  AND r.latency_ms IS NOT NULL), 0)::BIGINT
        AS "p50!",
    COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms)
        FILTER (WHERE r.created_at >= $3 AND r.created_at < $1
                  AND r.latency_ms IS NOT NULL), 0)::BIGINT
        AS "prev_p50!",
    COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms)
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
                  AND r.latency_ms IS NOT NULL), 0)::BIGINT
        AS "p95!",
    COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms)
        FILTER (WHERE r.created_at >= $3 AND r.created_at < $1
                  AND r.latency_ms IS NOT NULL), 0)::BIGINT
        AS "prev_p95!",
    COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
        AND r.status = 'failed')::BIGINT
        AS "errors!",
    COUNT(*) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1
        AND r.status = 'failed')::BIGINT
        AS "prev_errors!",
    COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
        AND r.status = 'rejected')::BIGINT
        AS "denied!",
    COUNT(*) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1
        AND r.status = 'rejected')::BIGINT
        AS "prev_denied!",
    COALESCE(SUM(COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0))
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
        AS "tokens!",
    COALESCE(SUM(COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0))
        FILTER (WHERE r.created_at >= $3 AND r.created_at < $1), 0)::BIGINT
        AS "prev_tokens!",
    COALESCE(SUM(COALESCE(r.input_tokens, 0))
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
        AS "input_tokens!",
    COALESCE(SUM(COALESCE(r.cache_read_tokens, 0))
        FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
        AS "cache_tokens!",
    COALESCE(SUM(COALESCE(r.cache_read_tokens, 0))
        FILTER (WHERE r.created_at >= $3 AND r.created_at < $1), 0)::BIGINT
        AS "prev_cache_tokens!",
    COUNT(DISTINCT r.context_id) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
        AS "conversations!",
    COUNT(DISTINCT r.context_id) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1)::BIGINT
        AS "prev_conversations!",
    (SELECT COUNT(*) FROM ai_request_tool_calls t JOIN ai_requests q ON q.id = t.request_id
      WHERE q.traffic = 'production' AND q.created_at >= $1 AND q.created_at < $2)::BIGINT
        AS "tool_calls!",
    (SELECT COUNT(*) FROM ai_request_tool_calls t JOIN ai_requests q ON q.id = t.request_id
      WHERE q.traffic = 'production' AND q.created_at >= $3 AND q.created_at < $1)::BIGINT
        AS "prev_tool_calls!"
FROM ai_requests r
WHERE r.traffic = 'production'
  AND r.created_at >= $3
  AND r.created_at < $2
