-- Tables and views with no reader and no writer are dropped.
--
-- daily_summaries, admin_traffic_reports, tenant_activity, session_ratings,
-- skill_ratings, user_profile_reports, session_transcripts,
-- session_cost_snapshots and ingestion_repairs belong to features removed
-- long ago; only the retention function, the ingestion-owner triggers and
-- one always-empty rating column still named them. The views
-- conversation_rollups, conversation_analysis_rows and v_all_activity were
-- read by nothing.
--
-- Views before tables, because v_all_activity reads session_ratings.
-- Every statement is a DROP ... IF EXISTS, so this is a retirement
-- migration: executed as well as stamped on a fresh extension.
DROP VIEW IF EXISTS v_all_activity;
DROP VIEW IF EXISTS conversation_rollups;
DROP VIEW IF EXISTS conversation_analysis_rows;
DROP TABLE IF EXISTS daily_summaries;
DROP TABLE IF EXISTS admin_traffic_reports;
DROP TABLE IF EXISTS tenant_activity;
DROP TABLE IF EXISTS session_ratings;
DROP TABLE IF EXISTS skill_ratings;
DROP TABLE IF EXISTS user_profile_reports;
DROP TABLE IF EXISTS session_transcripts;
DROP TABLE IF EXISTS session_cost_snapshots;
DROP TABLE IF EXISTS ingestion_repairs;
