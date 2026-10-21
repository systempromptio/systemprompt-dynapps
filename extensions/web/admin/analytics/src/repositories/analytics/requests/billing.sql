SELECT
    COALESCE(SUM(COALESCE(ar.cost_microdollars, 0)), 0)::bigint AS "user_key_cost_microdollars!",
    COUNT(*)::bigint AS "user_key_requests!"
FROM ai_requests ar
LEFT JOIN user_scope_defaults sd ON sd.user_id = ar.user_id
WHERE ar.created_at >= $1 AND ar.created_at < $2
  AND ar.billing_source = 'user_key'
  AND ($8::TEXT[] IS NULL OR ar.user_id = ANY($8))
  AND ($3::text IS NULL OR ar.user_id = $3)
  AND ($4::text IS NULL
       OR ($4 = 'unrouted' AND ar.model IS NULL)
       OR ar.model = $4)
  AND ($5::text IS NULL
       OR ($5 = 'unrouted' AND ar.provider IS NULL)
       OR ar.provider = $5)
  AND ($6::text IS NULL OR ar.status = $6)
  AND ($9::text IS NULL OR sd.primary_group_id = $9)
  AND ($10::text IS NULL OR sd.primary_project_id = $10)
  AND ($7::text IS NULL
       OR ar.user_id ILIKE $7
       OR ar.model ILIKE $7
       OR ar.provider ILIKE $7
       OR COALESCE(ar.error_message, '') ILIKE $7
       OR COALESCE(ar.trace_id, '') ILIKE $7)
