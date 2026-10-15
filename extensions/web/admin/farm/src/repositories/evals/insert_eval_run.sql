INSERT INTO agent_farm_eval_runs (id, suite_id, farm_id, agent_id, user_id, triggered_via,
                                  client_id)
VALUES ($1, $2, $3, $4, $5, $6, (SELECT client_id FROM agent_farms WHERE id = $3))
RETURNING id AS "id: EvalRunId", suite_id AS "suite_id: EvalSuiteId", farm_id AS "farm_id: FarmId",
       agent_id AS "agent_id: FarmAgentId", user_id AS "user_id: UserId", status, started_at,
       finished_at, passed, failed, errored, mean_score, cost_microdollars, created_at,
       triggered_via
