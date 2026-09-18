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

-- Redefined by 36_marketplace_versions.sql once `marketplace_versions` exists,
-- adding the marketplace hash; this is the body without it.
CREATE OR REPLACE FUNCTION feedback_event_skill(event_key TEXT)
RETURNS JSONB LANGUAGE sql STABLE AS $$
SELECT CASE WHEN e.plugin_id IS NULL OR e.skill IS NULL OR e.skill='' THEN NULL
 ELSE jsonb_strip_nulls(jsonb_build_object('plugin_id',e.plugin_id,'skill',e.skill,
  'marketplace_id',o.marketplace_id,'source',o.source,'source_hash',s.content_hash)) END
FROM analysis_skill_events e
LEFT JOIN service_owned_ids o ON o.kind='plugin' AND o.id=e.plugin_id
LEFT JOIN service_sources s ON s.name=o.source
WHERE e.id=event_key
$$;

CREATE OR REPLACE FUNCTION feedback_normalized_fact(queue_id TEXT)
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
        SELECT j.* INTO a FROM conversation_analyses j
            WHERE j.user_id=e.user_id AND j.status='classified' AND j.completion IS NOT NULL
              AND EXISTS(SELECT 1 FROM ai_requests cr WHERE cr.context_id=j.context_id AND cr.client_session_id=e.session_id)
            ORDER BY j.classified_at DESC LIMIT 1;
        IF NOT FOUND THEN RETURN NULL; END IF;
        value := jsonb_build_object('assessment_key',key,'invocation_key',feedback_fact_key('invocation',e.id,NULL,e.user_id),
            'conversation_key',jsonb_build_object('source','astound.conversations','id',md5(jsonb_build_array(e.user_id,e.session_id)::text)),
            'occurred_at',a.classified_at,'outcome',jsonb_build_object('status','scored','score_millionths',a.completion::bigint*10000));
    END IF;
    RETURN jsonb_build_object('kind',q.kind,'value',value);
END $$;

DROP TRIGGER IF EXISTS attribute_skill_version ON plugin_usage_events;
CREATE OR REPLACE VIEW analysis_skill_version_events AS
SELECT e.id AS invocation_id,e.user_id,e.session_id,e.plugin_id,e.skill,
       e.tool_use_id,e.source,e.invoked_at,a.value->>'installation_id' AS installation_id,
       a.value->>'resource_id' AS resource_id,a.value->>'revision_id' AS revision_id,
       (a.value->>'generation')::bigint AS publication_generation,'production'::text AS traffic_class,
       CASE WHEN a.value IS NOT NULL THEN 'verified' ELSE 'revision_unknown' END AS attribution_status
FROM analysis_skill_events e
-- OFFSET 0 fences the LATERAL against pull-up. Without it Postgres inlines the
-- subquery and re-evaluates this six-join function once per referenced column,
-- so selecting the view cost five calls per row instead of one.
CROSS JOIN LATERAL (SELECT feedback_event_receipt(e.id) AS value OFFSET 0) a;
