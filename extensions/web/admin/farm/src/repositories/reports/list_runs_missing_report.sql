-- Terminal runs that never got a completion report: runs that finished
-- before reports existed, or whose observer stopped before writing one.
SELECT r.id AS "id!: FarmRunId"
FROM agent_farm_runs r
LEFT JOIN agent_farm_run_reports rr ON rr.run_id = r.id
WHERE rr.run_id IS NULL
  AND r.status IN ('succeeded', 'failed', 'cancelled', 'timed_out')
ORDER BY r.created_at
LIMIT $1
