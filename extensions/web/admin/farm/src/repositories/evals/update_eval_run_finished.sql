UPDATE agent_farm_eval_runs
SET status = $2, mean_score = $3, cost_microdollars = $4, finished_at = NOW()
WHERE id = $1
