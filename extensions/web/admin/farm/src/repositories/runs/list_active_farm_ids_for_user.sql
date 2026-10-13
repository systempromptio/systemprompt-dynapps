SELECT DISTINCT r.farm_id AS "farm_id!"
FROM agent_farm_runs r
JOIN agent_farms f ON f.id = r.farm_id
WHERE (r.user_id = $1 OR f.run_as_user_id = $1)
  AND r.status IN ('queued', 'preparing', 'running')
ORDER BY r.farm_id
