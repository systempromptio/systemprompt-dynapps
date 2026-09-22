-- Audit-event NOTIFY triggers
--
-- Fires a `pg_notify('audit_events', <json>)` whenever a row is inserted into
-- `governance_decisions`, `ai_requests`, or `plugin_usage_events`. The Rust
-- admin extension subscribes via PgListener and fans the payload out to SSE
-- clients on `/admin/api/sse/audit`. CREATE OR REPLACE TRIGGER (PG14+) keeps
-- the file declarative and idempotent across re-runs.
--
-- Robustness:
--   * Every trigger body runs inside a BEGIN/EXCEPTION WHEN OTHERS block so an
--     enrichment failure (bad cast, missing join target, future pg_notify
--     payload > 8000 bytes) downgrades to a RAISE WARNING instead of rolling
--     back the underlying audit INSERT. Auditing is the hot-path invariant;
--     SSE notification is best-effort.
--   * Payloads carry only stable row-local fields. Consumers that need user
--     display name / project fetch them via a repo helper keyed by `id`.
--     This removes the cross-extension JOIN against `user_profile_ext` from
--     the governance write path.
--   * Payload size is explicitly bounded to 7800 bytes (pg_notify limit is
--     8000); over-large payloads are replaced with a truncation marker so a
--     future field addition surfaces immediately in PG logs.

CREATE OR REPLACE FUNCTION audit_event_notify_governance()
RETURNS TRIGGER AS $$
DECLARE
    sev     TEXT;
    payload TEXT;
BEGIN
    BEGIN
        IF NEW.decision = 'deny' AND NEW.policy = 'secret_scan' THEN
            sev := 'breach';
        ELSIF NEW.decision = 'deny' THEN
            sev := 'deny';
        ELSE
            sev := 'info';
        END IF;

        payload := json_build_object(
            'table',      'governance_decisions',
            'id',         NEW.id,
            'session_id', NEW.session_id,
            'user_id',    NEW.user_id,
            'tool_name',  NEW.tool_name,
            'policy',     NEW.policy,
            'decision',   NEW.decision,
            'severity',   sev,
            'created_at', NEW.created_at
        )::text;

        IF length(payload) > 7800 THEN
            RAISE WARNING 'audit_event_notify_governance: payload truncated (% bytes)', length(payload);
            payload := json_build_object(
                'table',     'governance_decisions',
                'id',        NEW.id,
                'truncated', true
            )::text;
        END IF;

        PERFORM pg_notify('audit_events', payload);
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'audit_event_notify_governance failed: % (id=%, session=%)',
            SQLERRM, NEW.id, NEW.session_id;
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER audit_event_notify_governance_trg
    AFTER INSERT ON governance_decisions
    FOR EACH ROW
    EXECUTE FUNCTION audit_event_notify_governance();


-- One NOTIFY for one request row; the SSE consumer's contract is one event
-- per request. Scalar arguments so the function exists before the table does
-- on a fresh install (the installer applies functions first).
CREATE OR REPLACE FUNCTION audit_event_notify_ai_request(
    r_id TEXT, r_session_id TEXT, r_trace_id TEXT, r_context_id TEXT,
    r_user_id TEXT, r_model TEXT, r_status TEXT, r_created_at TIMESTAMPTZ
)
RETURNS VOID AS $$
DECLARE
    sev     TEXT;
    payload TEXT;
BEGIN
    BEGIN
        IF r_status NOT IN ('ok', 'success', 'completed', 'pending') THEN
            sev := 'error';
        ELSE
            sev := 'info';
        END IF;

        payload := json_build_object(
            'table',      'ai_requests',
            'id',         r_id,
            'session_id', r_session_id,
            'trace_id',   r_trace_id,
            'context_id', r_context_id,
            'user_id',    r_user_id,
            'model',      r_model,
            'status',     r_status,
            'severity',   sev,
            'created_at', r_created_at
        )::text;

        IF length(payload) > 7800 THEN
            RAISE WARNING 'audit_event_notify_ai_requests: payload truncated (% bytes)', length(payload);
            payload := json_build_object(
                'table',     'ai_requests',
                'id',        r_id,
                'truncated', true
            )::text;
        END IF;

        PERFORM pg_notify('audit_events', payload);
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'audit_event_notify_ai_requests failed: % (id=%, session=%)',
            SQLERRM, r_id, r_session_id;
    END;
END;
$$ LANGUAGE plpgsql;

-- Statement-level: one invocation per INSERT statement, looping the rows it
-- added. The TG_LEVEL branch keeps a database mid-upgrade working: the
-- installer swaps function bodies before migrations run, and the row-form
-- trigger still fires into this body until the declarative phase replaces it.
CREATE OR REPLACE FUNCTION audit_event_notify_ai_requests()
RETURNS TRIGGER AS $$
DECLARE
    r RECORD;
BEGIN
    IF TG_LEVEL = 'ROW' THEN
        PERFORM audit_event_notify_ai_request(NEW.id, NEW.session_id, NEW.trace_id, NEW.context_id,
            NEW.user_id, NEW.model, NEW.status, NEW.created_at);
        RETURN NEW;
    END IF;
    FOR r IN SELECT id, session_id, trace_id, context_id, user_id, model, status, created_at FROM new_rows LOOP
        PERFORM audit_event_notify_ai_request(r.id, r.session_id, r.trace_id, r.context_id,
            r.user_id, r.model, r.status, r.created_at);
    END LOOP;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS audit_event_notify_ai_requests_trg ON ai_requests;
CREATE OR REPLACE TRIGGER audit_event_notify_ai_requests_stmt
    AFTER INSERT ON ai_requests
    REFERENCING NEW TABLE AS new_rows
    FOR EACH STATEMENT
    EXECUTE FUNCTION audit_event_notify_ai_requests();


-- MCP tool execution events (plugin_usage_events). Fans tool-call activity onto
-- the same `audit_events` channel that AI requests and governance decisions use,
-- so the Live Overview > Services & tools pane can stream them without a
-- separate bus or polling fallback.
CREATE OR REPLACE FUNCTION audit_event_notify_plugin_usage()
RETURNS TRIGGER AS $$
DECLARE
    payload TEXT;
BEGIN
    BEGIN
        payload := json_build_object(
            'table',       'plugin_usage_events',
            'id',          NEW.id,
            'session_id',  NEW.session_id,
            'user_id',     NEW.user_id,
            'event_type',  NEW.event_type,
            'tool_name',   NEW.tool_name,
            'severity',    'info',
            'created_at',  NEW.created_at
        )::text;

        IF length(payload) > 7800 THEN
            RAISE WARNING 'audit_event_notify_plugin_usage: payload truncated (% bytes)', length(payload);
            payload := json_build_object(
                'table',     'plugin_usage_events',
                'id',        NEW.id,
                'truncated', true
            )::text;
        END IF;

        PERFORM pg_notify('audit_events', payload);
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'audit_event_notify_plugin_usage failed: % (id=%, session=%)',
            SQLERRM, NEW.id, NEW.session_id;
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER audit_event_notify_plugin_usage_trg
    AFTER INSERT ON plugin_usage_events
    FOR EACH ROW
    EXECUTE FUNCTION audit_event_notify_plugin_usage();
