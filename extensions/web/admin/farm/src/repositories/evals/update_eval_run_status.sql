UPDATE agent_farm_eval_runs
SET status = $2,
    started_at = CASE WHEN $2 = 'running' THEN COALESCE(started_at, NOW()) ELSE started_at END
WHERE id = $1 AND status IN ('queued', 'running')
