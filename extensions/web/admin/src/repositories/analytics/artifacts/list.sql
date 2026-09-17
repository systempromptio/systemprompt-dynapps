-- Artifacts in a window, newest first, with the ledger row of the same call
-- and the owner's display name. Every filter is optional: `$n IS NULL OR …`
-- keeps one query shape for the list and the strip above it.
SELECT
    a.artifact_id,
    a.mcp_execution_id,
    a.ai_tool_call_id,
    l.request_id,
    a.user_id,
    COALESCE(u.display_name, u.full_name, u.name, u.email) AS user_label,
    a.session_id,
    a.context_id,
    a.tool_name,
    a.server_name,
    a.artifact_type,
    a.title,
    a.source,
    l.correlation,
    l.client_kind,
    a.is_structured,
    a.has_ui_resource,
    a.is_error,
    a.payload_bytes,
    a.secret_redactions,
    (SELECT count(*) FROM mcp_artifact_findings f WHERE f.artifact_id = a.artifact_id)::bigint AS finding_count,
    (SELECT s.skill FROM analysis_skill_events s
      WHERE s.user_id = a.user_id AND s.session_id = a.session_id AND s.invoked_at <= a.created_at
      ORDER BY s.invoked_at DESC LIMIT 1) AS skill,
    a.created_at,
    count(*) OVER ()::bigint AS total_rows
FROM mcp_artifacts a
LEFT JOIN tool_call_ledger l ON l.mcp_execution_id = a.mcp_execution_id
LEFT JOIN users u ON u.id = a.user_id
WHERE ($1::timestamptz IS NULL OR a.created_at >= $1)
  AND ($2::timestamptz IS NULL OR a.created_at < $2)
  AND ($3::text IS NULL OR a.user_id = $3)
  AND ($4::text[] IS NULL OR a.user_id = ANY($4))
  AND ($5::text IS NULL OR a.session_id = $5)
  AND ($6::text IS NULL OR a.tool_name = $6)
  AND ($7::text IS NULL OR a.source = $7)
  AND (NOT $8::bool OR a.is_structured)
  AND (NOT $9::bool OR a.is_error)
  AND ($10::text IS NULL OR EXISTS (
        SELECT 1 FROM analysis_skill_events s
        WHERE s.user_id = a.user_id AND s.session_id = a.session_id
          AND s.invoked_at <= a.created_at AND s.skill = $10))
ORDER BY a.created_at DESC
LIMIT $11 OFFSET $12
