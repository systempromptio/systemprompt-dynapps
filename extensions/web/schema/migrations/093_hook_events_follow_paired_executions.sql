-- Core's mcp migration 014 folded every client-hook attestation into the
-- server-observed execution it duplicated and deleted the hook row, so a
-- `PostToolUse` event that pointed at the hook row now points at nothing.
-- The event carries the client's `tool_use_id`, which the paired execution
-- now holds as `ai_tool_call_id`, so the event follows it there. Idempotent:
-- only an event whose execution is missing is touched.
UPDATE plugin_usage_events e
SET mcp_execution_id = x.mcp_execution_id
FROM mcp_tool_executions x
WHERE e.tool_use_id IS NOT NULL
  AND x.ai_tool_call_id = e.tool_use_id
  AND (e.mcp_execution_id IS NULL
       OR NOT EXISTS (
           SELECT 1 FROM mcp_tool_executions d WHERE d.mcp_execution_id = e.mcp_execution_id
       ));
