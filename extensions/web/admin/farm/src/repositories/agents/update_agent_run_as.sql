-- Sets one agent's run-as account; the sync never writes this column.
UPDATE agent_farm_agents
SET run_as_user_id = $3
WHERE farm_id = $1 AND id = $2
