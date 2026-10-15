SELECT run_id AS "run_id: FarmRunId", client_id AS "client_id: ClientId", completed, summary,
       final_message, outputs, odoo_writes, eval_passed, eval_score, created_at
FROM agent_farm_run_reports
WHERE run_id = $1
