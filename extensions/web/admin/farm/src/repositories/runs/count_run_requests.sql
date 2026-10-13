SELECT COUNT(*)::bigint AS "count!"
FROM ai_requests
WHERE api_key_id = $1
