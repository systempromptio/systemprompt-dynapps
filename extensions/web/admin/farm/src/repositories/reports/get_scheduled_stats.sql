-- The scheduler-started runs since $1 (the nightly audit flow) and what
-- their reports recorded: outcome, cost, MCP reads and writes, governance
-- warnings, DynBrain delivery and the mean eval score, plus the last run.
SELECT COUNT(r.id)::bigint AS "runs!",
       COUNT(r.id) FILTER (WHERE r.status = 'succeeded')::bigint AS "succeeded!",
       COALESCE(SUM(r.cost_microdollars), 0)::bigint AS "cost_microdollars!",
       COALESCE(SUM(rep.mcp_reads), 0)::bigint AS "mcp_reads!",
       COALESCE(SUM(rep.mcp_writes), 0)::bigint AS "mcp_writes!",
       COALESCE(SUM(rep.governance_warnings), 0)::bigint AS "governance_warnings!",
       COUNT(rep.run_id) FILTER (WHERE rep.delivery_status = 'delivered')::bigint AS "delivered!",
       COUNT(rep.run_id)::bigint AS "reported!",
       AVG(rep.eval_score)::float8 AS "eval_mean?",
       MAX(r.created_at) AS "last_run_at?",
       (ARRAY_AGG(r.id ORDER BY r.created_at DESC))[1] AS "last_run_id?: FarmRunId",
       (ARRAY_AGG(r.status ORDER BY r.created_at DESC))[1] AS "last_status?"
FROM agent_farm_runs r
LEFT JOIN agent_farm_run_reports rep ON rep.run_id = r.id
WHERE r.triggered_via = 'scheduler' AND r.created_at >= $1
