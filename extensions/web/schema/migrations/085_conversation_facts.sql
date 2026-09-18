-- Twin of schema/45_conversation_facts.sql: the one judge row per
-- conversation and the deterministic conversation_facts rollup. Retires the
-- four-criteria session judge (evaluation_queue / evaluation_runs, migrations
-- 074 and 077), its attribution views and the AI evaluation report (081):
-- the single `completion` score on conversation_analyses replaces all of it,
-- so the feedback assessment fact now reads that row and the retention sweep
-- prunes the rollup instead of the runs.

DROP VIEW IF EXISTS evaluation_version_scores;
DROP VIEW IF EXISTS evaluation_session_skills;
DROP VIEW IF EXISTS evaluation_session_scorecard;
DROP VIEW IF EXISTS evaluation_session_scores;
DROP TABLE IF EXISTS evaluation_reports;
DROP TABLE IF EXISTS evaluation_runs;
DROP TABLE IF EXISTS evaluation_queue;
DELETE FROM feedback_ingestion_backfill WHERE source = 'evaluation_runs';

ALTER TABLE conversation_analyses ADD COLUMN IF NOT EXISTS title TEXT;
ALTER TABLE conversation_analyses ADD COLUMN IF NOT EXISTS completion SMALLINT
    CHECK (completion BETWEEN 0 AND 100);
ALTER TABLE conversation_analyses ADD COLUMN IF NOT EXISTS completion_rationale TEXT;
ALTER TABLE conversation_analyses ADD COLUMN IF NOT EXISTS input_tokens INTEGER;
ALTER TABLE conversation_analyses ADD COLUMN IF NOT EXISTS output_tokens INTEGER;
ALTER TABLE conversation_analyses ADD COLUMN IF NOT EXISTS cost_microdollars BIGINT;
ALTER TABLE conversation_analyses ADD COLUMN IF NOT EXISTS trigger TEXT NOT NULL DEFAULT 'automatic'
    CHECK (trigger IN ('automatic', 'manual'));
ALTER TABLE conversation_analyses ADD COLUMN IF NOT EXISTS requested_by TEXT;
CREATE INDEX IF NOT EXISTS idx_conversation_analyses_completion
    ON conversation_analyses(completion) WHERE completion IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_ai_requests_updated_at ON ai_requests(updated_at);

