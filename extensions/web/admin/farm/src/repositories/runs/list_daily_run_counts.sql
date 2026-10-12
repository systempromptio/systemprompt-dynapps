SELECT (created_at AT TIME ZONE 'UTC')::date AS "day!", COUNT(*)::bigint AS "runs!"
FROM agent_farm_runs
WHERE created_at >= $1
GROUP BY 1
ORDER BY 1
