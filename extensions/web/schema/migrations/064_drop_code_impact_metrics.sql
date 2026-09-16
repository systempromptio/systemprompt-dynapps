-- Code-impact metrics are withdrawn until there is a sound way to measure
-- them. Hook-observed line deltas counted lines in Edit/Write tool inputs
-- (a whole-file Write counted every line), and commits were regex-parsed
-- from git stdout in Claude Code Bash calls, so anything done in another
-- terminal was invisible. Neither frame answered "what did the AI produce",
-- and showing them side by side invited exactly that reading. Drop the
-- storage and redefine the two functions that referenced it.
DROP TABLE IF EXISTS user_commits;

ALTER TABLE admin_usage_daily_rollups
    DROP COLUMN IF EXISTS loc_added_ai,
    DROP COLUMN IF EXISTS loc_removed_ai,
    DROP COLUMN IF EXISTS commits_count,
    DROP COLUMN IF EXISTS commit_insertions,
    DROP COLUMN IF EXISTS commit_deletions;

ALTER TABLE plugin_usage_events
    DROP COLUMN IF EXISTS loc_added,
    DROP COLUMN IF EXISTS loc_removed;
ALTER TABLE plugin_usage_daily
    DROP COLUMN IF EXISTS loc_added,
    DROP COLUMN IF EXISTS loc_removed;
ALTER TABLE plugin_session_summaries
    DROP COLUMN IF EXISTS loc_added,
    DROP COLUMN IF EXISTS loc_removed;

-- Same bodies as schema/28_ingestion_integrity.sql and
-- schema/32_feedback_retention.sql, minus the dropped columns and table.
CREATE OR REPLACE FUNCTION drain_ingestion_outbox(batch_size integer)
RETURNS bigint LANGUAGE plpgsql AS $$
DECLARE event_ids text[]; affected bigint;
BEGIN
    SELECT array_agg(event_id) INTO event_ids FROM (
        SELECT event_id FROM ingestion_outbox WHERE processed_at IS NULL ORDER BY created_at
        LIMIT least(greatest(batch_size, 1), 10000) FOR UPDATE SKIP LOCKED
    ) pending;
    IF event_ids IS NULL THEN RETURN 0; END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended('ingestion:' || session_id, 0))
    FROM (SELECT DISTINCT session_id FROM plugin_usage_events WHERE id = ANY(event_ids) ORDER BY session_id) owners;
    INSERT INTO plugin_session_summaries(id,session_id,user_id,total_events,tool_uses,prompts,errors,
        content_input_bytes,content_output_bytes,started_at,subagent_spawns,user_prompts,automated_actions)
    SELECT 'sess_' || e.session_id,e.session_id,e.user_id,count(*),
        count(*) FILTER(WHERE event_type IN ('PostToolUse','PostToolUseFailure')),
        count(*) FILTER(WHERE event_type = 'UserPromptSubmit'),
        count(*) FILTER(WHERE event_type = 'PostToolUseFailure'),
        coalesce(sum(content_input_bytes),0),coalesce(sum(content_output_bytes),0),
        min(e.created_at),
        count(*) FILTER(WHERE event_type='SubagentStop'),
        count(*) FILTER(WHERE event_type='UserPromptSubmit' AND coalesce(metadata->>'agent_id','')=''),
        count(*) FILTER(WHERE event_type IN ('PostToolUse','PostToolUseFailure') AND coalesce(metadata->>'agent_id','')<>'')
    FROM plugin_usage_events e WHERE (e.user_id,e.session_id) IN (
        SELECT user_id,session_id FROM plugin_usage_events WHERE id = ANY(event_ids))
    GROUP BY e.user_id,e.session_id
    ON CONFLICT(session_id) DO UPDATE SET total_events=EXCLUDED.total_events,
        tool_uses=EXCLUDED.tool_uses,prompts=EXCLUDED.prompts,errors=EXCLUDED.errors,
        content_input_bytes=EXCLUDED.content_input_bytes,content_output_bytes=EXCLUDED.content_output_bytes,
        subagent_spawns=EXCLUDED.subagent_spawns,
        user_prompts=EXCLUDED.user_prompts,automated_actions=EXCLUDED.automated_actions,updated_at=now()
    WHERE plugin_session_summaries.user_id=EXCLUDED.user_id;
    UPDATE ingestion_outbox SET processed_at=now() WHERE event_id=ANY(event_ids);
    GET DIAGNOSTICS affected = ROW_COUNT;
    RETURN affected;
END $$;

