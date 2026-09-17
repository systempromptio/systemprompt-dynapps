-- One artifact with its stored body, its execution, the ledger row of the
-- same call and the decision keyed to that call.
SELECT
    a.artifact_id,
    a.mcp_execution_id,
    a.ai_tool_call_id,
    l.request_id,
    a.user_id,
    COALESCE(u.display_name, u.full_name, u.name, u.email) AS user_label,
    a.session_id,
    a.context_id,
    a.trace_id,
    a.tool_name,
    a.server_name,
    a.artifact_type,
    a.title,
    a.source,
    a.last_seen_source,
    l.correlation,
    l.client_kind,
    a.is_structured,
    a.has_ui_resource,
    a.is_error,
    a.payload_bytes,
    a.payload_sha256,
    a.secret_redactions,
    p.body,
    x.status AS execution_status,
    x.execution_time_ms,
    x.error_message,
    l.intended_at,
    g.id AS governance_decision_id,
    g.decision AS governance_decision,
    (SELECT s.skill FROM analysis_skill_events s
      WHERE s.user_id = a.user_id AND s.session_id = a.session_id AND s.invoked_at <= a.created_at
      ORDER BY s.invoked_at DESC LIMIT 1) AS skill,
    a.created_at
FROM mcp_artifacts a
LEFT JOIN tool_call_ledger l ON l.mcp_execution_id = a.mcp_execution_id
LEFT JOIN mcp_tool_executions x ON x.mcp_execution_id = a.mcp_execution_id
LEFT JOIN artifact_payloads p ON p.sha256 = a.payload_sha256
LEFT JOIN users u ON u.id = a.user_id
LEFT JOIN LATERAL (
    SELECT d.id, d.decision FROM governance_decisions d
    WHERE d.tool_use_id IS NOT NULL AND d.tool_use_id = a.ai_tool_call_id
    ORDER BY d.created_at DESC LIMIT 1
) g ON TRUE
WHERE a.artifact_id = $1
