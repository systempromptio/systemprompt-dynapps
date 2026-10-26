-- The overview's time series: every per-bucket figure the landing page
-- charts, on one spine of $3 equal buckets between $1 and $2, each $4 seconds
-- wide. The request aggregate, the tool-call count and the governance
-- denials are three CTEs joined onto the spine, so a quiet bucket comes back
-- as zeros rather than vanishing and shifting every later point left.
WITH spine AS (
    SELECT generate_series(0, $3::INT - 1) AS bucket
), req AS (
    SELECT LEAST($3::INT - 1,
                 FLOOR(EXTRACT(EPOCH FROM (r.created_at - $1)) / $4::FLOAT8)::INT) AS bucket,
           r.id, r.user_id, r.context_id, r.cost_microdollars, r.status, r.latency_ms,
           COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0) AS tokens,
           COALESCE(r.cache_read_tokens, 0) AS cache_tokens
    FROM ai_requests r
    WHERE r.traffic = 'production'
      AND r.created_at >= $1
      AND r.created_at < $2
), agg AS (
    SELECT bucket,
           COUNT(*)::BIGINT AS requests,
           COUNT(DISTINCT user_id)::BIGINT AS users,
           COUNT(DISTINCT context_id)::BIGINT AS conversations,
           COALESCE(SUM(cost_microdollars), 0)::BIGINT AS cost,
           COALESCE(SUM(tokens), 0)::BIGINT AS tokens,
           COALESCE(SUM(cache_tokens), 0)::BIGINT AS cache_tokens,
           COUNT(*) FILTER (WHERE status = 'failed')::BIGINT AS errors,
           COUNT(*) FILTER (WHERE status = 'rejected')::BIGINT AS rejected,
           COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY latency_ms)
               FILTER (WHERE latency_ms IS NOT NULL), 0)::BIGINT AS p50,
           COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms)
               FILTER (WHERE latency_ms IS NOT NULL), 0)::BIGINT AS p95
    FROM req
    GROUP BY bucket
), tools AS (
    SELECT q.bucket, COUNT(*)::BIGINT AS tool_calls
    FROM req q
    JOIN ai_request_tool_calls t ON t.request_id = q.id
    GROUP BY q.bucket
), denials AS (
    SELECT LEAST($3::INT - 1,
                 FLOOR(EXTRACT(EPOCH FROM (d.created_at - $1)) / $4::FLOAT8)::INT) AS bucket,
           COUNT(*)::BIGINT AS denied
    FROM governance_decisions d
    WHERE d.decision = 'deny'
      AND d.created_at >= $1
      AND d.created_at < $2
    GROUP BY 1
)
SELECT spine.bucket AS "bucket!",
       COALESCE(agg.requests, 0)::BIGINT AS "requests!",
       COALESCE(agg.users, 0)::BIGINT AS "users!",
       COALESCE(agg.conversations, 0)::BIGINT AS "conversations!",
       COALESCE(agg.cost, 0)::BIGINT AS "cost!",
       COALESCE(agg.tokens, 0)::BIGINT AS "tokens!",
       COALESCE(agg.cache_tokens, 0)::BIGINT AS "cache_tokens!",
       COALESCE(agg.errors, 0)::BIGINT AS "errors!",
       COALESCE(agg.rejected, 0)::BIGINT AS "rejected!",
       COALESCE(agg.p50, 0)::BIGINT AS "p50!",
       COALESCE(agg.p95, 0)::BIGINT AS "p95!",
       COALESCE(tools.tool_calls, 0)::BIGINT AS "tool_calls!",
       COALESCE(denials.denied, 0)::BIGINT AS "denied!"
FROM spine
LEFT JOIN agg ON agg.bucket = spine.bucket
LEFT JOIN tools ON tools.bucket = spine.bucket
LEFT JOIN denials ON denials.bucket = spine.bucket
ORDER BY spine.bucket
