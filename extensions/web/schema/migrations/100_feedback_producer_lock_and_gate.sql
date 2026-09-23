-- Feedback ingestion no longer serialises on the producer row, and retention
-- no longer gates on a counter.
--
-- Every per-row enqueue path took `SELECT ... FROM analytics_ingestion_producers
-- ... FOR UPDATE` before touching the outbox, so every hook event, gateway
-- request and artifact in flight queued behind one row. The outbox upsert is
-- keyed by md5 and the counter is recounted on each drain settle (migration
-- 084), so that lock bought no correctness — it is gone from
-- enqueue_feedback_fact, enqueue_feedback_session, enqueue_feedback_artifact,
-- capture_feedback_source and capture_feedback_identity. The per-key
-- `feedback_ingestion_outbox ... FOR UPDATE` that guards the was_pending
-- decision stays.
--
-- expire_feedback_raw keeps the producer lock (it takes the tables in
-- EXCLUSIVE mode anyway) but stops asking `pending_count<>0`: a counter that
-- drifted upward wedged retention forever. It now asks the queue itself --
-- the same predicate feedback_producer_recount counts, so an undone backfill
-- source still holds retention back.
--
-- Idempotent: CREATE OR REPLACE of each whole body, identical to the base
-- schema files 30_feedback_ingestion.sql, 32_feedback_retention.sql and
-- 39_artifact_facts.sql.

CREATE OR REPLACE FUNCTION enqueue_feedback_fact(fact_kind TEXT, event_key TEXT, request_key TEXT, consumer_key TEXT, session_key TEXT, event_time TIMESTAMPTZ DEFAULT NULL)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE key TEXT; was_pending BOOLEAN; original_time TIMESTAMPTZ;
BEGIN
    key := md5(jsonb_build_array(fact_kind,event_key,request_key,consumer_key)::text);
    original_time := COALESCE(event_time,(SELECT created_at FROM plugin_usage_events WHERE id=event_key),(SELECT created_at FROM ai_requests WHERE id=request_key));
    SELECT pending INTO was_pending FROM feedback_ingestion_outbox WHERE id=key FOR UPDATE;
    INSERT INTO feedback_ingestion_outbox(id,kind,event_id,request_id,user_id,native_session_id,occurred_at)
    VALUES(key,fact_kind,event_key,request_key,consumer_key,session_key,COALESCE(original_time,clock_timestamp()))
    ON CONFLICT(id) DO UPDATE SET revision=feedback_ingestion_outbox.revision+1,pending=true,
        lease_token=NULL,lease_until=NULL,next_attempt=clock_timestamp(),updated_at=clock_timestamp(),native_session_id=EXCLUDED.native_session_id;
    IF was_pending IS DISTINCT FROM true THEN
        UPDATE analytics_ingestion_producers SET pending_count=pending_count+1,
            oldest_pending_at=COALESCE(oldest_pending_at,clock_timestamp()),updated_at=clock_timestamp()
        WHERE producer='astound.feedback';
    END IF;
END $$;

CREATE OR REPLACE FUNCTION capture_feedback_source()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB; old_data JSONB; session_key TEXT; linked RECORD;
BEGIN
    IF TG_OP='DELETE' AND current_setting('astound.feedback_retention',true)='expiry' THEN RETURN OLD; END IF;
    data := CASE WHEN TG_OP='DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
    session_key := CASE WHEN TG_TABLE_NAME='ai_requests' THEN data->>'client_session_id' ELSE data->>'session_id' END;
    IF TG_TABLE_NAME='ai_requests' THEN
        PERFORM enqueue_feedback_fact('request',NULL,data->>'id',data->>'user_id',session_key,(data->>'created_at')::timestamptz);
        FOR linked IN SELECT * FROM feedback_ingestion_outbox WHERE request_id=data->>'id' AND kind='resource_association' LOOP
            PERFORM enqueue_feedback_fact(linked.kind,linked.event_id,linked.request_id,linked.user_id,linked.native_session_id,linked.occurred_at);
        END LOOP;
    ELSIF TG_TABLE_NAME='plugin_usage_events' THEN
        IF TG_OP <> 'DELETE' THEN
            INSERT INTO feedback_ingestion_evidence(event_id,evidence) VALUES(data->>'id',CASE WHEN TG_OP='UPDATE' THEN to_jsonb(OLD) ELSE data END) ON CONFLICT DO NOTHING;
        END IF;
        PERFORM enqueue_feedback_fact('invocation',data->>'id',NULL,data->>'user_id',session_key,(data->>'created_at')::timestamptz);
        PERFORM enqueue_feedback_fact('assessment',data->>'id',NULL,data->>'user_id',session_key);
    END IF;
    PERFORM enqueue_feedback_session(data->>'user_id',session_key);
    IF TG_OP='UPDATE' THEN
        old_data := to_jsonb(OLD);
        IF TG_TABLE_NAME='ai_requests' THEN
            PERFORM enqueue_feedback_fact('request',NULL,old_data->>'id',old_data->>'user_id',old_data->>'client_session_id',(old_data->>'created_at')::timestamptz);
        END IF;
        PERFORM enqueue_feedback_session(old_data->>'user_id',CASE WHEN TG_TABLE_NAME='ai_requests' THEN old_data->>'client_session_id' ELSE old_data->>'session_id' END);
    END IF;
    RETURN COALESCE(NEW,OLD);
