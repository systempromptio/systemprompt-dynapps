UPDATE agent_farm_runs
SET status = 'preparing', updated_at = NOW()
WHERE id = $1 AND status = 'queued' AND cancel_requested_at IS NULL
