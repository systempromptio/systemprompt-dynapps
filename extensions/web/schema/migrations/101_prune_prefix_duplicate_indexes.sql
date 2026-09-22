-- Drop every index on this extension's tables whose column list is a strict
-- prefix of another index on the same table. A prefix index answers no query
-- its superset cannot; it only costs a write on every insert. The production
-- analysis of 2026-09-22 found 87 such pairs instance-wide, 15 of them on
-- tables this extension owns.
--
-- Each one names the index that already covers it. None is UNIQUE and none
-- backs a constraint, so no uniqueness or foreign key changes. A leading
-- column indexed DESC here and ASC in the superset is still covered: btree
-- scans a prefix in either direction.

-- covered by idx_atr_date (report_date DESC, generated_at DESC)
DROP INDEX IF EXISTS idx_admin_traffic_reports_date;
-- covered by daily_summaries_pkey (user_id, summary_date)
DROP INDEX IF EXISTS idx_daily_summaries_user;
-- covered by session_entity_links_user_id_session_id_entity_type_entity__key
DROP INDEX IF EXISTS idx_sel_user;
-- covered by session_ratings_user_id_session_id_key
DROP INDEX IF EXISTS idx_sr_user;
-- covered by skill_ratings_user_id_skill_name_key
DROP INDEX IF EXISTS idx_skr_user;
-- covered by idx_tenant_activity_sync (remote_created_at, synced_at)
DROP INDEX IF EXISTS idx_tenant_activity_remote_created_at;

-- covered by plugin_env_vars_user_id_plugin_id_var_name_key
DROP INDEX IF EXISTS idx_plugin_env_user_plugin;
-- covered by secret_resolution_tokens_token_hash_key (same column)
DROP INDEX IF EXISTS idx_secret_resolution_tokens_hash;
-- covered by user_encryption_keys_user_id_key (same column)
DROP INDEX IF EXISTS idx_user_encryption_keys_user_id;

-- covered by plugin_session_summaries_session_id_key (same column)
DROP INDEX IF EXISTS idx_session_summary_session;
-- covered by idx_usage_daily_unique (date, user_id, event_type, COALESCE(tool_name, ''))
DROP INDEX IF EXISTS idx_usage_daily_date;

-- covered by marketplace.paddle_customers_user_id_key (same column)
DROP INDEX IF EXISTS marketplace.idx_mpc_user;
-- covered by marketplace.paddle_customers_paddle_customer_id_key (same column)
DROP INDEX IF EXISTS marketplace.idx_mpc_paddle;
-- covered by marketplace.subscriptions_paddle_subscription_id_key (same column)
DROP INDEX IF EXISTS marketplace.idx_ms_paddle;
-- covered by marketplace.paddle_webhook_events_event_id_key (same column)
DROP INDEX IF EXISTS marketplace.idx_mpwe_event;
