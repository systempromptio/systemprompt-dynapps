DELETE FROM agent_farm_fixtures
WHERE farm_id = $1 AND NOT (name = ANY($2::text[]))
