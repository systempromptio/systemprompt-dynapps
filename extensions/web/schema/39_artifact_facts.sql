-- Artifacts on the feedback plane.
--
-- Every tool result the platform ingests lands in core's `mcp_artifacts`
-- (one typed artifact per execution, joined to intent by the client's
-- tool_use_id). This file projects each one onto the feedback plane as an
-- `artifact` fact, so the Analysis pages count artifacts per skill and per
-- conversation through the same normalized-facts spine as invocations and
-- requests. It redefines the enqueue, key and normalisation functions from
-- 30_/31_ to add the kind; it must load after them and after 36_, which
-- redefines feedback_event_skill.

-- The outbox row for an artifact fact names the artifact by its own column
-- (declared in 30_feedback_ingestion.sql; added to established databases by
-- migration 076).
CREATE INDEX IF NOT EXISTS feedback_ingestion_artifact ON feedback_ingestion_outbox(artifact_id) WHERE artifact_id IS NOT NULL;

CREATE OR REPLACE FUNCTION feedback_fact_key(kind TEXT,event_key TEXT,request_key TEXT,consumer_key TEXT)
RETURNS JSONB LANGUAGE sql IMMUTABLE AS $$
SELECT jsonb_build_object('kind',kind,'source',CASE kind
 WHEN 'invocation' THEN 'astound.plugin_usage_events' WHEN 'request' THEN 'astound.gateway_requests'
 WHEN 'assessment' THEN 'astound.session_assessments' WHEN 'artifact' THEN 'astound.tool_artifacts'
 ELSE 'astound.resource_associations' END,
 'id',md5(jsonb_build_array(kind,event_key,request_key,consumer_key)::text))
$$;

CREATE OR REPLACE FUNCTION enqueue_feedback_artifact(artifact_key TEXT, consumer_key TEXT, session_key TEXT, event_time TIMESTAMPTZ DEFAULT NULL)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE key TEXT; was_pending BOOLEAN; original_time TIMESTAMPTZ;
BEGIN
    IF artifact_key IS NULL OR consumer_key IS NULL THEN RETURN; END IF;
    PERFORM 1 FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE;
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

-- The artifact's own invocation: the hook event that reported the same call.
CREATE OR REPLACE FUNCTION feedback_artifact_invocation(artifact_key TEXT)
RETURNS TEXT LANGUAGE sql STABLE AS $$
SELECT e.id FROM mcp_artifacts a
JOIN plugin_usage_events e ON e.tool_use_id=a.ai_tool_call_id AND e.user_id=a.user_id
 AND e.event_type IN ('PostToolUse','PostToolUseFailure')
WHERE a.artifact_id=artifact_key AND a.ai_tool_call_id IS NOT NULL
ORDER BY e.created_at DESC LIMIT 1
$$;

CREATE OR REPLACE FUNCTION feedback_artifact_request(artifact_key TEXT)
RETURNS TEXT LANGUAGE sql STABLE AS $$
SELECT t.request_id FROM mcp_artifacts a
JOIN ai_request_tool_calls t ON t.ai_tool_call_id=a.ai_tool_call_id
JOIN ai_requests r ON r.id=t.request_id AND r.user_id=a.user_id AND NOT r.synthetic AND r.request_kind='turn'
WHERE a.artifact_id=artifact_key AND a.ai_tool_call_id IS NOT NULL
ORDER BY t.created_at DESC LIMIT 1
$$;

-- The skill the artifact belongs to: the most recent skill invocation in the
-- same session before the result was produced.
CREATE OR REPLACE FUNCTION feedback_artifact_skill(artifact_key TEXT)
RETURNS JSONB LANGUAGE sql STABLE AS $$
SELECT feedback_event_skill(s.id) FROM mcp_artifacts a
JOIN analysis_skill_events s ON s.user_id=a.user_id AND s.session_id=a.session_id AND s.invoked_at<=a.created_at
WHERE a.artifact_id=artifact_key
ORDER BY s.invoked_at DESC LIMIT 1
$$;

CREATE OR REPLACE FUNCTION feedback_normalized_artifact(queue_id TEXT)
RETURNS JSONB LANGUAGE plpgsql STABLE AS $$
DECLARE q feedback_ingestion_outbox; a mcp_artifacts; key JSONB; invocation_key TEXT; request_key TEXT;
BEGIN
    SELECT * INTO q FROM feedback_ingestion_outbox WHERE id=queue_id;
    SELECT * INTO a FROM mcp_artifacts WHERE artifact_id=q.artifact_id AND user_id=q.user_id;
    IF NOT FOUND THEN RETURN NULL; END IF;
    key := feedback_fact_key('artifact',q.artifact_id,NULL,q.user_id);
    invocation_key := feedback_artifact_invocation(a.artifact_id);
    request_key := feedback_artifact_request(a.artifact_id);
    RETURN jsonb_build_object('kind','artifact','value',jsonb_strip_nulls(jsonb_build_object(
        'artifact_key',key,
        'execution_id',a.mcp_execution_id,
        'invocation_key',CASE WHEN invocation_key IS NULL THEN NULL ELSE feedback_fact_key('invocation',invocation_key,NULL,q.user_id) END,
        'request_key',CASE WHEN request_key IS NULL THEN NULL ELSE feedback_fact_key('request',NULL,request_key,q.user_id) END,
        'occurred_at',a.created_at,
        'consumer',CASE WHEN invocation_key IS NULL THEN jsonb_build_object('status','historical_unknown') ELSE feedback_event_identity(invocation_key) END,
        'skill',feedback_artifact_skill(a.artifact_id),
        'tool_name',COALESCE(a.tool_name,'unknown'),
        'server_name',a.server_name,
        'artifact_type',a.artifact_type,
        'source',a.source,
        'correlation',(SELECT correlation FROM mcp_tool_executions x WHERE x.mcp_execution_id=a.mcp_execution_id),
        'is_structured',a.is_structured,
        'has_ui_resource',a.has_ui_resource,
        'succeeded',NOT a.is_error,
        'payload_bytes',a.payload_bytes,
        'findings',(SELECT count(*) FROM mcp_artifact_findings f WHERE f.artifact_id=a.artifact_id))));
