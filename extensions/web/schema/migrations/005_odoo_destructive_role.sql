-- The `odoo_destructive` role: the governance grant that lets a person or a
-- service account's destructive Odoo tools pass scope_check and
-- tool_blocklist. The CHECK is widened in place; no rows are rewritten. Same
-- DDL as schema/23_groups_projects.sql.

ALTER TABLE user_manual_roles
    DROP CONSTRAINT IF EXISTS user_manual_roles_role_check;
ALTER TABLE user_manual_roles
    ADD CONSTRAINT user_manual_roles_role_check CHECK (role IN (
        'platform_admin', 'admin', 'developer', 'user', 'project_manager',
        'knowledge_worker', 'super_admin', 'odoo_destructive'));
