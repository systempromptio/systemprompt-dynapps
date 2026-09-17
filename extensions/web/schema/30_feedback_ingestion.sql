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

