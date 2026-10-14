-- The scope's rollup and the same rollup per origin kind, from one pass:
-- GROUPING SETS yields the per-kind rows plus the total row (is_total = 1).
-- Per-run metered cost, tool uses and hook denials are folded in first so
-- they sum the same way the counts do.
WITH runs AS (
    SELECT ar.origin_kind,
           ar.status,
           ar.error_class,
           ar.duration_ms,
           ar.cost_microdollars,
           (SELECT COALESCE(SUM(r.cost_microdollars), 0)::bigint
              FROM ai_requests r
             WHERE r.session_id = ar.gateway_session_id) AS metered,
           (SELECT COUNT(*)::bigint
              FROM agent_run_events e
             WHERE e.run_id = ar.id AND e.kind = 'tool_use') AS tool_uses,
           (SELECT COUNT(*)::bigint
              FROM agent_run_events e
             WHERE e.run_id = ar.id
               AND e.kind = 'hook'
               AND e.payload ->> 'decision' = 'deny') AS hook_denials
      FROM agent_runs ar
      LEFT JOIN agent_farm_runs fr ON fr.agent_run_id = ar.id
      LEFT JOIN agent_farms f ON f.id = fr.farm_id
     WHERE ($1::text IS NULL OR ar.user_id = $1)
       AND ($2::text[] IS NULL OR f.project_id = ANY($2))
       AND ($3::text IS NULL OR ar.origin_kind = $3)
       AND ($4::text IS NULL OR ar.origin_id = $4)
       AND ($5::text IS NULL OR ar.status = $5)
       AND ($6::timestamptz IS NULL OR ar.created_at >= $6)
       AND ($7::timestamptz IS NULL OR ar.created_at < $7)
       AND ($8::text IS NULL OR fr.farm_id = $8)
)
SELECT origin_kind AS "origin_kind?",
       GROUPING(origin_kind)::int AS "is_total!",
       COUNT(*)::bigint AS "total!",
       COUNT(*) FILTER (WHERE status = 'queued')::bigint AS "queued!",
       COUNT(*) FILTER (WHERE status = 'preparing')::bigint AS "preparing!",
       COUNT(*) FILTER (WHERE status = 'running')::bigint AS "running!",
       COUNT(*) FILTER (WHERE status = 'succeeded')::bigint AS "succeeded!",
       COUNT(*) FILTER (WHERE status = 'failed')::bigint AS "failed!",
       COUNT(*) FILTER (WHERE status = 'cancelled')::bigint AS "cancelled!",
       COUNT(*) FILTER (WHERE status = 'timed_out')::bigint AS "timed_out!",
       COUNT(*) FILTER (WHERE error_class = 'ok')::bigint AS "err_ok!",
       COUNT(*) FILTER (WHERE error_class = 'budget_exceeded')::bigint AS "err_budget_exceeded!",
       COUNT(*) FILTER (WHERE error_class = 'timeout')::bigint AS "err_timeout!",
       COUNT(*) FILTER (WHERE error_class = 'output_invalid')::bigint AS "err_output_invalid!",
       COUNT(*) FILTER (WHERE error_class = 'hook_denied')::bigint AS "err_hook_denied!",
       COUNT(*) FILTER (WHERE error_class = 'host_crashed')::bigint AS "err_host_crashed!",
       COUNT(*) FILTER (WHERE error_class = 'connector_unavailable')::bigint
           AS "err_connector_unavailable!",
       COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars_reported!",
       COALESCE(SUM(metered), 0)::bigint AS "cost_microdollars_metered!",
       (percentile_cont(0.5) WITHIN GROUP (ORDER BY duration_ms::float8))::float8
           AS "p50_duration_ms?",
       (percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms::float8))::float8
           AS "p95_duration_ms?",
       COALESCE(SUM(tool_uses), 0)::bigint AS "tool_use_count!",
       COALESCE(SUM(hook_denials), 0)::bigint AS "hook_deny!"
FROM runs
GROUP BY GROUPING SETS ((origin_kind), ())
ORDER BY GROUPING(origin_kind) DESC, origin_kind
