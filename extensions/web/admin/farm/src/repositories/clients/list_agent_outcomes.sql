-- Per farm agent over the window: runs, outcomes, cost, completion and the
-- mean eval score. $1 narrows to one client, $2 to one user's runs.
SELECT r.farm_id AS "farm_id!: FarmId",
       r.agent_id AS "agent_id!: FarmAgentId",
       COUNT(*)::bigint AS "runs!",
       COUNT(*) FILTER (WHERE r.status = 'succeeded')::bigint AS "succeeded!",
       COUNT(*) FILTER (WHERE r.status IN ('succeeded', 'failed', 'timed_out'))::bigint AS "finished!",
       COALESCE(SUM(r.cost_microdollars), 0)::bigint AS "cost_microdollars!",
       COUNT(rr.run_id) FILTER (WHERE rr.completed)::bigint AS "completed!",
       (SELECT AVG(e.mean_score)
          FROM agent_farm_eval_runs e
         WHERE e.farm_id = r.farm_id AND e.agent_id = r.agent_id
           AND ($1::text IS NULL OR e.client_id = $1)
           AND ($2::text IS NULL OR e.user_id = $2)
           AND e.mean_score IS NOT NULL
           AND e.created_at >= $3)::float8 AS "eval_mean?",
       MAX(r.created_at) AS "last_run_at!"
FROM agent_farm_runs r
LEFT JOIN agent_farm_run_reports rr ON rr.run_id = r.id
WHERE r.created_at >= $3
  AND ($1::text IS NULL OR r.client_id = $1)
  AND ($2::text IS NULL OR r.user_id = $2)
GROUP BY r.farm_id, r.agent_id
ORDER BY COUNT(*) DESC, r.farm_id, r.agent_id
