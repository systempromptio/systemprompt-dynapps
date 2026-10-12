SELECT COUNT(*)::bigint AS "count!"
FROM agent_farm_agents
WHERE ($1::text IS NULL OR farm_id = $1)
