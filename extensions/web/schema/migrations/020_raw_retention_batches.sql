-- Raw evidence expiry deletes in bounded batches: `expire_raw_evidence` gains
-- a batch size and the job calls it until nothing is left, one short
-- transaction per call. The one-transaction form is dropped.
-- Same DDL as schema/32_raw_retention.sql.

DROP FUNCTION IF EXISTS expire_raw_evidence(TIMESTAMPTZ);

CREATE OR REPLACE FUNCTION expire_raw_evidence(cutoff TIMESTAMPTZ, batch_size INTEGER)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE removed BIGINT; total BIGINT := 0;
BEGIN
    IF cutoff > clock_timestamp()-interval '90 days' THEN
        RAISE EXCEPTION 'Raw evidence retention cutoff is too recent' USING ERRCODE='23514';
    END IF;
    IF batch_size IS NULL OR batch_size < 1 THEN
        RAISE EXCEPTION 'Raw evidence retention batch size must be positive' USING ERRCODE='22023';
    END IF;
    DELETE FROM managed_consumer_session_bindings WHERE ctid IN (
        SELECT b.ctid FROM managed_consumer_session_bindings b WHERE b.bound_at<cutoff
            AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.user_id=b.consumer_id AND e.session_id=b.native_session_id AND e.created_at>=cutoff)
        LIMIT batch_size);
    GET DIAGNOSTICS removed = ROW_COUNT; total := total + removed;
    DELETE FROM ingestion_event_receipts WHERE ctid IN (
        SELECT ctid FROM ingestion_event_receipts WHERE accepted_at<cutoff LIMIT batch_size);
    GET DIAGNOSTICS removed = ROW_COUNT; total := total + removed;
    DELETE FROM plugin_usage_events WHERE ctid IN (
        SELECT ctid FROM plugin_usage_events WHERE created_at<cutoff LIMIT batch_size);
    GET DIAGNOSTICS removed = ROW_COUNT; total := total + removed;
    DELETE FROM ai_requests WHERE ctid IN (
        SELECT ctid FROM ai_requests WHERE created_at<cutoff LIMIT batch_size);
    GET DIAGNOSTICS removed = ROW_COUNT; total := total + removed;
    DELETE FROM conversation_analyses WHERE ctid IN (
        SELECT ctid FROM conversation_analyses WHERE created_at<cutoff LIMIT batch_size);
    GET DIAGNOSTICS removed = ROW_COUNT; total := total + removed;
    DELETE FROM plugin_session_summaries WHERE ctid IN (
        SELECT ctid FROM plugin_session_summaries WHERE COALESCE(ended_at,updated_at,created_at)<cutoff LIMIT batch_size);
    GET DIAGNOSTICS removed = ROW_COUNT; total := total + removed;
    DELETE FROM session_entity_links WHERE ctid IN (
        SELECT ctid FROM session_entity_links WHERE last_seen_at<cutoff LIMIT batch_size);
    GET DIAGNOSTICS removed = ROW_COUNT; total := total + removed;
    DELETE FROM ingestion_session_owners WHERE ctid IN (
        SELECT o.ctid FROM ingestion_session_owners o WHERE o.created_at<cutoff
            AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.session_id=o.session_id)
            AND NOT EXISTS(SELECT 1 FROM plugin_session_summaries s WHERE s.session_id=o.session_id)
        LIMIT batch_size);
    GET DIAGNOSTICS removed = ROW_COUNT; total := total + removed;
    IF total = 0 THEN
        PERFORM public.expire_user_sessions(cutoff);
    END IF;
    RETURN total;
END $$;
