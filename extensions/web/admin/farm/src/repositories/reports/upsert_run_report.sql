-- One report per run, recomputed on every call. The Odoo writes are the odoo
-- MCP server's audit rows (governance_decisions) whose evaluated_rules carry
-- this farm run id; read-only tools are left out. The eval verdict is read
-- from the run's eval result once it has settled.
INSERT INTO agent_farm_run_reports (run_id, client_id, completed, summary, final_message, outputs,
                                    odoo_writes, eval_passed, eval_score)
SELECT r.id,
       r.client_id,
       r.status = 'succeeded' AND (er.id IS NULL OR er.status = 'passed'),
       $2,
       $3,
       $4,
       (SELECT COALESCE(jsonb_agg(jsonb_build_object(
                    'tool', g.evaluated_rules->>'tool',
                    'model', g.evaluated_rules->>'model',
                    'record_ids', COALESCE(g.evaluated_rules->'record_ids', '[]'::jsonb),
                    'decision', g.decision,
                    'policy', g.policy,
                    'at', g.created_at) ORDER BY g.created_at), '[]'::jsonb)
        FROM governance_decisions g
        WHERE g.evaluated_rules->>'server' = 'odoo'
          AND g.evaluated_rules->>'farm_run_id' = r.id
          AND g.evaluated_rules->>'tool' !~ '(_get|_list|_search|_report|_inventory)$'),
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
    odoo_writes = EXCLUDED.odoo_writes,
    eval_passed = EXCLUDED.eval_passed,
    eval_score = EXCLUDED.eval_score
