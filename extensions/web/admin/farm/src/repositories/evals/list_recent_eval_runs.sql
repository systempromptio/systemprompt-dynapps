SELECT id AS "id: EvalRunId", suite_id AS "suite_id: EvalSuiteId", farm_id AS "farm_id: FarmId",
       agent_id AS "agent_id: FarmAgentId", user_id AS "user_id: UserId", status, started_at,
       finished_at, passed, failed, errored, mean_score, cost_microdollars, created_at,
       triggered_via
FROM agent_farm_eval_runs
WHERE ($1::text IS NULL OR farm_id = $1) AND ($2::text IS NULL OR suite_id = $2)
ORDER BY created_at DESC, id
LIMIT $3