CREATE TABLE IF NOT EXISTS conversation_facts (
    context_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_id TEXT,
    client_session_id TEXT,
    group_id TEXT,
    project_id TEXT,
    client_kind TEXT NOT NULL DEFAULT 'unknown',
    client_attestation TEXT NOT NULL DEFAULT 'unknown',
    wire_protocol TEXT NOT NULL DEFAULT 'unknown',
    model TEXT,
    provider TEXT,
    models TEXT[] NOT NULL DEFAULT '{}',
    providers TEXT[] NOT NULL DEFAULT '{}',
    request_count BIGINT NOT NULL DEFAULT 0,
    turn_count BIGINT NOT NULL DEFAULT 0,
    side_call_count BIGINT NOT NULL DEFAULT 0,
    side_call_cost_microdollars BIGINT NOT NULL DEFAULT 0,
    error_count BIGINT NOT NULL DEFAULT 0,
    rejected_count BIGINT NOT NULL DEFAULT 0,
    streaming_count BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    reasoning_tokens BIGINT NOT NULL DEFAULT 0,
    cost_microdollars BIGINT NOT NULL DEFAULT 0,
    p50_latency_ms INTEGER,
    p95_latency_ms INTEGER,
    max_latency_ms INTEGER,
    tool_calls_intended BIGINT NOT NULL DEFAULT 0,
    tool_calls_executed BIGINT NOT NULL DEFAULT 0,
    tool_calls_failed BIGINT NOT NULL DEFAULT 0,
    artifact_count BIGINT NOT NULL DEFAULT 0,
    safety_findings BIGINT NOT NULL DEFAULT 0,
    safety_blocked BIGINT NOT NULL DEFAULT 0,
    gov_allow BIGINT NOT NULL DEFAULT 0,
    gov_warn BIGINT NOT NULL DEFAULT 0,
    gov_deny BIGINT NOT NULL DEFAULT 0,
    prompt_count BIGINT NOT NULL DEFAULT 0,
    hook_event_count BIGINT NOT NULL DEFAULT 0,
    hook_status TEXT,
    skill_invocations BIGINT NOT NULL DEFAULT 0,
    skills TEXT[] NOT NULL DEFAULT '{}',
    first_at TIMESTAMPTZ NOT NULL,
    last_at TIMESTAMPTZ NOT NULL,
    duration_seconds BIGINT NOT NULL DEFAULT 0,
    refreshed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_last_at ON conversation_facts(last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_user ON conversation_facts(user_id, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_group ON conversation_facts(group_id, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_project ON conversation_facts(project_id, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_model ON conversation_facts(model, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_client ON conversation_facts(client_kind, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_session ON conversation_facts(client_session_id)
    WHERE client_session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_conversation_facts_skills ON conversation_facts USING gin(skills);

-- Rebuilds the fact row for each context given (NULL = every context that
-- has a conversation request). Returns the number of rows written. The
-- request classification is `conversation_requests` (27_), so a turn here is
-- a turn everywhere; the group and project are the ones stamped on the latest
-- request, so a conversation stays filed where it was made.
CREATE OR REPLACE FUNCTION refresh_conversation_facts(context_ids TEXT[])
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE written BIGINT;
BEGIN
    WITH requests AS MATERIALIZED (
        SELECT r.*, ar.wire_protocol, ar.cache_read_tokens, ar.cache_creation_tokens,
               ar.reasoning_tokens, ar.is_streaming
        FROM conversation_requests r
        JOIN ai_requests ar ON ar.id = r.id
        WHERE context_ids IS NULL OR r.context_id = ANY(context_ids)
    ), agg AS (
        SELECT r.context_id,
            (ARRAY_AGG(r.user_id ORDER BY r.created_at DESC, r.id DESC))[1] AS user_id,
            (ARRAY_AGG(r.id ORDER BY r.created_at DESC, r.id DESC))[1] AS last_request_id,
            (ARRAY_AGG(r.session_id ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.session_id IS NOT NULL))[1] AS session_id,
            (ARRAY_AGG(r.client_session_id ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.client_session_id IS NOT NULL))[1] AS client_session_id,
            (ARRAY_AGG(r.client_kind ORDER BY r.created_at DESC, r.id DESC))[1] AS client_kind,
            (ARRAY_AGG(r.client_attestation ORDER BY r.created_at DESC, r.id DESC))[1] AS client_attestation,
            (ARRAY_AGG(r.wire_protocol ORDER BY r.created_at DESC, r.id DESC))[1] AS wire_protocol,
            (ARRAY_AGG(r.model ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.effective_kind = 'turn' AND r.model IS NOT NULL))[1] AS model,
            (ARRAY_AGG(r.provider ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.effective_kind = 'turn' AND r.provider IS NOT NULL))[1] AS provider,
            COUNT(*)::bigint AS request_count,
            COUNT(*) FILTER (WHERE r.effective_kind = 'turn')::bigint AS turn_count,
            COUNT(*) FILTER (WHERE r.effective_kind <> 'turn')::bigint AS side_call_count,
            COALESCE(SUM(r.cost_microdollars) FILTER (WHERE r.effective_kind <> 'turn'), 0)::bigint AS side_call_cost,
            COUNT(*) FILTER (WHERE r.status = 'failed')::bigint AS error_count,
            COUNT(*) FILTER (WHERE r.status = 'rejected')::bigint AS rejected_count,
            COUNT(*) FILTER (WHERE r.is_streaming)::bigint AS streaming_count,
            COALESCE(SUM(r.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(r.output_tokens), 0)::bigint AS output_tokens,
            COALESCE(SUM(r.cache_read_tokens), 0)::bigint AS cache_read_tokens,
            COALESCE(SUM(r.cache_creation_tokens), 0)::bigint AS cache_creation_tokens,
            COALESCE(SUM(r.reasoning_tokens), 0)::bigint AS reasoning_tokens,
            COALESCE(SUM(r.cost_microdollars), 0)::bigint AS cost_microdollars,
            percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.latency_ms IS NOT NULL AND r.effective_kind = 'turn') AS p50,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.latency_ms IS NOT NULL AND r.effective_kind = 'turn') AS p95,
            MAX(r.latency_ms) FILTER (WHERE r.effective_kind = 'turn') AS max_latency_ms,
            MIN(r.created_at) AS first_at,
            MAX(COALESCE(r.completed_at, r.created_at)) AS last_at
        FROM requests r
        GROUP BY r.context_id
    ), used AS (
        SELECT r.context_id,
            ARRAY(SELECT m.model FROM (
                SELECT model, MIN(created_at) AS first_use FROM requests q
                WHERE q.context_id = r.context_id AND q.model IS NOT NULL GROUP BY model) m
                ORDER BY m.first_use, m.model) AS models,
            ARRAY(SELECT DISTINCT q.provider FROM requests q
                WHERE q.context_id = r.context_id AND q.provider IS NOT NULL ORDER BY q.provider) AS providers
        FROM (SELECT DISTINCT context_id FROM requests) r
    ), intents AS (
        SELECT r.context_id, COUNT(*)::bigint AS tool_calls_intended
        FROM requests r JOIN ai_request_tool_calls t ON t.request_id = r.id
        GROUP BY r.context_id
    ), safety AS (
        SELECT r.context_id, COUNT(*)::bigint AS findings,
               COUNT(*) FILTER (WHERE f.blocked)::bigint AS blocked
        FROM requests r JOIN ai_safety_findings f ON f.ai_request_id = r.id
        GROUP BY r.context_id
    ), executions AS (
        SELECT a.context_id,
               COUNT(e.mcp_execution_id)::bigint AS executed,
               COUNT(e.mcp_execution_id) FILTER (WHERE e.status IN ('failed', 'timeout'))::bigint AS failed,
               COUNT(DISTINCT m.id)::bigint AS artifacts
        FROM agg a
        LEFT JOIN mcp_tool_executions e ON e.context_id = a.context_id
            OR (a.client_session_id IS NOT NULL AND e.trace_id = a.client_session_id)
        LEFT JOIN mcp_artifacts m ON m.mcp_execution_id = e.mcp_execution_id
        GROUP BY a.context_id
    ), governance AS (
        SELECT a.context_id,
               COUNT(*) FILTER (WHERE d.decision = 'allow')::bigint AS allow,
               COUNT(*) FILTER (WHERE d.decision = 'warn')::bigint AS warn,
               COUNT(*) FILTER (WHERE d.decision = 'deny')::bigint AS deny
        FROM agg a
        JOIN governance_decisions d ON d.context_id = a.context_id
            OR (a.client_session_id IS NOT NULL AND d.session_id = a.client_session_id)
        GROUP BY a.context_id
    ), hooks AS (
        SELECT a.context_id,
               COUNT(*)::bigint AS hook_events,
               COUNT(*) FILTER (WHERE e.event_type = 'UserPromptSubmit')::bigint AS prompts
        FROM agg a JOIN plugin_usage_events e ON e.session_id = a.client_session_id
        WHERE a.client_session_id IS NOT NULL
        GROUP BY a.context_id
    ), skills AS (
        SELECT a.context_id, COUNT(*)::bigint AS invocations,
               ARRAY(SELECT x.skill FROM (
                   SELECT skill, MIN(invoked_at) AS first_at FROM analysis_skill_events q
                   WHERE q.session_id = a.client_session_id AND q.skill IS NOT NULL GROUP BY skill) x
                   ORDER BY x.first_at, x.skill) AS skills
        FROM agg a JOIN analysis_skill_events s ON s.session_id = a.client_session_id AND s.skill IS NOT NULL
        WHERE a.client_session_id IS NOT NULL
        GROUP BY a.context_id, a.client_session_id
    ), upsert AS (
        INSERT INTO conversation_facts AS f (
            context_id, user_id, session_id, client_session_id, group_id, project_id,
            client_kind, client_attestation, wire_protocol, model, provider, models, providers,
            request_count, turn_count, side_call_count, side_call_cost_microdollars,
            error_count, rejected_count, streaming_count,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, reasoning_tokens,
            cost_microdollars, p50_latency_ms, p95_latency_ms, max_latency_ms,
            tool_calls_intended, tool_calls_executed, tool_calls_failed, artifact_count,
            safety_findings, safety_blocked, gov_allow, gov_warn, gov_deny,
            prompt_count, hook_event_count, hook_status, skill_invocations, skills,
            first_at, last_at, duration_seconds, refreshed_at)
        SELECT a.context_id, a.user_id, a.session_id, a.client_session_id, rs.group_id, rs.project_id,
               COALESCE(a.client_kind, 'unknown'), COALESCE(a.client_attestation, 'unknown'),
               COALESCE(a.wire_protocol, 'unknown'), a.model, a.provider,
               COALESCE(u.models, '{}'), COALESCE(u.providers, '{}'),
               a.request_count, a.turn_count, a.side_call_count, a.side_call_cost,
               a.error_count, a.rejected_count, a.streaming_count,
               a.input_tokens, a.output_tokens, a.cache_read_tokens, a.cache_creation_tokens, a.reasoning_tokens,
               a.cost_microdollars, a.p50::int, a.p95::int, a.max_latency_ms,
               COALESCE(i.tool_calls_intended, 0), COALESCE(x.executed, 0), COALESCE(x.failed, 0), COALESCE(x.artifacts, 0),
               COALESCE(s.findings, 0), COALESCE(s.blocked, 0),
               COALESCE(g.allow, 0), COALESCE(g.warn, 0), COALESCE(g.deny, 0),
               COALESCE(h.prompts, 0), COALESCE(h.hook_events, 0), ps.status,
               COALESCE(k.invocations, 0), COALESCE(k.skills, '{}'),
               a.first_at, GREATEST(a.last_at, c.updated_at),
               GREATEST(EXTRACT(EPOCH FROM GREATEST(a.last_at, c.updated_at) - a.first_at)::bigint, 0),
               clock_timestamp()
        FROM agg a
        LEFT JOIN used u ON u.context_id = a.context_id
        LEFT JOIN intents i ON i.context_id = a.context_id
        LEFT JOIN safety s ON s.context_id = a.context_id
        LEFT JOIN executions x ON x.context_id = a.context_id
        LEFT JOIN governance g ON g.context_id = a.context_id
        LEFT JOIN hooks h ON h.context_id = a.context_id
        LEFT JOIN skills k ON k.context_id = a.context_id
        LEFT JOIN ai_request_scopes rs ON rs.request_id = a.last_request_id
        LEFT JOIN user_contexts c ON c.context_id = a.context_id
        LEFT JOIN plugin_session_summaries ps ON ps.session_id = a.client_session_id
        ON CONFLICT (context_id) DO UPDATE SET
            user_id = EXCLUDED.user_id, session_id = EXCLUDED.session_id,
            client_session_id = EXCLUDED.client_session_id,
            group_id = EXCLUDED.group_id, project_id = EXCLUDED.project_id,
            client_kind = EXCLUDED.client_kind, client_attestation = EXCLUDED.client_attestation,
            wire_protocol = EXCLUDED.wire_protocol, model = EXCLUDED.model, provider = EXCLUDED.provider,
            models = EXCLUDED.models, providers = EXCLUDED.providers,
            request_count = EXCLUDED.request_count, turn_count = EXCLUDED.turn_count,
            side_call_count = EXCLUDED.side_call_count,
            side_call_cost_microdollars = EXCLUDED.side_call_cost_microdollars,
            error_count = EXCLUDED.error_count, rejected_count = EXCLUDED.rejected_count,
            streaming_count = EXCLUDED.streaming_count,
            input_tokens = EXCLUDED.input_tokens, output_tokens = EXCLUDED.output_tokens,
            cache_read_tokens = EXCLUDED.cache_read_tokens,
            cache_creation_tokens = EXCLUDED.cache_creation_tokens,
            reasoning_tokens = EXCLUDED.reasoning_tokens, cost_microdollars = EXCLUDED.cost_microdollars,
            p50_latency_ms = EXCLUDED.p50_latency_ms, p95_latency_ms = EXCLUDED.p95_latency_ms,
            max_latency_ms = EXCLUDED.max_latency_ms,
            tool_calls_intended = EXCLUDED.tool_calls_intended,
            tool_calls_executed = EXCLUDED.tool_calls_executed,
            tool_calls_failed = EXCLUDED.tool_calls_failed, artifact_count = EXCLUDED.artifact_count,
            safety_findings = EXCLUDED.safety_findings, safety_blocked = EXCLUDED.safety_blocked,
            gov_allow = EXCLUDED.gov_allow, gov_warn = EXCLUDED.gov_warn, gov_deny = EXCLUDED.gov_deny,
            prompt_count = EXCLUDED.prompt_count, hook_event_count = EXCLUDED.hook_event_count,
            hook_status = EXCLUDED.hook_status, skill_invocations = EXCLUDED.skill_invocations,
            skills = EXCLUDED.skills, first_at = EXCLUDED.first_at, last_at = EXCLUDED.last_at,
            duration_seconds = EXCLUDED.duration_seconds, refreshed_at = EXCLUDED.refreshed_at
        RETURNING 1)
    SELECT COUNT(*) INTO written FROM upsert;
    IF context_ids IS NOT NULL THEN
        DELETE FROM conversation_facts f WHERE f.context_id = ANY(context_ids)
            AND NOT EXISTS (SELECT 1 FROM conversation_requests r WHERE r.context_id = f.context_id);
    END IF;
    RETURN COALESCE(written, 0);
END $$;

-- Every context touched since `since` on any plane that feeds the row: a
-- request settled, a hook event, a governance decision, a tool execution or
-- an artifact. The rollup job calls this each tick with an overlapping window;
-- the upsert is idempotent, so overlap is free.
CREATE OR REPLACE FUNCTION refresh_conversation_facts_since(since TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE sql AS $$
SELECT refresh_conversation_facts(ARRAY(
    SELECT DISTINCT context_id FROM (
        SELECT r.context_id FROM ai_requests r WHERE r.updated_at >= since OR r.created_at >= since
        UNION ALL
        SELECT r.context_id FROM ai_requests r
        WHERE r.client_session_id IN (
            SELECT e.session_id FROM plugin_usage_events e WHERE e.created_at >= since
            UNION ALL
            SELECT d.session_id FROM governance_decisions d WHERE d.created_at >= since
            UNION ALL
            SELECT x.trace_id FROM mcp_tool_executions x WHERE x.created_at >= since AND x.trace_id IS NOT NULL)
        UNION ALL
        SELECT x.context_id FROM mcp_tool_executions x WHERE x.created_at >= since AND x.context_id IS NOT NULL
        UNION ALL
        SELECT m.context_id FROM mcp_artifacts m WHERE m.created_at >= since AND m.context_id IS NOT NULL
        UNION ALL
        SELECT d.context_id FROM governance_decisions d WHERE d.created_at >= since
    ) touched
    WHERE context_id IS NOT NULL AND context_id <> '00000000-0000-0000-0000-4c4547414359'))
$$;

-- Which skills a conversation invoked and which marketplace version was
-- being served when it first did — the version is fixed at first invocation,
-- never at judge time, so a deploy between the two never credits the new hash
-- with behaviour the old one produced.
CREATE OR REPLACE VIEW conversation_skill_versions AS
SELECT f.context_id, f.user_id, f.client_session_id, e.plugin_id, e.skill, o.marketplace_id,
       marketplace_version_at(o.marketplace_id, MIN(e.invoked_at)) AS marketplace_hash,
       COUNT(*)::bigint AS invocations, MIN(e.invoked_at) AS first_invoked_at
FROM conversation_facts f
JOIN analysis_skill_events e ON e.session_id = f.client_session_id AND e.skill IS NOT NULL
LEFT JOIN service_owned_ids o ON o.kind = 'plugin' AND o.id = e.plugin_id
WHERE f.client_session_id IS NOT NULL
GROUP BY f.context_id, f.user_id, f.client_session_id, e.plugin_id, e.skill, o.marketplace_id;

-- A judge verdict is a feedback assessment fact for every skill invocation in
-- the conversation's harness session. The fact body is built by
-- feedback_normalized_fact (31_), which reads this table by context; here the
-- conversation's session is re-enqueued so those facts are regenerated.
CREATE OR REPLACE FUNCTION capture_feedback_judgement()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data JSONB; row RECORD;
BEGIN
    IF TG_OP='DELETE' AND current_setting('astound.feedback_retention',true)='expiry' THEN RETURN OLD; END IF;
    data := CASE WHEN TG_OP='DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
    FOR row IN SELECT DISTINCT r.user_id, r.client_session_id FROM ai_requests r
        WHERE r.context_id = data->>'context_id' AND r.client_session_id IS NOT NULL LOOP
        PERFORM enqueue_feedback_session(row.user_id, row.client_session_id);
    END LOOP;
    RETURN COALESCE(NEW,OLD);
END $$;
DROP TRIGGER IF EXISTS feedback_capture ON conversation_analyses;
CREATE TRIGGER feedback_capture AFTER INSERT OR UPDATE OR DELETE ON conversation_analyses
FOR EACH ROW EXECUTE FUNCTION capture_feedback_judgement();

-- feedback_normalized_fact_base (39_) with the assessment branch reading the judge row.
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

-- expire_feedback_raw (32_) without the retired tables.
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
    DELETE FROM conversation_facts WHERE last_at<cutoff;
    DELETE FROM conversation_analyses WHERE created_at<cutoff;
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

-- conversation_analysis_rows (37_), re-created so its definition matches the
-- declarative file byte for byte; the judge's columns live on the pages'
-- own queries, never on this ad-hoc view.
DROP VIEW IF EXISTS conversation_analysis_rows;
CREATE OR REPLACE VIEW conversation_analysis_rows AS
SELECT r.*, a.status AS analysis_status, a.category, a.summary, a.tags, a.outcome, a.confidence,
       a.skills_used, a.classified_at, a.model AS judge_model,
       COALESCE((SELECT array_agg(s.skill ORDER BY s.first_at) FROM conversation_skill_uses s
                 WHERE s.client_session_id = r.client_session_id), '{}'::text[]) AS skills_hooked
FROM conversation_metrics_for(NULL) r
LEFT JOIN conversation_analyses a ON a.context_id = r.context_id;

SELECT refresh_conversation_facts(NULL);
