INSERT INTO agent_farm_eval_results (id, eval_run_id, case_id)
VALUES ($1, $2, $3)
RETURNING id AS "id: EvalResultId", eval_run_id AS "eval_run_id: EvalRunId", case_id, run_id, status,
       assertions, judge_score, judge_rationale, created_at, updated_at
