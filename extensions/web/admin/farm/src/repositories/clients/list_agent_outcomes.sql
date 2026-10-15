-- Per farm agent over the window: runs, outcomes, cost, completion and the
-- mean eval score of its runs, infrastructure failures left out of the mean
-- and counted separately. $1 narrows to one client, $2 to one user's runs.
SELECT r.farm_id AS "farm_id!: FarmId",
       r.agent_id AS "agent_id!: FarmAgentId",
       COUNT(*)::bigint AS "runs!",
       COUNT(*) FILTER (WHERE r.status = 'succeeded')::bigint AS "succeeded!",
       COUNT(*) FILTER (WHERE r.status IN ('succeeded', 'failed', 'timed_out'))::bigint AS "finished!",
       COALESCE(SUM(r.cost_microdollars), 0)::bigint AS "cost_microdollars!",
       COUNT(rr.run_id) FILTER (WHERE rr.completed)::bigint AS "completed!",
       AVG(s.score)::float8 AS "eval_mean?",
       COUNT(s.score)::bigint AS "eval_scored!",
       COUNT(*) FILTER (WHERE s.infra_failure)::bigint AS "infra_failures!",
       MAX(r.created_at) AS "last_run_at!"
FROM agent_farm_runs r
LEFT JOIN agent_farm_run_reports rr ON rr.run_id = r.id
LEFT JOIN agent_farm_run_eval_scores s ON s.run_id = r.id
WHERE r.created_at >= $3
  AND ($1::text IS NULL OR r.client_id = $1)
  AND ($2::text IS NULL OR r.user_id = $2)
GROUP BY r.farm_id, r.agent_id
ORDER BY COUNT(*) DESC, r.farm_id, r.agent_id
