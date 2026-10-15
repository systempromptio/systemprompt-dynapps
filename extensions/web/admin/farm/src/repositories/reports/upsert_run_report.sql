-- One report per run, recomputed on every call. The MCP tally, governance
-- warnings and delivery status come from the observer, which reads them from
-- the run's events and governance rows. The eval verdict is read from the
-- run's eval result once it has settled.
INSERT INTO agent_farm_run_reports (run_id, client_id, completed, summary, final_message, outputs,
                                    mcp_calls, mcp_reads, mcp_writes, governance_warnings,
                                    delivery_status, eval_passed, eval_score)
SELECT r.id,
       r.client_id,
       r.status = 'succeeded' AND (er.id IS NULL OR er.status = 'passed'),
       $2,
       $3,
       $4,
       $5,
       $6,
       $7,
       $8,
       $9,
       CASE WHEN er.status IN ('passed', 'failed', 'errored') THEN er.status = 'passed' END,
       er.judge_score
FROM agent_farm_runs r
LEFT JOIN agent_farm_eval_results er ON er.id = r.eval_result_id
WHERE r.id = $1
ON CONFLICT (run_id) DO UPDATE SET
    client_id = EXCLUDED.client_id,
    completed = EXCLUDED.completed,
    summary = EXCLUDED.summary,
    final_message = EXCLUDED.final_message,
    outputs = EXCLUDED.outputs,
    mcp_calls = EXCLUDED.mcp_calls,
    mcp_reads = EXCLUDED.mcp_reads,
    mcp_writes = EXCLUDED.mcp_writes,
    governance_warnings = EXCLUDED.governance_warnings,
    delivery_status = EXCLUDED.delivery_status,
    eval_passed = EXCLUDED.eval_passed,
    eval_score = EXCLUDED.eval_score
