-- The strip above the list: the same WHERE as list.sql, aggregated.
SELECT
    count(*)::bigint                                          AS artifacts,
    count(*) FILTER (WHERE a.is_structured)::bigint           AS structured,
    count(*) FILTER (WHERE a.is_error)::bigint                AS errors,
    count(*) FILTER (WHERE a.has_ui_resource)::bigint         AS ui,
    count(*) FILTER (WHERE x.correlation = 'inferred')::bigint AS inferred,
    count(*) FILTER (WHERE EXISTS (
        SELECT 1 FROM mcp_artifact_findings f WHERE f.artifact_id = a.artifact_id))::bigint AS with_findings,
    COALESCE(sum(a.payload_bytes), 0)::bigint                 AS bytes,
    count(DISTINCT a.tool_name)::bigint                       AS tools,
    count(DISTINCT a.session_id)::bigint                      AS sessions
FROM mcp_artifacts a
LEFT JOIN mcp_tool_executions x ON x.mcp_execution_id = a.mcp_execution_id
WHERE ($1::timestamptz IS NULL OR a.created_at >= $1)
  AND ($2::timestamptz IS NULL OR a.created_at < $2)
  AND ($3::text IS NULL OR a.user_id = $3)
  AND ($4::text[] IS NULL OR a.user_id = ANY($4))
  AND ($5::text IS NULL OR a.session_id = $5)
  AND ($6::text IS NULL OR a.tool_name = $6)
  AND ($7::text IS NULL OR a.source = $7)
  AND (NOT $8::bool OR a.is_structured)
  AND (NOT $9::bool OR a.is_error)
