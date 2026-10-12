SELECT suite_id AS "suite_id: EvalSuiteId", id, prompt, fixture, timeout_secs, assertions,
       judge_rubric, judge_threshold
FROM agent_farm_eval_cases
WHERE suite_id = $1
ORDER BY id
