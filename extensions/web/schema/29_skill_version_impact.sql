-- Owner-scoped uniqueness on the two event tables, so a row can be named by
-- (user_id, id) and a lookup that carries the owner never crosses tenants.
CREATE UNIQUE INDEX IF NOT EXISTS plugin_usage_events_owner_id
    ON plugin_usage_events(user_id,id);
CREATE UNIQUE INDEX IF NOT EXISTS mcp_tool_executions_owner_id
    ON mcp_tool_executions(user_id,mcp_execution_id);
