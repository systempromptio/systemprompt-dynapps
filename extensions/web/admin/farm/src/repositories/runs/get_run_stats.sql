SELECT COUNT(*)::bigint AS "total!",
       COUNT(*) FILTER (WHERE status IN ('preparing', 'running'))::bigint AS "running!",
       COUNT(*) FILTER (WHERE status = 'queued')::bigint AS "queued!",
       COUNT(*) FILTER (WHERE status = 'succeeded')::bigint AS "succeeded!",
       COUNT(*) FILTER (WHERE status = 'failed')::bigint AS "failed!",
       COUNT(*) FILTER (WHERE status = 'timed_out')::bigint AS "timed_out!",
       COUNT(*) FILTER (WHERE status = 'cancelled')::bigint AS "cancelled!",
       COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars!",
       (percentile_cont(0.5) WITHIN GROUP (ORDER BY duration_ms))::float8 AS "p50_duration_ms?",
       (percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms))::float8 AS "p95_duration_ms?",
       COALESCE(SUM(input_tokens), 0)::bigint AS "input_tokens!",
       COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!"
FROM agent_farm_runs
WHERE ($1::text IS NULL OR farm_id = $1)
  AND ($2::text IS NULL OR agent_id = $2)
  AND ($3::text IS NULL OR user_id = $3)
  AND ($4::text IS NULL OR status = $4)
  AND ($5::text IS NULL OR triggered_via = $5)
  AND ($6::timestamptz IS NULL OR created_at >= $6)
  AND ($7::timestamptz IS NULL OR created_at < $7)