END $$;

-- Re-declared with the artifact branch in front; every other kind is the
-- body 31_feedback_projection.sql (and 36_) already defined.
CREATE OR REPLACE FUNCTION feedback_normalized_fact_base(queue_id TEXT)
RETURNS JSONB LANGUAGE plpgsql STABLE AS $$
DECLARE q feedback_ingestion_outbox; e RECORD; r RECORD; a RECORD; key JSONB; value JSONB;
BEGIN
    SELECT * INTO q FROM feedback_ingestion_outbox WHERE id=queue_id;
    key := feedback_fact_key(q.kind,q.event_id,q.request_id,q.user_id);
    IF q.kind IN ('invocation','assessment','resource_association') THEN
        SELECT s.*,p.event_type INTO e FROM analysis_skill_events s JOIN plugin_usage_events p ON p.id=s.id WHERE s.id=q.event_id AND s.user_id=q.user_id;
        IF NOT FOUND THEN RETURN NULL; END IF;
    END IF;
    IF q.kind IN ('request','resource_association') THEN
        SELECT * INTO r FROM ai_requests WHERE id=q.request_id AND user_id=q.user_id AND NOT synthetic AND request_kind='turn';
        IF NOT FOUND THEN RETURN NULL; END IF;
    END IF;
    IF q.kind='invocation' THEN
        value := jsonb_build_object('invocation_id',q.id,'occurred_at',e.invoked_at,'consumer',feedback_event_identity(e.id),
            'attribution',feedback_event_attribution(e.id),'skill',feedback_event_skill(e.id),
            'succeeded',e.event_type<>'PostToolUseFailure','latency_micros',NULL);
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
        SELECT * INTO a FROM evaluation_runs WHERE user_id=e.user_id AND session_id=e.session_id AND criterion='quality' AND status='succeeded'
            ORDER BY created_at DESC LIMIT 1;
        IF NOT FOUND THEN RETURN NULL; END IF;
        value := jsonb_build_object('assessment_key',key,'invocation_key',feedback_fact_key('invocation',e.id,NULL,e.user_id),
            'conversation_key',jsonb_build_object('source','astound.conversations','id',md5(jsonb_build_array(e.user_id,e.session_id)::text)),
            'occurred_at',a.created_at,'outcome',CASE WHEN a.score BETWEEN 0 AND 100 THEN jsonb_build_object('status','scored','score_millionths',a.score::bigint*10000)
             ELSE jsonb_build_object('status','unavailable') END);
    END IF;
    RETURN jsonb_build_object('kind',q.kind,'value',value);
END $$;

CREATE OR REPLACE FUNCTION feedback_normalized_fact(queue_id TEXT)
RETURNS JSONB LANGUAGE plpgsql STABLE AS $$
DECLARE fact_kind TEXT;
BEGIN
    SELECT kind INTO fact_kind FROM feedback_ingestion_outbox WHERE id=queue_id;
    IF fact_kind='artifact' THEN RETURN feedback_normalized_artifact(queue_id); END IF;
    RETURN feedback_normalized_fact_base(queue_id);
END $$;

-- Every artifact written or re-linked is a fact; a session re-enqueue picks
-- its artifacts up with everything else.
CREATE OR REPLACE FUNCTION capture_feedback_artifact()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB;
BEGIN
    IF TG_OP='DELETE' AND current_setting('astound.feedback_retention',true)='expiry' THEN RETURN OLD; END IF;
    data := CASE WHEN TG_OP='DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
    IF data->>'user_id' IS NULL THEN RETURN COALESCE(NEW,OLD); END IF;
    PERFORM enqueue_feedback_artifact(data->>'artifact_id',data->>'user_id',data->>'session_id',(data->>'created_at')::timestamptz);
    RETURN COALESCE(NEW,OLD);
END $$;
DROP TRIGGER IF EXISTS feedback_capture_artifact ON mcp_artifacts;
CREATE TRIGGER feedback_capture_artifact AFTER INSERT OR UPDATE OR DELETE ON mcp_artifacts FOR EACH ROW EXECUTE FUNCTION capture_feedback_artifact();

CREATE OR REPLACE FUNCTION enqueue_feedback_session(consumer_key TEXT, session_key TEXT)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE row RECORD;
BEGIN
    IF session_key IS NULL THEN RETURN; END IF;
    PERFORM 1 FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE;
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
