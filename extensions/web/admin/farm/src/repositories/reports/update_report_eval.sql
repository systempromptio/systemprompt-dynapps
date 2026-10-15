-- Re-reads a settled eval result into its run's report (the eval settles
-- after the run's report is first written).
UPDATE agent_farm_run_reports rep
SET eval_passed = CASE WHEN er.status IN ('passed', 'failed', 'errored') THEN er.status = 'passed' END,
    eval_score = er.judge_score,
    completed = r.status = 'succeeded' AND er.status = 'passed'
FROM agent_farm_eval_results er
JOIN agent_farm_runs r ON r.id = er.run_id
WHERE er.id = $1 AND rep.run_id = er.run_id