END $$;

CREATE OR REPLACE FUNCTION capture_feedback_identity()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB; row RECORD;
BEGIN
    IF TG_OP='DELETE' AND current_setting('astound.feedback_retention',true)='expiry' THEN RETURN OLD; END IF;
    data := CASE WHEN TG_OP='DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
    IF TG_TABLE_NAME='managed_consumer_attribution_projection' THEN
        FOR row IN SELECT consumer_id,native_session_id FROM managed_consumer_invocation_evidence WHERE id=data->>'evidence_id' LOOP
            PERFORM enqueue_feedback_session(row.consumer_id,row.native_session_id);
        END LOOP;
    ELSE
        PERFORM enqueue_feedback_session(data->>'consumer_id',data->>'native_session_id');
    END IF;
    RETURN COALESCE(NEW,OLD);
END $$;

CREATE OR REPLACE FUNCTION enqueue_feedback_artifact(artifact_key TEXT, consumer_key TEXT, session_key TEXT, event_time TIMESTAMPTZ DEFAULT NULL)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE key TEXT; was_pending BOOLEAN; original_time TIMESTAMPTZ;
BEGIN
    IF artifact_key IS NULL OR consumer_key IS NULL THEN RETURN; END IF;
    key := md5(jsonb_build_array('artifact',artifact_key,NULL,consumer_key)::text);
    original_time := COALESCE(event_time,(SELECT created_at FROM mcp_artifacts WHERE artifact_id=artifact_key));
    SELECT pending INTO was_pending FROM feedback_ingestion_outbox WHERE id=key FOR UPDATE;
    INSERT INTO feedback_ingestion_outbox(id,kind,artifact_id,user_id,native_session_id,occurred_at)
    VALUES(key,'artifact',artifact_key,consumer_key,session_key,COALESCE(original_time,clock_timestamp()))
    ON CONFLICT(id) DO UPDATE SET revision=feedback_ingestion_outbox.revision+1,pending=true,
        lease_token=NULL,lease_until=NULL,next_attempt=clock_timestamp(),updated_at=clock_timestamp(),native_session_id=EXCLUDED.native_session_id;
    IF was_pending IS DISTINCT FROM true THEN
        UPDATE analytics_ingestion_producers SET pending_count=pending_count+1,
            oldest_pending_at=COALESCE(oldest_pending_at,clock_timestamp()),updated_at=clock_timestamp()
        WHERE producer='astound.feedback';
    END IF;
END $$;

