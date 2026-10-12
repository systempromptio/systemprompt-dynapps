UPDATE agent_farm_runs
SET status = 'running', started_at = COALESCE(started_at, NOW()), updated_at = NOW()
WHERE id = $1 AND status IN ('queued', 'preparing')
