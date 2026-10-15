SELECT runner_machine_id, runner_region
FROM agent_farm_runs
WHERE id = $1
