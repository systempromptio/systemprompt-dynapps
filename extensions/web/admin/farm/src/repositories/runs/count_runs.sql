SELECT COUNT(*)::bigint AS "count!"
FROM agent_farm_runs
WHERE ($1::text IS NULL OR farm_id = $1)
  AND ($2::text IS NULL OR agent_id = $2)
  AND ($3::text IS NULL OR user_id = $3)
  AND ($4::text IS NULL OR status = $4)
  AND ($5::text IS NULL OR triggered_via = $5)
  AND ($6::timestamptz IS NULL OR created_at >= $6)
  AND ($7::timestamptz IS NULL OR created_at < $7)
