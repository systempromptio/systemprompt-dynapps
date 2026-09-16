-- Current version-impact schema for clean installations. Migrations 056 and
-- 057 perform the equivalent forward transition for existing databases.
CREATE OR REPLACE VIEW analysis_skill_version_events AS
SELECT e.id AS invocation_id,e.user_id,e.session_id,e.plugin_id,e.skill,
       e.tool_use_id,e.source,e.invoked_at,a.installation_id,a.resource_id,a.revision_id,
       a.publication_generation,COALESCE(a.traffic_class,'production') AS traffic_class,
       COALESCE(a.status,'revision_unknown') AS attribution_status
FROM analysis_skill_events e
LEFT JOIN managed_invocation_attributions a
  ON a.owner_id=e.user_id AND a.invocation_id=e.id;

CREATE UNIQUE INDEX IF NOT EXISTS plugin_usage_events_owner_id
    ON plugin_usage_events(user_id,id);
CREATE UNIQUE INDEX IF NOT EXISTS mcp_tool_executions_owner_id
    ON mcp_tool_executions(user_id,mcp_execution_id);
