-- The model calls of one farm task added up per model ($1 = gateway session).
SELECT model,
       count(*)::bigint AS "calls!",
       count(*) FILTER (WHERE status = 'failed')::bigint AS "failed!",
       COALESCE(sum(input_tokens), 0)::bigint AS "input_tokens!",
       COALESCE(sum(output_tokens), 0)::bigint AS "output_tokens!",
       COALESCE(sum(cache_read_tokens), 0)::bigint AS "cache_read_tokens!",
       COALESCE(sum(cache_creation_tokens), 0)::bigint AS "cache_creation_tokens!",
       COALESCE(sum(cost_microdollars), 0)::bigint AS "cost_microdollars!"
FROM ai_requests
WHERE session_id = $1
GROUP BY model
ORDER BY 8 DESC, model
