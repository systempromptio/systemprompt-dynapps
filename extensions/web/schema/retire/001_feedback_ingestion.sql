-- The feedback ingestion pipeline (retired in migration 105), dropped before
-- any extension migrates.
--
-- Core analytics 015 drops `analytics_ingestion_producers`, which every
-- `capture_feedback_*` function writes. Core migrates before this extension,
-- so on a database that skips releases (0.58 → 0.60) these triggers outlived
-- that table and every write in between failed: web 093 on
-- `plugin_usage_events`, web 102 on `ai_requests`. The retire phase runs
-- ahead of every migration, so they are gone first. 105 keeps its own drops
-- (already applied everywhere, and IF EXISTS); this file adds the two
-- `feedback_capture_identity` triggers 105 never named.
--
-- Triggers before the functions they run.
DROP TRIGGER IF EXISTS feedback_capture ON plugin_usage_events;
DROP TRIGGER IF EXISTS feedback_capture ON ai_requests;
DROP TRIGGER IF EXISTS feedback_capture ON conversation_analyses;
DROP TRIGGER IF EXISTS feedback_capture_tool ON ai_request_tool_calls;
DROP TRIGGER IF EXISTS feedback_capture_artifact ON mcp_artifacts;
DROP TRIGGER IF EXISTS feedback_capture_grant ON managed_consumer_grants;
DROP TRIGGER IF EXISTS feedback_capture_identity ON feedback_authenticated_events;
DROP TRIGGER IF EXISTS feedback_capture_identity ON managed_consumer_session_bindings;
DROP TRIGGER IF EXISTS feedback_capture ON session_analyses;
DROP TRIGGER IF EXISTS feedback_capture_identity ON managed_consumer_invocation_evidence;
DROP TRIGGER IF EXISTS feedback_capture_identity ON managed_consumer_attribution_projection;

DROP FUNCTION IF EXISTS capture_feedback_source();
DROP FUNCTION IF EXISTS capture_feedback_identity();
DROP FUNCTION IF EXISTS capture_feedback_tool_request();
DROP FUNCTION IF EXISTS capture_feedback_grant();
DROP FUNCTION IF EXISTS capture_feedback_artifact();
DROP FUNCTION IF EXISTS capture_feedback_judgement();
DROP FUNCTION IF EXISTS enqueue_feedback_session(TEXT, TEXT);
DROP FUNCTION IF EXISTS enqueue_feedback_fact(TEXT, TEXT, TEXT, TEXT, TEXT, TIMESTAMPTZ);
DROP FUNCTION IF EXISTS enqueue_feedback_artifact(TEXT, TEXT, TEXT, TIMESTAMPTZ);
DROP FUNCTION IF EXISTS feedback_normalized_fact(TEXT);
DROP FUNCTION IF EXISTS feedback_normalized_fact_base(TEXT);
DROP FUNCTION IF EXISTS feedback_normalized_artifact(TEXT);
DROP FUNCTION IF EXISTS feedback_artifact_invocation(TEXT);
DROP FUNCTION IF EXISTS feedback_artifact_request(TEXT);
DROP FUNCTION IF EXISTS feedback_artifact_skill(TEXT);
DROP FUNCTION IF EXISTS feedback_event_attribution(TEXT);
DROP FUNCTION IF EXISTS feedback_event_skill(TEXT);
DROP FUNCTION IF EXISTS feedback_fact_key(TEXT, TEXT, TEXT, TEXT);
DROP FUNCTION IF EXISTS feedback_producer_recount(BOOLEAN);
DROP FUNCTION IF EXISTS expire_feedback_raw(TIMESTAMPTZ);
