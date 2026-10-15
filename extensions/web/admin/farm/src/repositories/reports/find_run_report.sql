-- A run's report with the run facts it is read beside: how it was started,
-- when it ran, and the turns, tokens and cost core recorded.
SELECT rep.run_id AS "run_id: FarmRunId", rep.client_id AS "client_id: ClientId", rep.completed,
       rep.summary, rep.final_message, rep.outputs, rep.mcp_calls, rep.mcp_reads, rep.mcp_writes,
       rep.governance_warnings, rep.delivery_status, rep.eval_passed, rep.eval_score,
       rep.created_at, r.triggered_via, r.started_at, r.finished_at, r.cost_microdollars,
       ar.turns AS "turns?", ar.input_tokens AS "input_tokens?",
       ar.output_tokens AS "output_tokens?"
FROM agent_farm_run_reports rep
JOIN agent_farm_runs r ON r.id = rep.run_id
LEFT JOIN agent_runs ar ON ar.id = r.agent_run_id
WHERE rep.run_id = $1
