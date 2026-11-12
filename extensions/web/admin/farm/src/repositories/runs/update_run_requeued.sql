UPDATE agent_farm_runs
SET status = 'queued', updated_at = NOW()
WHERE id = $1 AND status IN ('preparing', 'running')
