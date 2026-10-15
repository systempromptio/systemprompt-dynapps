-- Sets one agent's run-as mode and account; the sync never writes these.
UPDATE agent_farm_agents
SET run_as_user_id = $3, run_as_mode = $4
WHERE farm_id = $1 AND id = $2
