-- The model calls of many gateway sessions added up per session, in one
-- query ($1 = the farm tasks' usage.gateway_session_id values).
SELECT session_id AS "gateway_session!",
       count(*)::bigint AS "calls!",
       COALESCE(sum(input_tokens), 0)::bigint AS "input_tokens!",
       COALESCE(sum(output_tokens), 0)::bigint AS "output_tokens!",
       COALESCE(sum(cache_read_tokens), 0)::bigint AS "cache_read_tokens!",
       COALESCE(sum(cache_creation_tokens), 0)::bigint AS "cache_creation_tokens!",
       COALESCE(sum(cost_microdollars), 0)::bigint AS "cost_microdollars!"
FROM ai_requests
WHERE session_id = ANY($1::text[])
GROUP BY session_id
