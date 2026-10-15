SELECT run_id AS "run_id!: FarmRunId",
       completed AS "completed!",
       summary AS "summary?",
       final_message AS "final_message?",
       outputs AS "outputs!",
       odoo_writes AS "odoo_writes!",
       eval_passed AS "eval_passed?",
       eval_score AS "eval_score?",
       created_at AS "created_at!"
FROM agent_farm_run_reports
WHERE run_id = $1