CREATE OR REPLACE FUNCTION expire_feedback_raw(cutoff TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE deleted BIGINT; producer_row RECORD;
BEGIN
    PERFORM public.prepare_reporting_privacy();
    LOCK TABLE analytics_ingestion_producers,analytics_fact_checkpoints,analytics_fact_backfills,
        analytics_fact_changes,analytics_fact_consumers,analytics_fact_deltas IN EXCLUSIVE MODE;
    IF cutoff > clock_timestamp()-interval '90 days' THEN
        RAISE EXCEPTION 'Feedback retention cutoff is too recent' USING ERRCODE='23514';
    END IF;
    FOR producer_row IN SELECT * FROM analytics_ingestion_producers ORDER BY producer FOR UPDATE LOOP
        IF producer_row.pending_count<>0 THEN RAISE EXCEPTION 'Feedback retention waits for pending producers' USING ERRCODE='55000'; END IF;
    END LOOP;
    IF EXISTS(SELECT 1 FROM analytics_fact_changes WHERE state IN ('pending','leased'))
       OR EXISTS(SELECT 1 FROM analytics_fact_deltas WHERE consumed_at IS NULL)
       OR EXISTS(SELECT 1 FROM analytics_fact_checkpoints p WHERE NOT EXISTS(
           SELECT 1 FROM analytics_fact_consumers c WHERE c.owner_id=p.owner_id AND c.consumer='snapshots-v1' AND c.generation>=p.generation)) THEN
        RAISE EXCEPTION 'Feedback retention waits for completed snapshots' USING ERRCODE='55000';
    END IF;
    IF EXISTS(SELECT 1 FROM analytics_fact_checkpoints p WHERE NOT EXISTS(
        SELECT 1 FROM analytics_snapshot_state s WHERE s.owner_id=p.owner_id AND s.evidence_cutoff>=cutoff)) THEN
        RAISE EXCEPTION 'Feedback retention waits for every owner compaction' USING ERRCODE='55000';
    END IF;
    PERFORM set_config('astound.feedback_retention','expiry',true);
    DELETE FROM reviewed_production_failures WHERE invocation_id IN(SELECT id FROM plugin_usage_events WHERE created_at<cutoff);
    DELETE FROM managed_invocation_attributions WHERE invocation_id IN(SELECT id FROM plugin_usage_events WHERE created_at<cutoff);
    DELETE FROM managed_consumer_invocation_evidence WHERE occurred_at<cutoff;
    DELETE FROM managed_consumer_session_bindings b WHERE bound_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM managed_consumer_invocation_evidence e WHERE e.consumer_id=b.consumer_id AND e.device_id=b.device_id AND e.host=b.host AND e.native_session_id=b.native_session_id)
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.user_id=b.consumer_id AND e.session_id=b.native_session_id AND e.created_at>=cutoff);
    DELETE FROM ingestion_event_receipts WHERE accepted_at<cutoff;
    DELETE FROM plugin_usage_events WHERE created_at<cutoff;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    DELETE FROM ai_requests WHERE created_at<cutoff;
    DELETE FROM session_analyses WHERE created_at<cutoff;
    DELETE FROM session_transcripts WHERE captured_at<cutoff;
    DELETE FROM session_cost_snapshots WHERE updated_at<cutoff;
    DELETE FROM plugin_session_summaries WHERE COALESCE(ended_at,updated_at,created_at)<cutoff;
    DELETE FROM session_entity_links WHERE last_seen_at<cutoff;
    DELETE FROM session_ratings WHERE created_at<cutoff;
    DELETE FROM daily_summaries WHERE summary_date<cutoff::date;
    DELETE FROM user_profile_reports WHERE generated_at<cutoff;
    DELETE FROM admin_traffic_reports WHERE report_date<cutoff::date;
    DELETE FROM admin_usage_daily_rollups WHERE date<cutoff::date;
    DELETE FROM plugin_usage_daily WHERE date<cutoff::date;
    DELETE FROM ingestion_repairs WHERE repaired_at<cutoff;
    DELETE FROM ingestion_session_owners o WHERE created_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.session_id=o.session_id)
        AND NOT EXISTS(SELECT 1 FROM session_transcripts t WHERE t.session_id=o.session_id)
        AND NOT EXISTS(SELECT 1 FROM plugin_session_summaries s WHERE s.session_id=o.session_id)
        AND NOT EXISTS(SELECT 1 FROM session_cost_snapshots c WHERE c.session_id=o.session_id);
    DELETE FROM feedback_ingestion_outbox q WHERE NOT pending AND (
        occurred_at<cutoff
        OR (event_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.id=q.event_id))
        OR (request_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM ai_requests r WHERE r.id=q.request_id))
    );
    UPDATE feedback_ingestion_backfill SET after_id='' WHERE done;
    PERFORM public.expire_reporting_sessions(cutoff);
    PERFORM public.finish_reporting_privacy(cutoff);
    PERFORM set_config('astound.feedback_retention','',true);
    RETURN deleted;
END $$;
