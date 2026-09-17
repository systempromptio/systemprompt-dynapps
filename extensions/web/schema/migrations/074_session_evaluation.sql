-- Twin of schema/38_session_evaluation.sql: the LLM-as-judge engine.
--
-- The previous scorer wrote session_analyses from a Stop hook and never ran
-- in production (its AiService was layered as None), so the table only ever
-- held seed data. It is dropped, not migrated. The feedback plane's
-- `assessment` fact now reads the latest succeeded `quality` run.
DROP TABLE IF EXISTS session_analyses CASCADE;

-- Same body as schema/10_admin_dashboard.sql, minus the dropped join.
CREATE OR REPLACE VIEW v_all_activity AS
    (SELECT a.id, a.user_id,
        COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
        a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at,
        NOT ('anonymous' = ANY(u.roles)) AND u.email NOT LIKE '%@anonymous.local' AS is_real_user
    FROM user_activity a
    JOIN users u ON u.id = a.user_id)
    UNION ALL
    (SELECT s.id, s.user_id,
        COALESCE(u2.display_name, u2.full_name, u2.name, u2.email, s.user_id) AS display_name,
        'session' AS category,
        CASE WHEN s.ended_at IS NOT NULL THEN 'completed' ELSE 'started' END AS action,
        'session' AS entity_type, s.session_id AS entity_name,
        CASE
            WHEN s.ended_at IS NOT NULL AND s.ai_title IS NOT NULL AND s.ai_title != '' THEN s.ai_title
            WHEN s.ended_at IS NOT NULL THEN
                CONCAT('Completed AI session (', s.prompts, ' prompts, ', s.tool_uses, ' tool calls)')
            ELSE 'Started AI session'
        END AS description,
        COALESCE(s.ended_at, s.started_at, s.created_at) AS created_at,
        NOT ('anonymous' = ANY(u2.roles)) AND u2.email NOT LIKE '%@anonymous.local' AS is_real_user
    FROM plugin_session_summaries s
    JOIN users u2 ON u2.id = s.user_id)
    UNION ALL
    (SELECT r.id, r.user_id,
        COALESCE(u3.display_name, u3.full_name, u3.name, u3.email, r.user_id) AS display_name,
        'session_rated' AS category, 'rated' AS action,
        'session' AS entity_type, r.session_id AS entity_name,
        CONCAT('Rated session ', REPEAT('★', r.rating::int), REPEAT('☆', 5 - r.rating::int)) AS description,
        r.created_at,
        NOT ('anonymous' = ANY(u3.roles)) AND u3.email NOT LIKE '%@anonymous.local' AS is_real_user
    FROM session_ratings r
    JOIN users u3 ON u3.id = r.user_id);

-- Session evaluation: the LLM-as-judge engine. A closed session is queued
-- once per criterion; the session_evaluation job leases queue rows and writes
-- one run per attempt. Runs keep the full prompt and response so the
-- dashboard can show exactly what the judge saw. Twin: migrations/072.

CREATE TABLE IF NOT EXISTS evaluation_queue (
    session_id TEXT NOT NULL,
    criterion TEXT NOT NULL CHECK (criterion IN ('intent','quality','speed','cost')),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    host TEXT NOT NULL DEFAULT 'unknown',
    closed_at TIMESTAMPTZ NOT NULL,
    closed_reason TEXT NOT NULL CHECK (closed_reason IN ('session_end','quiet')),
    pending BOOLEAN NOT NULL DEFAULT true,
    attempts INT NOT NULL DEFAULT 0,
    next_attempt TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    lease_token TEXT,
    lease_until TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (session_id, criterion)
);
CREATE INDEX IF NOT EXISTS idx_evaluation_queue_pending ON evaluation_queue(next_attempt) WHERE pending;

CREATE TABLE IF NOT EXISTS evaluation_runs (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    session_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    host TEXT NOT NULL,
    criterion TEXT NOT NULL CHECK (criterion IN ('intent','quality','speed','cost')),
    attempt INT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('succeeded','failed')),
    score_source TEXT NOT NULL CHECK (score_source IN ('judge','metric')),
    score SMALLINT CHECK (score BETWEEN 0 AND 100),
    rationale TEXT,
    evidence JSONB NOT NULL DEFAULT '[]',
    metrics JSONB NOT NULL DEFAULT '{}',
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    ai_request_id TEXT,
    input_tokens BIGINT,
    output_tokens BIGINT,
    cost_microdollars BIGINT,
    latency_ms INT,
    system_prompt TEXT NOT NULL,
    user_prompt TEXT NOT NULL,
    response TEXT,
    error TEXT,
    config_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS idx_evaluation_runs_session ON evaluation_runs(session_id, criterion, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_evaluation_runs_created ON evaluation_runs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_evaluation_runs_user ON evaluation_runs(user_id, created_at DESC);

DROP TRIGGER IF EXISTS ingestion_owner ON evaluation_runs;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON evaluation_runs
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();
DROP TRIGGER IF EXISTS feedback_capture ON evaluation_runs;
CREATE TRIGGER feedback_capture AFTER INSERT OR UPDATE OR DELETE ON evaluation_runs FOR EACH ROW EXECUTE FUNCTION capture_feedback_source();

-- Same body as schema/31_feedback_projection.sql.
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

-- Same body as schema/32_feedback_retention.sql.
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
    DELETE FROM managed_invocation_attributions WHERE invocation_id IN(SELECT id FROM plugin_usage_events WHERE created_at<cutoff);
    DELETE FROM managed_consumer_invocation_evidence WHERE occurred_at<cutoff;
    DELETE FROM managed_consumer_session_bindings b WHERE bound_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM managed_consumer_invocation_evidence e WHERE e.consumer_id=b.consumer_id AND e.device_id=b.device_id AND e.host=b.host AND e.native_session_id=b.native_session_id)
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.user_id=b.consumer_id AND e.session_id=b.native_session_id AND e.created_at>=cutoff);
    DELETE FROM ingestion_event_receipts WHERE accepted_at<cutoff;
    DELETE FROM plugin_usage_events WHERE created_at<cutoff;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    DELETE FROM ai_requests WHERE created_at<cutoff;
    DELETE FROM evaluation_runs WHERE created_at<cutoff;
    DELETE FROM evaluation_queue WHERE created_at<cutoff;
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

DELETE FROM feedback_ingestion_backfill WHERE source='session_analyses';
INSERT INTO feedback_ingestion_backfill(source) VALUES('evaluation_runs') ON CONFLICT DO NOTHING;
