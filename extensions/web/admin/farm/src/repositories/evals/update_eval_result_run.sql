UPDATE agent_farm_eval_results
SET run_id = $2, status = 'running', updated_at = NOW()
WHERE id = $1
