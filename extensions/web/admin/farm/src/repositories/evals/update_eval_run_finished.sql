UPDATE agent_farm_eval_runs
SET status = CASE WHEN status = 'cancelled' THEN status ELSE $2 END,
    mean_score = $3, cost_microdollars = $4, finished_at = NOW()
WHERE id = $1 AND finished_at IS NULL
  AND status IN ('queued', 'running', 'cancelled')
