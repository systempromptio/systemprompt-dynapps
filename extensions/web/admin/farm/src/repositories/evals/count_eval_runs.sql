SELECT COUNT(*)::bigint AS "count!"
FROM agent_farm_eval_runs
WHERE ($1::text IS NULL OR farm_id = $1) AND ($2::text IS NULL OR suite_id = $2)
