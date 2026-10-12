UPDATE agent_farm_runs
SET status = 'running', started_at = NOW(), gateway_session_id = $2, api_key_id = $3,
    run_dir = $4, updated_at = NOW()
WHERE id = $1 AND status IN ('queued', 'preparing')
