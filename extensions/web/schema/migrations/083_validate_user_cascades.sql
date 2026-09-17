-- Finish what 082 started. The cascading user_id constraints were added NOT
-- VALID so rows an earlier delete had already orphaned would not fail the
-- migration; a fresh install declares the same constraints validated, and the
-- upgrade test holds the two schemas to be identical. The orphans are exactly
-- the rows a deleted user should have taken with them, so they go, and each
-- constraint is validated.
--
-- Core 0.55.0 dropped idx_mcp_artifacts_mcp_execution_id from its schema
-- (008_artifact_narrow_waist replaced it with the unique
-- idx_mcp_artifacts_execution) without a migration to remove the old index
-- from an existing database; it is removed here so upgraded and fresh
-- databases carry the same indexes.
DELETE FROM user_settings t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE user_settings VALIDATE CONSTRAINT user_settings_user_id_fkey;
DELETE FROM user_profile_reports t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE user_profile_reports VALIDATE CONSTRAINT user_profile_reports_user_id_fkey;
DELETE FROM user_encryption_keys t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE user_encryption_keys VALIDATE CONSTRAINT user_encryption_keys_user_id_fkey;
DELETE FROM plugin_env_vars t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE plugin_env_vars VALIDATE CONSTRAINT plugin_env_vars_user_id_fkey;
DELETE FROM secret_audit_log t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE secret_audit_log VALIDATE CONSTRAINT secret_audit_log_user_id_fkey;
DELETE FROM secret_resolution_tokens t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE secret_resolution_tokens VALIDATE CONSTRAINT secret_resolution_tokens_user_id_fkey;
DELETE FROM plugin_usage_events t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE plugin_usage_events VALIDATE CONSTRAINT plugin_usage_events_user_id_fkey;
DELETE FROM plugin_usage_daily t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE plugin_usage_daily VALIDATE CONSTRAINT plugin_usage_daily_user_id_fkey;
DELETE FROM plugin_session_summaries t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE plugin_session_summaries VALIDATE CONSTRAINT plugin_session_summaries_user_id_fkey;
DELETE FROM session_transcripts t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE session_transcripts VALIDATE CONSTRAINT session_transcripts_user_id_fkey;
DELETE FROM session_ratings t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE session_ratings VALIDATE CONSTRAINT session_ratings_user_id_fkey;
DELETE FROM session_entity_links t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE session_entity_links VALIDATE CONSTRAINT session_entity_links_user_id_fkey;
DELETE FROM skill_ratings t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE skill_ratings VALIDATE CONSTRAINT skill_ratings_user_id_fkey;
DELETE FROM session_cost_snapshots t WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id);
ALTER TABLE session_cost_snapshots VALIDATE CONSTRAINT session_cost_snapshots_user_id_fkey;
DROP INDEX IF EXISTS idx_mcp_artifacts_mcp_execution_id;
