-- C14
SELECT * FROM analytics_ingestion_producers;
SELECT producer, pending, count(*), min(created_at), max(created_at) FROM feedback_ingestion_outbox GROUP BY 1,2 ORDER BY 1,2;
SELECT kind, pending, count(*) FROM feedback_ingestion_outbox GROUP BY 1,2 ORDER BY 3 DESC;
SELECT * FROM analytics_fact_checkpoints;
SELECT state, count(*) FROM analytics_fact_changes GROUP BY 1;
SELECT * FROM analytics_fact_consumers;
SELECT count(*) deltas, count(*) FILTER (WHERE consumed_at IS NULL) unconsumed FROM analytics_fact_deltas;
SELECT * FROM feedback_ingestion_backfill;
SELECT * FROM analytics_snapshot_state;
SELECT count(*) FROM analytics_snapshot_daily; SELECT count(*) FROM analytics_normalized_facts; SELECT count(*) FROM analytics_feedback_snapshots;
-- C16: governance vs requests per day
SELECT d, sum(dec_no_req) decisions_without_request, sum(req_no_dec) requests_without_decision, sum(reqs) requests, sum(decs) decisions FROM (
  SELECT g.created_at::date d, count(*) FILTER (WHERE r.id IS NULL) dec_no_req, 0 req_no_dec, 0 reqs, count(*) decs FROM governance_decisions g LEFT JOIN ai_requests r ON r.trace_id=g.trace_id WHERE g.tool_name IS NOT NULL GROUP BY 1
  UNION ALL SELECT r.created_at::date, 0, count(*) FILTER (WHERE g.trace_id IS NULL), count(*), 0 FROM ai_requests r LEFT JOIN (SELECT DISTINCT trace_id FROM governance_decisions) g ON g.trace_id=r.trace_id GROUP BY 1) x GROUP BY 1 ORDER BY 1;
-- decisions without request: are they MCP tool calls (trace in mcp_tool_executions)?
SELECT count(*) n, count(*) FILTER (WHERE m.trace_id IS NOT NULL) matches_mcp_exec, count(*) FILTER (WHERE g.tool_use_id IS NOT NULL) has_tool_use_id FROM governance_decisions g LEFT JOIN ai_requests r ON r.trace_id=g.trace_id LEFT JOIN (SELECT DISTINCT trace_id FROM mcp_tool_executions) m ON m.trace_id=g.trace_id WHERE r.id IS NULL;
SELECT policy, decision, count(*) FROM governance_decisions g WHERE NOT EXISTS (SELECT 1 FROM ai_requests r WHERE r.trace_id=g.trace_id) GROUP BY 1,2 ORDER BY 3 DESC;
SELECT policy, count(*) FROM governance_decisions GROUP BY 1 ORDER BY 2 DESC;
-- scopes
SELECT count(*) requests, count(s.ai_request_id) with_scope, count(*) FILTER (WHERE s.group_id IS NOT NULL AND s.group_id NOT IN (SELECT id FROM groups)) missing_group, count(*) FILTER (WHERE s.project_id IS NOT NULL AND s.project_id NOT IN (SELECT id FROM projects)) missing_project,
       count(*) FILTER (WHERE s.group_id IS NULL) null_group, count(*) FILTER (WHERE s.project_id IS NULL) null_project FROM ai_requests r LEFT JOIN ai_request_scopes s ON s.ai_request_id=r.id;
\d ai_request_scopes
SELECT group_id, project_id, source, count(*) FROM ai_request_scopes GROUP BY 1,2,3 ORDER BY 4 DESC LIMIT 10;
-- C17
SELECT count(*) findings, count(*) FILTER (WHERE r.id IS NULL) orphan_findings, count(*) FILTER (WHERE blocked) blocked FROM ai_safety_findings f LEFT JOIN ai_requests r ON r.id=f.ai_request_id;
SELECT category, direction, blocked, count(*) FROM ai_safety_findings GROUP BY 1,2,3 ORDER BY 4 DESC LIMIT 12;
SELECT count(*) tool_calls, count(mcp_execution_id) linked_exec, count(*) FILTER (WHERE mcp_execution_id IS NOT NULL AND mcp_execution_id NOT IN (SELECT mcp_execution_id FROM mcp_tool_executions)) dangling FROM ai_request_tool_calls;
SELECT count(*) execs, count(*) FILTER (WHERE mcp_execution_id NOT IN (SELECT mcp_execution_id FROM ai_request_tool_calls WHERE mcp_execution_id IS NOT NULL)) not_referenced_by_tool_call FROM mcp_tool_executions;
-- C18
SELECT (SELECT count(*) FROM mcp_artifacts) artifacts, (SELECT count(*) FROM artifact_parts) parts, (SELECT count(*) FROM artifact_payloads) payloads,
       (SELECT count(*) FROM mcp_artifacts a WHERE NOT EXISTS (SELECT 1 FROM artifact_parts p WHERE p.artifact_id=a.artifact_id)) artifacts_without_parts,
       (SELECT count(*) FROM artifact_parts p WHERE NOT EXISTS (SELECT 1 FROM artifact_payloads y WHERE y.part_id=p.id)) parts_without_payload,
       (SELECT count(*) FROM mcp_artifacts a WHERE NOT EXISTS (SELECT 1 FROM mcp_tool_executions e WHERE e.mcp_execution_id=a.mcp_execution_id)) artifacts_without_execution;
\d artifact_parts
\d artifact_payloads
