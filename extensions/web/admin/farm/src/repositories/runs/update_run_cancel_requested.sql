-- A queued run has no process to signal, so it is cancelled outright; a
-- preparing or running one is flagged for the runner to kill.
UPDATE agent_farm_runs
SET cancel_requested_at = COALESCE(cancel_requested_at, NOW()),
    status = CASE WHEN status = 'queued' THEN 'cancelled' ELSE status END,
    finished_at = CASE WHEN status = 'queued' THEN NOW() ELSE finished_at END,
    updated_at = NOW()
WHERE id = $1 AND status IN ('queued', 'preparing', 'running')
