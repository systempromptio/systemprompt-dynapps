-- Every extension table whose user_id is always a real account now references
-- users(id) ON DELETE CASCADE, so a deleted user takes these rows with them
-- instead of leaving orphans the console still counts. NOT VALID: rows an
-- earlier delete already orphaned are not the migration's to judge — new
-- rows are checked, and `ALTER TABLE … VALIDATE CONSTRAINT` can be run once
-- the orphans have been purged. Tables whose user_id can carry the `system`
-- principal (ai_request_scopes, conversation_analyses, the daily rollups)
-- keep no constraint and are cleared by the user purge instead.
ALTER TABLE user_settings DROP CONSTRAINT IF EXISTS user_settings_user_id_fkey;
ALTER TABLE user_settings ADD CONSTRAINT user_settings_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE user_profile_reports DROP CONSTRAINT IF EXISTS user_profile_reports_user_id_fkey;
ALTER TABLE user_profile_reports ADD CONSTRAINT user_profile_reports_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE user_encryption_keys DROP CONSTRAINT IF EXISTS user_encryption_keys_user_id_fkey;
ALTER TABLE user_encryption_keys ADD CONSTRAINT user_encryption_keys_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE plugin_env_vars DROP CONSTRAINT IF EXISTS plugin_env_vars_user_id_fkey;
ALTER TABLE plugin_env_vars ADD CONSTRAINT plugin_env_vars_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE secret_audit_log DROP CONSTRAINT IF EXISTS secret_audit_log_user_id_fkey;
ALTER TABLE secret_audit_log ADD CONSTRAINT secret_audit_log_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE secret_resolution_tokens DROP CONSTRAINT IF EXISTS secret_resolution_tokens_user_id_fkey;
ALTER TABLE secret_resolution_tokens ADD CONSTRAINT secret_resolution_tokens_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE plugin_usage_events DROP CONSTRAINT IF EXISTS plugin_usage_events_user_id_fkey;
ALTER TABLE plugin_usage_events ADD CONSTRAINT plugin_usage_events_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE plugin_usage_daily DROP CONSTRAINT IF EXISTS plugin_usage_daily_user_id_fkey;
ALTER TABLE plugin_usage_daily ADD CONSTRAINT plugin_usage_daily_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE plugin_session_summaries DROP CONSTRAINT IF EXISTS plugin_session_summaries_user_id_fkey;
ALTER TABLE plugin_session_summaries ADD CONSTRAINT plugin_session_summaries_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE session_transcripts DROP CONSTRAINT IF EXISTS session_transcripts_user_id_fkey;
ALTER TABLE session_transcripts ADD CONSTRAINT session_transcripts_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE session_ratings DROP CONSTRAINT IF EXISTS session_ratings_user_id_fkey;
ALTER TABLE session_ratings ADD CONSTRAINT session_ratings_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE session_entity_links DROP CONSTRAINT IF EXISTS session_entity_links_user_id_fkey;
ALTER TABLE session_entity_links ADD CONSTRAINT session_entity_links_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE skill_ratings DROP CONSTRAINT IF EXISTS skill_ratings_user_id_fkey;
ALTER TABLE skill_ratings ADD CONSTRAINT skill_ratings_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
ALTER TABLE session_cost_snapshots DROP CONSTRAINT IF EXISTS session_cost_snapshots_user_id_fkey;
ALTER TABLE session_cost_snapshots ADD CONSTRAINT session_cost_snapshots_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE NOT VALID;
