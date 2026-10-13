UPDATE agent_farm_runs
SET status = $2,
    error_class = $3,
    cost_microdollars = $4,
    event_count = $5,
    tool_call_count = $6,
    gateway_request_count = $7,
    context_id = COALESCE($8, context_id),
    attempt = $9,
    finished_at = NOW(),
    updated_at = NOW()
WHERE id = $1 AND status IN ('queued', 'preparing', 'running')
