-- The feedback ingestion pipeline is retired.
--
-- Row triggers on every hook event, gateway request, tool call, artifact,
-- judge verdict, grant and device identity re-enqueued the writer's whole
-- client session into feedback_ingestion_outbox (113 ms a row on an
-- ai_requests settlement), for a drain job that projected them into core's
-- normalized facts and snapshots. The one page that read those facts, the
-- marketplace Versions pages, now reads conversation_skill_facts (104), and
-- nothing else ever did. Raw-evidence expiry no longer waits for the
-- projection: expire_feedback_raw becomes expire_raw_evidence, declared in
-- 32_raw_retention.sql.
--
-- Kept: feedback_authenticated_events and the receipt functions behind
-- analysis_skill_version_events (30_, 31_), whose identity now comes from
-- authenticated hook events alone.
--
-- Triggers before the functions they run, functions before the tables their
-- bodies name.
DROP TRIGGER IF EXISTS feedback_capture ON plugin_usage_events;
DROP TRIGGER IF EXISTS feedback_capture ON ai_requests;
DROP TRIGGER IF EXISTS feedback_capture ON conversation_analyses;
DROP TRIGGER IF EXISTS feedback_capture_tool ON ai_request_tool_calls;
DROP TRIGGER IF EXISTS feedback_capture_artifact ON mcp_artifacts;
DROP TRIGGER IF EXISTS feedback_capture_grant ON managed_consumer_grants;
DROP TRIGGER IF EXISTS feedback_capture_identity ON feedback_authenticated_events;
DROP TRIGGER IF EXISTS feedback_capture_identity ON managed_consumer_session_bindings;

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

DROP TABLE IF EXISTS feedback_ingestion_evidence;
DROP TABLE IF EXISTS feedback_ingestion_backfill;
DROP TABLE IF EXISTS feedback_ingestion_outbox;

-- analysis_skill_version_events loses its `traffic_class` column, which was
-- always 'production'. A view cannot drop a column in place, so it is dropped
-- here and 31_skill_version_receipts.sql recreates it in the dependent phase.
DROP VIEW IF EXISTS analysis_skill_version_events;

-- On a database from 0.52 the view above joined managed_invocation_attributions,
-- so core's marketplace 016 left that never-written table for this extension to
-- drop once the view was gone. Everywhere else core has dropped it already.
DROP TABLE IF EXISTS managed_invocation_attributions;
