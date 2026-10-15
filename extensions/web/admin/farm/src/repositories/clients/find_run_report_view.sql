SELECT run_id AS "run_id!: FarmRunId",
       completed AS "completed!",
       summary AS "summary?",
       final_message AS "final_message?",
       outputs AS "outputs!",
       mcp_calls AS "mcp_calls!",
       mcp_reads AS "mcp_reads!",
       mcp_writes AS "mcp_writes!",
       governance_warnings AS "governance_warnings!",
       delivery_status AS "delivery_status!",
       eval_passed AS "eval_passed?",
       eval_score AS "eval_score?",
       created_at AS "created_at!"
FROM agent_farm_run_reports
WHERE run_id = $1
