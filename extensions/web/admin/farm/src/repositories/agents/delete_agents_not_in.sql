DELETE FROM agent_farm_agents
WHERE farm_id = $1 AND NOT (id = ANY($2::text[]))
