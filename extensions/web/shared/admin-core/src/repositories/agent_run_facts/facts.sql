-- One row per run in scope, with every per-run fact folded in. $1 narrows
-- to one run (the `find`); the rest are the list's scope and filter.
-- Metered cost and governance counts hang off the run's gateway session,
-- the one credential core mints per run, so a run with no session meters
-- nothing. Hook verdicts come from the run's own event stream.
SELECT ar.id AS "run_id!: AgentRunId",
       ar.origin_kind AS "origin_kind!",
       ar.origin_id AS "origin_id!",
       ar.user_id AS "user_id!: UserId",
       f.project_id AS "farm_project_id?: ProjectId",
       ar.status AS "status!",
       ar.error_class AS "error_class?",
       ar.started_at AS "started_at?",
       ar.finished_at AS "finished_at?",
       ar.duration_ms AS "duration_ms?",
       ar.attempt AS "attempt!",
       ar.turns AS "turns?",
       ar.input_tokens AS "input_tokens?",
       ar.output_tokens AS "output_tokens?",
       ar.cost_microdollars AS "cost_microdollars_reported?",
       ar.request -> 'output_schema' AS "output_schema?",
       ar.structured_output AS "structured_output?",
       ar.diff_summary AS "diff_summary?",
       (SELECT COALESCE(SUM(r.cost_microdollars), 0)::bigint
          FROM ai_requests r
         WHERE r.session_id = ar.gateway_session_id) AS "cost_microdollars_metered!",
       ev.tool_use_count AS "tool_use_count!",
       ev.tool_result_count AS "tool_result_count!",
       ev.tool_result_error_count AS "tool_result_error_count!",
       ev.hook_allow AS "hook_allow!",
       ev.hook_deny AS "hook_deny!",
       ev.tools_by_name AS "tools_by_name!: Json<Vec<(String, i64)>>",
       gov.allow AS "governance_allow!",
       gov.deny AS "governance_deny!"
FROM agent_runs ar
LEFT JOIN agent_farm_runs fr ON fr.agent_run_id = ar.id
LEFT JOIN agent_farms f ON f.id = fr.farm_id
CROSS JOIN LATERAL (
    SELECT COUNT(*) FILTER (WHERE e.kind = 'tool_use')::bigint AS tool_use_count,
           COUNT(*) FILTER (WHERE e.kind = 'tool_result')::bigint AS tool_result_count,
           COUNT(*) FILTER (WHERE e.kind = 'tool_result'
                              AND (e.payload ->> 'is_error')::boolean)::bigint
               AS tool_result_error_count,
           COUNT(*) FILTER (WHERE e.kind = 'hook'
                              AND e.payload ->> 'decision' = 'allow')::bigint AS hook_allow,
           COUNT(*) FILTER (WHERE e.kind = 'hook'
                              AND e.payload ->> 'decision' = 'deny')::bigint AS hook_deny,
           COALESCE((SELECT jsonb_agg(jsonb_build_array(t.tool_name, t.n)
                                      ORDER BY t.n DESC, t.tool_name)
                       FROM (SELECT tool_name, COUNT(*)::bigint AS n
                               FROM agent_run_events
                              WHERE run_id = ar.id
                                AND kind = 'tool_use'
                                AND tool_name IS NOT NULL
                              GROUP BY tool_name) t),
                    '[]'::jsonb) AS tools_by_name
      FROM agent_run_events e
     WHERE e.run_id = ar.id
) ev
CROSS JOIN LATERAL (
    SELECT COUNT(*) FILTER (WHERE g.decision = 'allow')::bigint AS allow,
           COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint AS deny
      FROM governance_decisions g
     WHERE g.session_id = ar.gateway_session_id
) gov
WHERE ($1::text IS NULL OR ar.id = $1)
  AND ($2::text IS NULL OR ar.user_id = $2)
  AND ($3::text[] IS NULL OR f.project_id = ANY($3))
  AND ($4::text IS NULL OR ar.origin_kind = $4)
  AND ($5::text IS NULL OR ar.origin_id = $5)
  AND ($6::text IS NULL OR ar.status = $6)
  AND ($7::timestamptz IS NULL OR ar.created_at >= $7)
  AND ($8::timestamptz IS NULL OR ar.created_at < $8)
  AND ($9::text IS NULL OR fr.farm_id = $9)
ORDER BY ar.created_at DESC, ar.id
LIMIT $10
