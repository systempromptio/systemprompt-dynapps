SELECT id AS "id: EvalRunId", suite_id AS "suite_id: EvalSuiteId", farm_id AS "farm_id: FarmId",
       agent_id AS "agent_id: FarmAgentId", user_id AS "user_id: UserId", status, started_at,
       finished_at, passed, failed, errored, mean_score, cost_microdollars, created_at
FROM agent_farm_eval_runs
WHERE id = $1
