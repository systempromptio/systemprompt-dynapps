SELECT run_id AS "run_id: FarmRunId", seq, kind, tool_name, tool_use_id, payload, created_at
FROM agent_farm_run_events
WHERE run_id = $1 AND seq > $2
ORDER BY seq
LIMIT $3
