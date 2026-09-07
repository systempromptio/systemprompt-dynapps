-- Retain the cleanup from retired migration 038 without recreating the
-- temporary project column or referencing the retired directory table.
DROP INDEX IF EXISTS idx_user_profile_ext_department;
ALTER TABLE user_profile_ext DROP COLUMN IF EXISTS department;
DELETE FROM access_control_rules WHERE rule_type IN ('department', 'organization');
DROP TABLE IF EXISTS org_budget_warnings;
DROP TABLE IF EXISTS user_invites;
DROP TABLE IF EXISTS departments;
DROP TABLE IF EXISTS organization_members;
DROP TABLE IF EXISTS organizations;
DROP TABLE IF EXISTS plans;