CREATE OR REPLACE FUNCTION enqueue_feedback_session(consumer_key TEXT, session_key TEXT)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE row RECORD;
BEGIN
    IF session_key IS NULL THEN RETURN; END IF;
    FOR row IN SELECT kind,event_id,request_id,artifact_id FROM feedback_ingestion_outbox WHERE user_id=consumer_key AND native_session_id=session_key LOOP
        IF row.kind='artifact' THEN
            PERFORM enqueue_feedback_artifact(row.artifact_id,consumer_key,session_key);
        ELSE
            PERFORM enqueue_feedback_fact(row.kind,row.event_id,row.request_id,consumer_key,session_key);
        END IF;
    END LOOP;
    FOR row IN SELECT id FROM analysis_skill_events WHERE user_id=consumer_key AND session_id=session_key LOOP
        PERFORM enqueue_feedback_fact('invocation',row.id,NULL,consumer_key,session_key);
        PERFORM enqueue_feedback_fact('assessment',row.id,NULL,consumer_key,session_key);
    END LOOP;
    FOR row IN SELECT id FROM ai_requests WHERE user_id=consumer_key AND client_session_id=session_key LOOP
        PERFORM enqueue_feedback_fact('request',NULL,row.id,consumer_key,session_key);
    END LOOP;
    FOR row IN SELECT DISTINCT e.id AS event_id,r.id AS request_id FROM analysis_skill_events e
        JOIN ai_requests r ON r.user_id=e.user_id AND (r.client_session_id=e.session_id OR (r.client_session_id IS NULL AND EXISTS(
            SELECT 1 FROM ai_request_tool_calls t WHERE t.request_id=r.id AND t.ai_tool_call_id=e.tool_use_id)))
        WHERE e.user_id=consumer_key AND e.session_id=session_key LOOP
        PERFORM enqueue_feedback_fact('resource_association',row.event_id,row.request_id,consumer_key,session_key);
    END LOOP;
    FOR row IN SELECT artifact_id FROM mcp_artifacts WHERE user_id=consumer_key AND session_id=session_key LOOP
        PERFORM enqueue_feedback_artifact(row.artifact_id,consumer_key,session_key);
    END LOOP;
END $$;

CREATE OR REPLACE FUNCTION expire_feedback_raw(cutoff TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE deleted BIGINT;
BEGIN
    PERFORM public.prepare_reporting_privacy();
    LOCK TABLE analytics_ingestion_producers,analytics_fact_checkpoints,analytics_fact_backfills,
        analytics_fact_changes,analytics_fact_consumers,analytics_fact_deltas IN EXCLUSIVE MODE;
    IF cutoff > clock_timestamp()-interval '90 days' THEN
        RAISE EXCEPTION 'Feedback retention cutoff is too recent' USING ERRCODE='23514';
    END IF;
    PERFORM 1 FROM analytics_ingestion_producers ORDER BY producer FOR UPDATE;
    IF EXISTS(SELECT 1 FROM feedback_ingestion_outbox WHERE pending)
       OR EXISTS(SELECT 1 FROM feedback_ingestion_backfill WHERE NOT done) THEN
        RAISE EXCEPTION 'Feedback retention waits for pending producers' USING ERRCODE='55000';
    END IF;
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
    DELETE FROM managed_invocation_attributions WHERE invocation_id IN(SELECT id FROM plugin_usage_events WHERE created_at<cutoff);
    DELETE FROM managed_consumer_invocation_evidence WHERE occurred_at<cutoff;
    DELETE FROM managed_consumer_session_bindings b WHERE bound_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM managed_consumer_invocation_evidence e WHERE e.consumer_id=b.consumer_id AND e.device_id=b.device_id AND e.host=b.host AND e.native_session_id=b.native_session_id)
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.user_id=b.consumer_id AND e.session_id=b.native_session_id AND e.created_at>=cutoff);
    DELETE FROM ingestion_event_receipts WHERE accepted_at<cutoff;
    DELETE FROM plugin_usage_events WHERE created_at<cutoff;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    DELETE FROM ai_requests WHERE created_at<cutoff;
    DELETE FROM conversation_analyses WHERE created_at<cutoff;
    DELETE FROM session_transcripts WHERE captured_at<cutoff;
    DELETE FROM session_cost_snapshots WHERE updated_at<cutoff;
    DELETE FROM plugin_session_summaries WHERE COALESCE(ended_at,updated_at,created_at)<cutoff;
    DELETE FROM session_entity_links WHERE last_seen_at<cutoff;
    DELETE FROM session_ratings WHERE created_at<cutoff;
    DELETE FROM daily_summaries WHERE summary_date<cutoff::date;
    DELETE FROM user_profile_reports WHERE generated_at<cutoff;
    DELETE FROM admin_traffic_reports WHERE report_date<cutoff::date;
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
