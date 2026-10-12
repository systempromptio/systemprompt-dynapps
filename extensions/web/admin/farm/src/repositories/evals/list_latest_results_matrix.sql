-- The latest decided result for every (suite, case): one cell of the
-- suite-by-case matrix. Pending and running results are not a verdict.
SELECT DISTINCT ON (er.suite_id, res.case_id)
       er.suite_id AS "suite_id!: EvalSuiteId",
       res.case_id AS "case_id!",
       er.agent_id AS "agent_id!: FarmAgentId",
       er.id AS "eval_run_id!: EvalRunId",
       res.status AS "status!",
       (res.status = 'passed') AS "passed!",
       res.judge_score AS "judge_score?",
       res.judge_rationale AS "rationale?",
       COALESCE(er.finished_at, res.updated_at) AS "finished_at?"
FROM agent_farm_eval_results res
JOIN agent_farm_eval_runs er ON er.id = res.eval_run_id
WHERE res.status IN ('passed', 'failed', 'errored')
  AND ($1::text IS NULL OR er.farm_id = $1)
ORDER BY er.suite_id, res.case_id, res.updated_at DESC
