UPDATE agent_farm_runs
SET runner_machine_id = $2, runner_region = $3, updated_at = NOW()
WHERE id = $1
