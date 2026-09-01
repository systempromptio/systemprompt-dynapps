-- Single-tenant, two projects. Organizations, plans, seats, departments and
-- organization budgets are gone: every user is admin or user, and belongs to
-- one project (commerce | core) taken from their AD group at sign-in.
ALTER TABLE user_profile_ext ADD COLUMN IF NOT EXISTS project TEXT;
UPDATE user_profile_ext upe SET project = g.project FROM (
    SELECT user_id,
           CASE group_name
               WHEN 'Systemprompt-Commerce' THEN 'commerce'
               WHEN 'Systemprompt-Core' THEN 'core'
           END AS project
    FROM adfs_group_memberships
    WHERE group_name IN ('Systemprompt-Commerce', 'Systemprompt-Core')
) g
WHERE g.user_id = upe.user_id AND upe.project IS NULL;
ALTER TABLE user_profile_ext DROP CONSTRAINT IF EXISTS user_profile_ext_project_check;
ALTER TABLE user_profile_ext ADD CONSTRAINT user_profile_ext_project_check
    CHECK (project IN ('commerce', 'core'));
CREATE INDEX IF NOT EXISTS idx_user_profile_ext_project ON user_profile_ext(project);
DROP INDEX IF EXISTS idx_user_profile_ext_department;
ALTER TABLE user_profile_ext DROP COLUMN IF EXISTS department;
DELETE FROM access_control_rules WHERE rule_type IN ('department', 'organization');
DELETE FROM adfs_group_memberships WHERE group_name = 'Systemprompt-Users';
DROP TABLE IF EXISTS org_budget_warnings;
DROP TABLE IF EXISTS user_invites;
DROP TABLE IF EXISTS departments;
DROP TABLE IF EXISTS organization_members;
DROP TABLE IF EXISTS organizations;
DROP TABLE IF EXISTS plans;
