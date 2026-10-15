SELECT farm_id AS "farm_id!: FarmId", agent_id AS "agent_id!: FarmAgentId",
       (COALESCE(finished_at, created_at) AT TIME ZONE 'UTC')::date AS "day!",
       COUNT(*)::bigint AS "runs!",
       COALESCE(SUM(passed), 0)::bigint AS "passed!",
       COALESCE(SUM(passed + failed + errored), 0)::bigint AS "decided!",
       AVG(mean_score) AS mean_score,
       COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars!"
FROM agent_farm_eval_runs
WHERE ($1::text IS NULL OR farm_id = $1)
  AND ($2::text IS NULL OR agent_id = $2)
  AND created_at >= $3
  AND status IN ('completed', 'cancelled')
GROUP BY farm_id, agent_id, 3
ORDER BY farm_id, agent_id, 3
