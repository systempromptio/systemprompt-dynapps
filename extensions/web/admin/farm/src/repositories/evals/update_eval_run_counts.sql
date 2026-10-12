UPDATE agent_farm_eval_runs
SET passed = $2, failed = $3, errored = $4
WHERE id = $1
