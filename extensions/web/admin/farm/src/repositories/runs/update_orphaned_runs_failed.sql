UPDATE agent_farm_runs
SET status = 'failed',
    finished_at = NOW(),
    error = COALESCE(error, 'The server restarted while this run was in flight'),
    pid = NULL,
    updated_at = NOW()
WHERE status IN ('preparing', 'running')
