SELECT DISTINCT farm_id AS "farm_id!"
FROM agent_farm_runs
WHERE user_id = $1 AND status IN ('queued', 'preparing', 'running')
ORDER BY farm_id
