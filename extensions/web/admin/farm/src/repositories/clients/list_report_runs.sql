-- The most recent runs with their completion report and eval verdict.
-- $1 narrows to one client, $2 to one user's runs.
SELECT r.id AS "run_id!: FarmRunId",
       r.farm_id AS "farm_id!: FarmId",
       r.agent_id AS "agent_id!: FarmAgentId",
       r.client_id AS "client_id?: ClientId",
       c.name AS "client_name?",
       r.user_id AS "user_id!: UserId",
       r.status AS "status!",
       r.triggered_via AS "triggered_via!",
       r.cost_microdollars AS "cost_microdollars!",
       r.created_at AS "created_at!",
       r.finished_at AS "finished_at?",
       rr.completed AS "completed?",
       rr.summary AS "summary?",
       rr.eval_passed AS "eval_passed?",
       rr.eval_score AS "eval_score?"
FROM agent_farm_runs r
LEFT JOIN clients c ON c.id = r.client_id
LEFT JOIN agent_farm_run_reports rr ON rr.run_id = r.id
WHERE ($1::text IS NULL OR r.client_id = $1)
  AND ($2::text IS NULL OR r.user_id = $2)
ORDER BY r.created_at DESC
LIMIT $3
