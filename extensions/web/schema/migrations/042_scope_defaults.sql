-- Exclusive attribution needs one container per person. Without it a member of
-- two groups lands in both group totals in full, and the group totals no longer
-- sum to the instance total.
CREATE TABLE IF NOT EXISTS user_scope_defaults (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    primary_group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
    primary_project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    source TEXT NOT NULL DEFAULT 'auto' CHECK (source IN ('auto', 'manual')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_user_scope_defaults_group
    ON user_scope_defaults(primary_group_id);
CREATE INDEX IF NOT EXISTS idx_user_scope_defaults_project
    ON user_scope_defaults(primary_project_id);

ALTER TABLE admin_usage_daily_rollups ADD COLUMN IF NOT EXISTS group_id TEXT;
ALTER TABLE admin_usage_daily_rollups ADD COLUMN IF NOT EXISTS project_id TEXT;

CREATE INDEX IF NOT EXISTS idx_admin_usage_rollups_group
    ON admin_usage_daily_rollups(group_id, date DESC);
CREATE INDEX IF NOT EXISTS idx_admin_usage_rollups_project
    ON admin_usage_daily_rollups(project_id, date DESC);
