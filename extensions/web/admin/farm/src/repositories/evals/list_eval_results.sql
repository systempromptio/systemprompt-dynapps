SELECT id AS "id: EvalResultId", eval_run_id AS "eval_run_id: EvalRunId", case_id, run_id, status,
       assertions, judge_score, judge_rationale, created_at, updated_at
FROM agent_farm_eval_results
WHERE eval_run_id = $1
ORDER BY case_id
