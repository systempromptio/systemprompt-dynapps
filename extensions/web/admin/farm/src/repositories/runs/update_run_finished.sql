UPDATE agent_farm_runs
SET status = $2,
    finished_at = NOW(),
    context_id = COALESCE($3, context_id),
    duration_ms = COALESCE($4, (EXTRACT(EPOCH FROM (NOW() - started_at)) * 1000)::bigint),
    num_turns = $5,
    input_tokens = $6,
    output_tokens = $7,
    cost_microdollars = $8,
    result_subtype = $9,
    exit_code = $10,
    error = $11,
    diff_summary = $12,
    pid = NULL,
    updated_at = NOW()
WHERE id = $1 AND status IN ('queued', 'preparing', 'running')
