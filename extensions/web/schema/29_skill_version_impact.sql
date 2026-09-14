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
CREATE UNIQUE INDEX IF NOT EXISTS eval_resource_revisions_owner_id
    ON eval_resource_revisions(owner_id,id);

-- The owner-paired keys need the unique indexes declared above, which on an
-- established database only arrive through migration 057. The installer
-- applies every FOREIGN KEY after migrations and indexes (core's deferred
-- foreign-key phase), and skips a key 057 already created under the same
-- columns — so the same declaration converges fresh and upgraded databases.
-- An inline key that ran in the structural phase took the 0.52 preview down
-- at boot on 2026-09-14; this shape is what the upgrade gate proves.
CREATE TABLE IF NOT EXISTS reviewed_production_failures (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    invocation_id TEXT NOT NULL REFERENCES plugin_usage_events(id),
    reviewer_id TEXT NOT NULL REFERENCES users(id),
    sanitized_evidence JSONB NOT NULL,
    development_case_revision_id TEXT NOT NULL REFERENCES eval_resource_revisions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner_id,invocation_id),
    CONSTRAINT reviewed_failure_invocation_owner
        FOREIGN KEY(owner_id,invocation_id) REFERENCES plugin_usage_events(user_id,id),
    CONSTRAINT reviewed_failure_case_owner
        FOREIGN KEY(owner_id,development_case_revision_id) REFERENCES eval_resource_revisions(owner_id,id)
);

