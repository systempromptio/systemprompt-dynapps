CREATE TABLE IF NOT EXISTS feedback_ingestion_outbox (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK(kind IN ('invocation','request','assessment','resource_association')),
    event_id TEXT, request_id TEXT,
    user_id TEXT NOT NULL, native_session_id TEXT,
    revision BIGINT NOT NULL DEFAULT 1,
    pending BOOLEAN NOT NULL DEFAULT true,
    lease_token TEXT, lease_until TIMESTAMPTZ,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_attempt TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    last_error TEXT
);
CREATE INDEX IF NOT EXISTS feedback_ingestion_pending ON feedback_ingestion_outbox(next_attempt,id) WHERE pending;
CREATE INDEX IF NOT EXISTS feedback_ingestion_session ON feedback_ingestion_outbox(user_id,native_session_id);
CREATE TABLE IF NOT EXISTS feedback_ingestion_backfill (
    source TEXT PRIMARY KEY, after_id TEXT NOT NULL DEFAULT '', done BOOLEAN NOT NULL DEFAULT false
);
INSERT INTO feedback_ingestion_backfill(source) VALUES('plugin_usage_events'),('ai_requests'),('session_analyses') ON CONFLICT DO NOTHING;
INSERT INTO analytics_ingestion_producers(producer,pending_count,oldest_pending_at)
VALUES('astound.feedback',3,clock_timestamp()) ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS feedback_authenticated_events (
    event_id TEXT PRIMARY KEY REFERENCES plugin_usage_events(id) ON DELETE CASCADE,
    consumer_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL REFERENCES user_device_certs(id) ON DELETE CASCADE,
    host TEXT NOT NULL,
    native_session_id TEXT NOT NULL,
    authenticated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE IF NOT EXISTS feedback_ingestion_evidence (
    event_id TEXT PRIMARY KEY REFERENCES plugin_usage_events(id) ON DELETE CASCADE,
    evidence JSONB NOT NULL,
    retained_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
DROP TRIGGER IF EXISTS feedback_ingestion_evidence_immutable ON feedback_ingestion_evidence;
CREATE TRIGGER feedback_ingestion_evidence_immutable BEFORE UPDATE ON feedback_ingestion_evidence FOR EACH ROW EXECUTE FUNCTION reject_managed_content_update();
DROP TRIGGER IF EXISTS feedback_authenticated_events_immutable ON feedback_authenticated_events;
CREATE TRIGGER feedback_authenticated_events_immutable BEFORE UPDATE ON feedback_authenticated_events FOR EACH ROW EXECUTE FUNCTION reject_managed_content_update();

CREATE OR REPLACE FUNCTION enqueue_feedback_fact(fact_kind TEXT, event_key TEXT, request_key TEXT, consumer_key TEXT, session_key TEXT, event_time TIMESTAMPTZ DEFAULT NULL)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE key TEXT; was_pending BOOLEAN; original_time TIMESTAMPTZ;
BEGIN
    PERFORM 1 FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE;
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

CREATE OR REPLACE FUNCTION enqueue_feedback_session(consumer_key TEXT, session_key TEXT)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE row RECORD;
BEGIN
    IF session_key IS NULL THEN RETURN; END IF;
    PERFORM 1 FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE;
    FOR row IN SELECT kind,event_id,request_id FROM feedback_ingestion_outbox WHERE user_id=consumer_key AND native_session_id=session_key LOOP
        PERFORM enqueue_feedback_fact(row.kind,row.event_id,row.request_id,consumer_key,session_key);
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
END $$;

CREATE OR REPLACE FUNCTION capture_feedback_source()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB; old_data JSONB; session_key TEXT; linked RECORD;
BEGIN
    IF TG_OP='DELETE' AND current_setting('astound.feedback_retention',true)='expiry' THEN RETURN OLD; END IF;
    PERFORM 1 FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE;
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
DROP TRIGGER IF EXISTS feedback_capture ON plugin_usage_events;
CREATE TRIGGER feedback_capture AFTER INSERT OR UPDATE OR DELETE ON plugin_usage_events FOR EACH ROW EXECUTE FUNCTION capture_feedback_source();
DROP TRIGGER IF EXISTS feedback_capture ON ai_requests;
CREATE TRIGGER feedback_capture AFTER INSERT OR UPDATE OR DELETE ON ai_requests FOR EACH ROW EXECUTE FUNCTION capture_feedback_source();
DROP TRIGGER IF EXISTS feedback_capture ON session_analyses;
CREATE TRIGGER feedback_capture AFTER INSERT OR UPDATE OR DELETE ON session_analyses FOR EACH ROW EXECUTE FUNCTION capture_feedback_source();

CREATE OR REPLACE FUNCTION capture_feedback_identity()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB; row RECORD;
BEGIN
    IF TG_OP='DELETE' AND current_setting('astound.feedback_retention',true)='expiry' THEN RETURN OLD; END IF;
    PERFORM 1 FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE;
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
DROP TRIGGER IF EXISTS feedback_capture_identity ON feedback_authenticated_events;
CREATE TRIGGER feedback_capture_identity AFTER INSERT OR UPDATE OR DELETE ON feedback_authenticated_events FOR EACH ROW EXECUTE FUNCTION capture_feedback_identity();
DROP TRIGGER IF EXISTS feedback_capture_identity ON managed_consumer_session_bindings;
CREATE TRIGGER feedback_capture_identity AFTER INSERT OR UPDATE OR DELETE ON managed_consumer_session_bindings FOR EACH ROW EXECUTE FUNCTION capture_feedback_identity();
DROP TRIGGER IF EXISTS feedback_capture_identity ON managed_consumer_invocation_evidence;
CREATE TRIGGER feedback_capture_identity AFTER INSERT OR UPDATE OR DELETE ON managed_consumer_invocation_evidence FOR EACH ROW EXECUTE FUNCTION capture_feedback_identity();
DROP TRIGGER IF EXISTS feedback_capture_identity ON managed_consumer_attribution_projection;
CREATE TRIGGER feedback_capture_identity AFTER INSERT OR UPDATE OR DELETE ON managed_consumer_attribution_projection FOR EACH ROW EXECUTE FUNCTION capture_feedback_identity();

CREATE OR REPLACE FUNCTION capture_feedback_tool_request()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB; row RECORD;
BEGIN
    IF TG_OP='DELETE' AND current_setting('astound.feedback_retention',true)='expiry' THEN RETURN OLD; END IF;
    data := CASE WHEN TG_OP='DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
    FOR row IN SELECT DISTINCT e.user_id,e.session_id FROM analysis_skill_events e JOIN ai_requests r ON r.user_id=e.user_id WHERE r.id=data->>'request_id' LOOP
        PERFORM enqueue_feedback_session(row.user_id,row.session_id);
    END LOOP;
    RETURN COALESCE(NEW,OLD);
END $$;
DROP TRIGGER IF EXISTS feedback_capture_tool ON ai_request_tool_calls;
CREATE TRIGGER feedback_capture_tool AFTER INSERT OR UPDATE OR DELETE ON ai_request_tool_calls FOR EACH ROW EXECUTE FUNCTION capture_feedback_tool_request();

CREATE OR REPLACE FUNCTION capture_feedback_grant()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB; row RECORD;
BEGIN
    data := CASE WHEN TG_OP='DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
    FOR row IN SELECT DISTINCT user_id,session_id FROM plugin_usage_events WHERE user_id=data->>'consumer_id' LOOP
        PERFORM enqueue_feedback_session(row.user_id,row.session_id);
    END LOOP;
    RETURN COALESCE(NEW,OLD);
END $$;
DROP TRIGGER IF EXISTS feedback_capture_grant ON managed_consumer_grants;
CREATE TRIGGER feedback_capture_grant AFTER INSERT OR UPDATE OR DELETE ON managed_consumer_grants FOR EACH ROW EXECUTE FUNCTION capture_feedback_grant();

CREATE OR REPLACE FUNCTION capture_feedback_traffic()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB;
BEGIN
    data := CASE WHEN TG_OP='DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
    PERFORM enqueue_feedback_session(data->>'owner_id',data->>'session_id');
    RETURN COALESCE(NEW,OLD);
END $$;
DROP TRIGGER IF EXISTS feedback_capture_traffic ON eval_session_bindings;
CREATE TRIGGER feedback_capture_traffic AFTER INSERT OR UPDATE OR DELETE ON eval_session_bindings FOR EACH ROW EXECUTE FUNCTION capture_feedback_traffic();

CREATE OR REPLACE FUNCTION feedback_fact_key(kind TEXT,event_key TEXT,request_key TEXT,consumer_key TEXT)
RETURNS JSONB LANGUAGE sql IMMUTABLE AS $$
SELECT jsonb_build_object('kind',kind,'source',CASE kind
 WHEN 'invocation' THEN 'astound.plugin_usage_events' WHEN 'request' THEN 'astound.gateway_requests'
 WHEN 'assessment' THEN 'astound.session_assessments' ELSE 'astound.resource_associations' END,
 'id',md5(jsonb_build_array(kind,event_key,request_key,consumer_key)::text))
$$;

CREATE OR REPLACE FUNCTION feedback_event_identity(event_key TEXT)
RETURNS JSONB LANGUAGE sql STABLE AS $$
WITH identities AS (
 SELECT a.consumer_id,a.device_id,a.host,a.native_session_id
 FROM feedback_authenticated_events a JOIN plugin_usage_events e ON e.id=a.event_id
 WHERE a.event_id=event_key AND a.consumer_id=e.user_id AND a.native_session_id=e.session_id
 UNION
 SELECT a.consumer_id,a.device_id,a.host,a.native_session_id
 FROM managed_consumer_invocation_evidence a JOIN plugin_usage_events e
 ON e.id=a.invocation_id AND e.user_id=a.consumer_id AND e.session_id=a.native_session_id
 WHERE e.id=event_key
)
SELECT CASE WHEN count(*)=1 THEN (jsonb_agg(jsonb_build_object('status','authenticated','consumer_id',consumer_id,'device_id',device_id,'host',host,'session_id',native_session_id)))->0
 ELSE jsonb_build_object('status','historical_unknown') END FROM identities
$$;

CREATE OR REPLACE FUNCTION feedback_event_receipt(event_key TEXT)
RETURNS JSONB LANGUAGE sql STABLE AS $$
WITH identity AS (SELECT feedback_event_identity(event_key) AS value), matches AS (
 SELECT DISTINCT r.id,r.installation_id,r.generation,r.resource_id,p.revision_id
 FROM analysis_skill_events e CROSS JOIN identity i
 JOIN managed_consumer_session_bindings b ON b.consumer_id=i.value->>'consumer_id'
 AND b.device_id=i.value->>'device_id' AND b.host=i.value->>'host' AND b.native_session_id=i.value->>'session_id'
 JOIN managed_installation_receipts r ON r.id=b.receipt_id AND r.consumer_id=b.consumer_id AND r.device_id=b.device_id AND r.host=b.host
 JOIN managed_publications p ON p.id=r.publication_id AND p.owner_id=r.owner_id
 JOIN managed_resources m ON m.id=r.resource_id AND m.owner_id=r.owner_id
 JOIN managed_consumer_grants g ON g.resource_id=r.resource_id AND g.owner_id=r.owner_id AND g.consumer_id=r.consumer_id AND g.revoked_at IS NULL
 WHERE e.id=event_key AND r.fully_verified AND jsonb_array_length(COALESCE(r.consumer_evidence->'runtime_files','[]'::jsonb))>0
 AND replace(m.resource_key,'-','_')=replace(split_part(e.skill,':',2),'-','_')
)
SELECT CASE WHEN count(*)=1 THEN (jsonb_agg(jsonb_build_object('receipt_id',id,'installation_id',installation_id,'generation',generation,'resource_id',resource_id,'revision_id',revision_id)))->0
 ELSE NULL END FROM matches
$$;

CREATE OR REPLACE FUNCTION feedback_event_attribution(event_key TEXT)
RETURNS JSONB LANGUAGE sql STABLE AS $$
SELECT CASE WHEN value IS NULL THEN jsonb_build_object('status','unknown')
 ELSE jsonb_build_object('status','verified','resource_id',value->>'resource_id','revision_id',value->>'revision_id') END
FROM (SELECT feedback_event_receipt(event_key) AS value) receipt
$$;

CREATE OR REPLACE FUNCTION feedback_normalized_fact(queue_id TEXT)
RETURNS JSONB LANGUAGE plpgsql STABLE AS $$
DECLARE q feedback_ingestion_outbox; e RECORD; r RECORD; a RECORD; key JSONB; value JSONB;
BEGIN
    SELECT * INTO q FROM feedback_ingestion_outbox WHERE id=queue_id;
    key := feedback_fact_key(q.kind,q.event_id,q.request_id,q.user_id);
    IF q.kind IN ('invocation','assessment','resource_association') THEN
        SELECT s.*,p.event_type INTO e FROM analysis_skill_events s JOIN plugin_usage_events p ON p.id=s.id WHERE s.id=q.event_id AND s.user_id=q.user_id AND NOT EXISTS(SELECT 1 FROM eval_session_bindings b WHERE b.owner_id=s.user_id AND b.session_id=s.session_id AND b.traffic_class<>'production');
        IF NOT FOUND THEN RETURN NULL; END IF;
    END IF;
    IF q.kind IN ('request','resource_association') THEN
        SELECT * INTO r FROM ai_requests WHERE id=q.request_id AND user_id=q.user_id AND NOT synthetic AND request_kind='turn' AND NOT EXISTS(SELECT 1 FROM eval_session_bindings b WHERE b.owner_id=ai_requests.user_id AND b.session_id IN(ai_requests.session_id,ai_requests.client_session_id) AND b.traffic_class<>'production');
        IF NOT FOUND THEN RETURN NULL; END IF;
    END IF;
    IF q.kind='invocation' THEN
        value := jsonb_build_object('invocation_id',q.id,'occurred_at',e.invoked_at,'consumer',feedback_event_identity(e.id),
            'attribution',feedback_event_attribution(e.id),'succeeded',e.event_type<>'PostToolUseFailure','latency_micros',NULL);
    ELSIF q.kind='request' THEN
        value := jsonb_build_object('request_key',key,'occurred_at',r.created_at,'consumer',jsonb_build_object('status','historical_unknown'),
            'succeeded',r.status='completed','spend',CASE WHEN r.cost_microdollars>0 THEN jsonb_build_object('status','known','currency','USD','amount_micros',r.cost_microdollars)
             ELSE jsonb_build_object('status','unknown_pricing') END,
            'input_tokens',CASE WHEN r.input_tokens>=0 THEN r.input_tokens::bigint+COALESCE(r.cache_read_tokens,0)+COALESCE(r.cache_creation_tokens,0) ELSE NULL END,
            'output_tokens',CASE WHEN r.output_tokens>=0 THEN r.output_tokens ELSE NULL END,
            'latency_micros',CASE WHEN r.latency_ms>=0 THEN r.latency_ms::bigint*1000 ELSE NULL END);
    ELSIF q.kind='resource_association' THEN
        IF r.client_session_id IS DISTINCT FROM e.session_id AND NOT(r.client_session_id IS NULL AND EXISTS(
          SELECT 1 FROM ai_request_tool_calls t WHERE t.request_id=r.id AND t.ai_tool_call_id=e.tool_use_id)) THEN RETURN NULL; END IF;
        IF r.client_session_id IS NULL AND EXISTS(SELECT 1 FROM analysis_skill_events other WHERE other.user_id=e.user_id AND other.tool_use_id=e.tool_use_id AND other.session_id<>e.session_id) THEN RETURN NULL; END IF;
        value := jsonb_build_object('association_key',key,'invocation_key',feedback_fact_key('invocation',e.id,NULL,e.user_id),
            'request_key',feedback_fact_key('request',NULL,r.id,r.user_id),'occurred_at',e.invoked_at,'attribution',feedback_event_attribution(e.id));
    ELSE
        SELECT * INTO a FROM session_analyses WHERE user_id=e.user_id AND session_id=e.session_id;
        IF NOT FOUND THEN RETURN NULL; END IF;
        value := jsonb_build_object('assessment_key',key,'invocation_key',feedback_fact_key('invocation',e.id,NULL,e.user_id),
            'conversation_key',jsonb_build_object('source','astound.conversations','id',md5(jsonb_build_array(e.user_id,e.session_id)::text)),
            'occurred_at',a.created_at,'outcome',CASE WHEN a.outcome IN ('failed','failure') THEN jsonb_build_object('status','failed')
             WHEN a.quality_score BETWEEN 1 AND 100 THEN jsonb_build_object('status','scored','score_millionths',a.quality_score::bigint*10000)
             ELSE jsonb_build_object('status','unavailable') END);
    END IF;
    RETURN jsonb_build_object('kind',q.kind,'value',value);
END $$;

DROP TRIGGER IF EXISTS attribute_skill_version ON plugin_usage_events;
DROP FUNCTION IF EXISTS attribute_ingested_skill_invocation();
CREATE OR REPLACE VIEW analysis_skill_version_events AS
SELECT e.id AS invocation_id,e.user_id,e.session_id,e.plugin_id,e.skill,
       e.tool_use_id,e.source,e.invoked_at,a.value->>'installation_id' AS installation_id,
       a.value->>'resource_id' AS resource_id,a.value->>'revision_id' AS revision_id,
       (a.value->>'generation')::bigint AS publication_generation,'production'::text AS traffic_class,
       CASE WHEN a.value IS NOT NULL THEN 'verified' ELSE 'revision_unknown' END AS attribution_status
FROM analysis_skill_events e
CROSS JOIN LATERAL (SELECT feedback_event_receipt(e.id) AS value) a;

CREATE OR REPLACE FUNCTION expire_feedback_raw(cutoff TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE deleted BIGINT; producer_row RECORD;
BEGIN
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
    DELETE FROM user_commits WHERE committed_at<cutoff;
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
    PERFORM set_config('astound.feedback_retention','',true);
    RETURN deleted;
END $$;
