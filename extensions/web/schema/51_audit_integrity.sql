-- Audit integrity (NFR-5.6 / REQ-026 / DEC-007): append-only by mechanism,
-- a retention floor under every audit delete, and a hash chain over the
-- gateway request log and its safety findings.
--
-- The application role owns these tables (it migrates them), and a table
-- owner can always re-grant itself or disable a trigger, so REVOKE alone is
-- not a mechanism here. The guard is a set of triggers every writer passes
-- through — core, this extension, the data console, a psql session — and a
-- per-installation mode decides what a breach does:
--   off      nothing is checked
--   warn     the statement runs and a row lands in audit_guard_violations
--   enforce  the statement is refused (restrict_violation)
-- The default is warn, matching the rest of this installation's measurement
-- window; set_audit_guard('enforce', TRUE) flips it once the violations
-- table has stayed empty under real traffic. A deliberate backfill or a legal
-- erasure inside the floor runs with
--   SET LOCAL systemprompt.audit_maintenance = 'on'
-- in its own transaction, which the guard lets through.
--
-- Matrix (UPDATE / DELETE / TRUNCATE):
--   ai_requests              columns, see audit_guard_ai_requests_update / floor 90 d / refused
--   ai_safety_findings       refused / floor 90 d / refused
--   ai_request_attributions  refused / only by cascade from ai_requests / refused
--   governance_decisions     refused by core's own trigger / floor 30 d / refused
--   mcp_tool_executions      allowed (in-flight results) / floor 30 d / refused
--   plugin_usage_events      allowed (execution link, merges) / floor 90 d / refused
--   secret_audit_log         refused / floor 365 d / refused
--   platform_data_clear_audit refused / never / refused
--   audit_chain_links        refused / floor 90 d / refused
--   audit_guard_violations   refused / floor 365 d / refused
-- A delete that arrives by foreign-key cascade (pg_trigger_depth() > 1) is
-- governed by its parent: a user purge or an ai_requests expiry takes its
-- children with it. The floors sit at or under every retention window that
-- deletes these rows (expire_raw_evidence refuses a cutoff under 90 days; the
-- profile defaults are 180 d governance, 365 d tool executions).
--
-- ai_requests rows are settled after insert, so UPDATE is constrained rather
-- than refused. Always immutable: id, request_id, created_at, actor_kind,
-- actor_id, api_key_id. Once a row is 'completed' or 'rejected', its routing
-- (provider, served_provider, model, requested_model) and usage (the token
-- counts, cost_microdollars, finish_reason) are frozen, and its status may only
-- move completed -> failed (an accounting failure). Everything else stays
-- writable: user_id (account merge), session/task/context ids (FK SET NULL,
-- attribution), message_count, error and accounting fields, timings.
--
-- The chain: the `audit_chain_seal` job (every 10 s) appends one
-- audit_chain_links row per new ai_requests and ai_safety_findings row,
-- seq + 1 under a row lock on audit_chain_heads, with
-- row_hash = sha256(prev_hash || canonical fields). ai_requests hashes its
-- immutable columns; a safety finding hashes every column. audit_chain_seal()
-- is the only writer (a transaction-scoped advisory lock), so the head lock is
-- never contended and no gateway insert waits on it. It walks each table from
-- its audit_chain_cursors position minus a ten-minute lookback, linking every
-- row that has no link yet, so a row whose transaction committed late is still
-- sealed. The seal is therefore eventually consistent (within one seal
-- interval); the UPDATE/DELETE/TRUNCATE guards above stay synchronous.
-- verify_audit_chain(table, from, to) reports gaps, broken links, altered and
-- deleted rows, and rows the sealer has passed with no link;
-- audit_chain_unsealed(table) counts the rows still waiting.
-- record_audit_chain_verification() is the nightly `audit_chain_verify` job
-- and keeps each run's anchor, head and unsealed count.
-- set_audit_guard(mode, FALSE) turns the chain off; the sealer then no-ops.

CREATE TABLE IF NOT EXISTS audit_guard_settings (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    mode TEXT NOT NULL DEFAULT 'warn' CHECK (mode IN ('off', 'warn', 'enforce')),
    chain_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS audit_guard_violations (
    id BIGSERIAL PRIMARY KEY,
    table_name TEXT NOT NULL,
    operation TEXT NOT NULL,
    row_id TEXT,
    rule TEXT NOT NULL,
    db_user TEXT NOT NULL DEFAULT current_user,
    application_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS idx_audit_guard_violations_created ON audit_guard_violations(created_at DESC);

CREATE TABLE IF NOT EXISTS audit_chain_heads (
    table_name TEXT PRIMARY KEY,
    seq BIGINT NOT NULL,
    head_hash TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS audit_chain_links (
    table_name TEXT NOT NULL,
    seq BIGINT NOT NULL,
    row_id TEXT NOT NULL,
    row_created_at TIMESTAMPTZ NOT NULL,
    prev_hash TEXT NOT NULL,
    row_hash TEXT NOT NULL,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (table_name, seq)
);
CREATE INDEX IF NOT EXISTS idx_audit_chain_links_row ON audit_chain_links(table_name, row_id);
CREATE INDEX IF NOT EXISTS idx_audit_chain_links_created ON audit_chain_links(table_name, row_created_at);

CREATE TABLE IF NOT EXISTS audit_chain_verifications (
    id BIGSERIAL PRIMARY KEY,
    run_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    table_name TEXT NOT NULL,
    window_from TIMESTAMPTZ NOT NULL,
    window_to TIMESTAMPTZ NOT NULL,
    first_seq BIGINT,
    last_seq BIGINT,
    anchor_hash TEXT,
    head_hash TEXT,
    links_checked BIGINT NOT NULL,
    problems BIGINT NOT NULL,
    first_problems JSONB NOT NULL DEFAULT '[]'::jsonb,
    unsealed_rows BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_audit_chain_verifications_run ON audit_chain_verifications(table_name, run_at DESC);

CREATE TABLE IF NOT EXISTS audit_chain_cursors (
    table_name TEXT PRIMARY KEY,
    last_created_at TIMESTAMPTZ NOT NULL,
    last_id TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION set_audit_guard(p_mode TEXT, p_chain BOOLEAN)
RETURNS VOID LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO audit_guard_settings (singleton, mode, chain_enabled, updated_at)
    VALUES (TRUE, p_mode, p_chain, CURRENT_TIMESTAMP)
    ON CONFLICT (singleton) DO UPDATE
    SET mode = EXCLUDED.mode, chain_enabled = EXCLUDED.chain_enabled, updated_at = EXCLUDED.updated_at;
END $$;

CREATE OR REPLACE FUNCTION audit_guard_violation(p_table TEXT, p_op TEXT, p_row_id TEXT, p_rule TEXT)
RETURNS VOID LANGUAGE plpgsql AS $$
DECLARE
    guard_mode TEXT := COALESCE((SELECT mode FROM audit_guard_settings WHERE singleton), 'warn');
BEGIN
    IF guard_mode = 'off' OR current_setting('systemprompt.audit_maintenance', TRUE) = 'on' THEN
        RETURN;
    END IF;
    IF guard_mode = 'enforce' THEN
        RAISE EXCEPTION 'audit guard: % on % refused: %', p_op, p_table, p_rule
            USING ERRCODE = 'restrict_violation',
                  HINT = 'A deliberate backfill or erasure sets systemprompt.audit_maintenance = on in its own transaction.';
    END IF;
    INSERT INTO audit_guard_violations (table_name, operation, row_id, rule, application_name)
    VALUES (p_table, p_op, p_row_id, p_rule, current_setting('application_name', TRUE));
END $$;

-- TG_ARGV: floor in days or 'never', the timestamp column, the id column.
CREATE OR REPLACE FUNCTION audit_guard_delete()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    old_row JSONB;
BEGIN
    IF pg_trigger_depth() > 1 THEN
        RETURN OLD;
    END IF;
    old_row := to_jsonb(OLD);
    IF TG_ARGV[0] <> 'never' THEN
        IF (old_row ->> TG_ARGV[1])::timestamptz < now() - make_interval(days => TG_ARGV[0]::int) THEN
            RETURN OLD;
        END IF;
    END IF;
    PERFORM audit_guard_violation(TG_TABLE_NAME, 'DELETE', old_row ->> TG_ARGV[2],
        CASE WHEN TG_ARGV[0] = 'never' THEN 'never deleted directly'
             ELSE format('row is inside the %s-day retention floor', TG_ARGV[0]) END);
    RETURN OLD;
END $$;

-- TG_ARGV: the id column.
CREATE OR REPLACE FUNCTION audit_guard_append_only()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    PERFORM audit_guard_violation(TG_TABLE_NAME, 'UPDATE', to_jsonb(OLD) ->> TG_ARGV[0], 'append-only');
    RETURN NEW;
END $$;

CREATE OR REPLACE FUNCTION audit_guard_truncate()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    PERFORM audit_guard_violation(TG_TABLE_NAME, 'TRUNCATE', NULL, 'audit tables are never truncated');
    RETURN NULL;
END $$;

CREATE OR REPLACE FUNCTION audit_guard_ai_requests_update()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    broken TEXT;
BEGIN
    IF (NEW.id, NEW.request_id, NEW.created_at, NEW.actor_kind, NEW.actor_id, NEW.api_key_id)
        IS DISTINCT FROM
       (OLD.id, OLD.request_id, OLD.created_at, OLD.actor_kind, OLD.actor_id, OLD.api_key_id) THEN
        broken := 'identity column changed';
    ELSIF OLD.status IN ('completed', 'rejected')
        AND (NEW.provider, NEW.served_provider, NEW.model, NEW.requested_model,
             NEW.input_tokens, NEW.output_tokens, NEW.cache_read_tokens, NEW.cache_creation_tokens,
             NEW.reasoning_tokens, NEW.tokens_used, NEW.cost_microdollars, NEW.finish_reason)
        IS DISTINCT FROM
            (OLD.provider, OLD.served_provider, OLD.model, OLD.requested_model,
             OLD.input_tokens, OLD.output_tokens, OLD.cache_read_tokens, OLD.cache_creation_tokens,
             OLD.reasoning_tokens, OLD.tokens_used, OLD.cost_microdollars, OLD.finish_reason) THEN
        broken := 'settled routing or usage changed';
    ELSIF OLD.status = 'completed' AND NEW.status NOT IN ('completed', 'failed') THEN
        broken := 'completed request reopened';
    ELSIF OLD.status = 'rejected' AND NEW.status <> 'rejected' THEN
        broken := 'rejected request reopened';
    END IF;
    IF broken IS NOT NULL THEN
        PERFORM audit_guard_violation('ai_requests', 'UPDATE', OLD.id, broken);
    END IF;
    RETURN NEW;
END $$;

CREATE OR REPLACE FUNCTION audit_chain_hash(p_prev TEXT, p_payload TEXT)
RETURNS TEXT LANGUAGE sql IMMUTABLE AS $$
    SELECT encode(sha256(convert_to(p_prev || chr(31) || p_payload, 'UTF8')), 'hex')
$$;

CREATE OR REPLACE FUNCTION audit_chain_payload_ai_requests(r ai_requests)
RETURNS TEXT LANGUAGE sql IMMUTABLE AS $$
    SELECT concat_ws(chr(31), r.id, r.request_id, r.actor_kind, r.actor_id,
        COALESCE(r.api_key_id, chr(30)),
        to_char(r.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US'))
$$;

CREATE OR REPLACE FUNCTION audit_chain_payload_ai_safety_findings(r ai_safety_findings)
RETURNS TEXT LANGUAGE sql IMMUTABLE AS $$
    SELECT concat_ws(chr(31), r.id, r.ai_request_id, r.phase, r.severity, r.category, r.scanner,
        COALESCE(r.excerpt, chr(30)), r.blocked::text,
        to_char(r.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US'))
$$;

CREATE OR REPLACE FUNCTION audit_chain_append(p_table TEXT, p_row_id TEXT, p_created TIMESTAMPTZ, p_payload TEXT)
RETURNS VOID LANGUAGE plpgsql AS $$
DECLARE
    head_seq BIGINT;
    head TEXT;
    next_hash TEXT;
BEGIN
    IF NOT COALESCE((SELECT chain_enabled FROM audit_guard_settings WHERE singleton), TRUE) THEN
        RETURN;
    END IF;
    INSERT INTO audit_chain_heads (table_name, seq, head_hash)
    VALUES (p_table, 0, repeat('0', 64)) ON CONFLICT (table_name) DO NOTHING;
    SELECT h.seq, h.head_hash INTO STRICT head_seq, head
    FROM audit_chain_heads h WHERE h.table_name = p_table FOR UPDATE;
    next_hash := audit_chain_hash(head, p_payload);
    INSERT INTO audit_chain_links (table_name, seq, row_id, row_created_at, prev_hash, row_hash)
    VALUES (p_table, head_seq + 1, p_row_id, p_created, head, next_hash);
    UPDATE audit_chain_heads
    SET seq = head_seq + 1, head_hash = next_hash, updated_at = clock_timestamp()
    WHERE table_name = p_table;
END $$;

-- Rows created before this bound have been passed by the sealer for good:
-- its cursor minus the lookback it re-scans for late commits. NULL while a
-- table has never been sealed.
CREATE OR REPLACE FUNCTION audit_chain_sealed_through(p_table TEXT)
RETURNS TIMESTAMPTZ LANGUAGE sql STABLE AS $$
    SELECT c.last_created_at - interval '10 minutes' FROM audit_chain_cursors c WHERE c.table_name = p_table
$$;

CREATE OR REPLACE FUNCTION audit_chain_seal_table(p_table TEXT, p_limit INTEGER)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE
    cur_at TIMESTAMPTZ;
    cur_id TEXT;
    from_at TIMESTAMPTZ;
    req ai_requests;
    fin ai_safety_findings;
    sealed BIGINT := 0;
BEGIN
    SELECT c.last_created_at, c.last_id INTO cur_at, cur_id
    FROM audit_chain_cursors c WHERE c.table_name = p_table;
    from_at := COALESCE(cur_at - interval '10 minutes', '-infinity'::timestamptz);
    IF p_table = 'ai_requests' THEN
        FOR req IN SELECT a.* FROM ai_requests a
                   WHERE a.created_at >= from_at
                     AND NOT EXISTS (SELECT 1 FROM audit_chain_links k
                                     WHERE k.table_name = 'ai_requests' AND k.row_id = a.id)
                   ORDER BY a.created_at, a.id LIMIT p_limit LOOP
            PERFORM audit_chain_append('ai_requests', req.id, req.created_at,
                audit_chain_payload_ai_requests(req));
            IF cur_at IS NULL OR (req.created_at, req.id::TEXT) > (cur_at, cur_id) THEN
                cur_at := req.created_at; cur_id := req.id;
            END IF;
            sealed := sealed + 1;
        END LOOP;
    ELSIF p_table = 'ai_safety_findings' THEN
        FOR fin IN SELECT f.* FROM ai_safety_findings f
                   WHERE f.created_at >= from_at
                     AND NOT EXISTS (SELECT 1 FROM audit_chain_links k
                                     WHERE k.table_name = 'ai_safety_findings' AND k.row_id = f.id)
                   ORDER BY f.created_at, f.id LIMIT p_limit LOOP
            PERFORM audit_chain_append('ai_safety_findings', fin.id, fin.created_at,
                audit_chain_payload_ai_safety_findings(fin));
            IF cur_at IS NULL OR (fin.created_at, fin.id::TEXT) > (cur_at, cur_id) THEN
                cur_at := fin.created_at; cur_id := fin.id;
            END IF;
            sealed := sealed + 1;
        END LOOP;
    ELSE
        RAISE EXCEPTION 'no audit chain for %', p_table USING ERRCODE = '22023';
    END IF;
    IF cur_at IS NOT NULL THEN
        INSERT INTO audit_chain_cursors (table_name, last_created_at, last_id)
        VALUES (p_table, cur_at, cur_id)
        ON CONFLICT (table_name) DO UPDATE
        SET last_created_at = EXCLUDED.last_created_at, last_id = EXCLUDED.last_id,
            updated_at = clock_timestamp();
    END IF;
    RETURN sealed;
END $$;

-- The `audit_chain_seal` job. One writer at a time by a transaction-scoped
-- advisory lock (a pooled connection never keeps it); a second caller returns
-- -1 at once. Returns the rows sealed, 0 when the chain is off.
CREATE OR REPLACE FUNCTION audit_chain_seal(p_limit INTEGER)
RETURNS BIGINT LANGUAGE plpgsql AS $$
BEGIN
    IF NOT COALESCE((SELECT chain_enabled FROM audit_guard_settings WHERE singleton), TRUE) THEN
        RETURN 0;
    END IF;
    -- Why: core's scheduler holds pg_advisory_lock(hashtext(<job name>)) on
    -- another connection while a job runs, so a key equal to the job name
    -- 'audit_chain_seal' is always taken by the job's own scheduler slot.
    IF NOT pg_try_advisory_xact_lock(hashtext('audit_chain_seal:chain')) THEN
        RETURN -1;
    END IF;
    RETURN audit_chain_seal_table('ai_requests', p_limit)
        + audit_chain_seal_table('ai_safety_findings', p_limit);
END $$;

CREATE OR REPLACE FUNCTION audit_chain_unsealed(p_table TEXT)
RETURNS BIGINT LANGUAGE plpgsql STABLE AS $$
DECLARE
    from_at TIMESTAMPTZ := COALESCE(audit_chain_sealed_through(p_table), '-infinity'::timestamptz);
    waiting BIGINT;
BEGIN
    IF p_table = 'ai_requests' THEN
        SELECT count(*) INTO waiting FROM ai_requests a
        WHERE a.created_at >= from_at
          AND NOT EXISTS (SELECT 1 FROM audit_chain_links k
                          WHERE k.table_name = 'ai_requests' AND k.row_id = a.id);
    ELSE
        SELECT count(*) INTO waiting FROM ai_safety_findings f
        WHERE f.created_at >= from_at
          AND NOT EXISTS (SELECT 1 FROM audit_chain_links k
                          WHERE k.table_name = 'ai_safety_findings' AND k.row_id = f.id);
    END IF;
    RETURN waiting;
END $$;

CREATE OR REPLACE FUNCTION verify_audit_chain(p_table TEXT, p_from TIMESTAMPTZ, p_to TIMESTAMPTZ)
RETURNS TABLE (link_seq BIGINT, link_row_id TEXT, problem TEXT)
LANGUAGE plpgsql STABLE AS $$
DECLARE
    lo BIGINT;
    hi BIGINT;
    first_linked TIMESTAMPTZ;
    sealed_to TIMESTAMPTZ;
    link RECORD;
    prev_seq BIGINT;
    v_prev_hash TEXT;
    payload TEXT;
BEGIN
    IF p_table NOT IN ('ai_requests', 'ai_safety_findings') THEN
        RAISE EXCEPTION 'no audit chain for %', p_table USING ERRCODE = '22023';
    END IF;
    SELECT min(k.seq), max(k.seq) INTO lo, hi FROM audit_chain_links k
    WHERE k.table_name = p_table AND k.row_created_at >= p_from AND k.row_created_at < p_to;
    SELECT min(k.linked_at) INTO first_linked FROM audit_chain_links k WHERE k.table_name = p_table;
    IF lo IS NOT NULL THEN
        FOR link IN SELECT k.* FROM audit_chain_links k
                    WHERE k.table_name = p_table AND k.seq BETWEEN lo AND hi ORDER BY k.seq LOOP
            IF prev_seq IS NOT NULL AND link.seq <> prev_seq + 1 THEN
                link_seq := prev_seq + 1; link_row_id := NULL;
                problem := format('links %s..%s missing', prev_seq + 1, link.seq - 1);
                RETURN NEXT;
            ELSIF prev_seq IS NOT NULL AND link.prev_hash <> v_prev_hash THEN
                link_seq := link.seq; link_row_id := link.row_id;
                problem := 'prev_hash does not match the previous link';
                RETURN NEXT;
            END IF;
            payload := NULL;
            IF p_table = 'ai_requests' THEN
                SELECT audit_chain_payload_ai_requests(a) INTO payload FROM ai_requests a WHERE a.id = link.row_id;
            ELSE
                SELECT audit_chain_payload_ai_safety_findings(f) INTO payload
                FROM ai_safety_findings f WHERE f.id = link.row_id;
            END IF;
            IF payload IS NULL THEN
                IF link.row_created_at >= now() - interval '90 days' THEN
                    link_seq := link.seq; link_row_id := link.row_id;
                    problem := 'row deleted inside the retention floor';
                    RETURN NEXT;
                END IF;
            ELSIF audit_chain_hash(link.prev_hash, payload) <> link.row_hash THEN
                link_seq := link.seq; link_row_id := link.row_id;
                problem := 'row altered: hash mismatch';
                RETURN NEXT;
            END IF;
            prev_seq := link.seq;
            v_prev_hash := link.row_hash;
        END LOOP;
    END IF;
    sealed_to := LEAST(p_to, audit_chain_sealed_through(p_table));
    IF first_linked IS NULL OR sealed_to IS NULL THEN
        RETURN;
    END IF;
    IF p_table = 'ai_requests' THEN
        RETURN QUERY SELECT NULL::BIGINT, a.id::TEXT, 'row has no chain link'::TEXT
        FROM ai_requests a
        WHERE a.created_at >= GREATEST(p_from, first_linked) AND a.created_at < sealed_to
          AND NOT EXISTS (SELECT 1 FROM audit_chain_links k
                          WHERE k.table_name = 'ai_requests' AND k.row_id = a.id);
    ELSE
        RETURN QUERY SELECT NULL::BIGINT, f.id::TEXT, 'row has no chain link'::TEXT
        FROM ai_safety_findings f
        WHERE f.created_at >= GREATEST(p_from, first_linked) AND f.created_at < sealed_to
          AND NOT EXISTS (SELECT 1 FROM audit_chain_links k
                          WHERE k.table_name = 'ai_safety_findings' AND k.row_id = f.id);
    END IF;
END $$;

-- The nightly check: verifies the last p_window of both chains, records one
-- row per table (with the anchor it started from, the head it reached, so
-- a later rewrite of the chain itself disagrees with what was recorded, and
-- the rows still waiting for the sealer), and
-- drops links past the longest floor. Returns the number of problems found.
CREATE OR REPLACE FUNCTION record_audit_chain_verification(p_window INTERVAL)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE
    chained TEXT;
    v_to TIMESTAMPTZ := clock_timestamp();
    v_from TIMESTAMPTZ := now() - p_window;
    found_problems BIGINT;
    total BIGINT := 0;
BEGIN
    FOREACH chained IN ARRAY ARRAY['ai_requests', 'ai_safety_findings'] LOOP
        WITH problems AS (
            SELECT * FROM verify_audit_chain(chained, v_from, v_to)
        ), span AS (
            SELECT min(k.seq) AS first_seq, max(k.seq) AS last_seq, count(*) AS checked
            FROM audit_chain_links k
            WHERE k.table_name = chained AND k.row_created_at >= v_from AND k.row_created_at < v_to
        )
        INSERT INTO audit_chain_verifications (table_name, window_from, window_to, first_seq, last_seq,
            anchor_hash, head_hash, links_checked, problems, first_problems, unsealed_rows)
        SELECT chained, v_from, v_to, span.first_seq, span.last_seq,
            (SELECT k.prev_hash FROM audit_chain_links k WHERE k.table_name = chained AND k.seq = span.first_seq),
            (SELECT k.row_hash FROM audit_chain_links k WHERE k.table_name = chained AND k.seq = span.last_seq),
            span.checked,
            (SELECT count(*) FROM problems),
            COALESCE((SELECT jsonb_agg(to_jsonb(p)) FROM (SELECT * FROM problems LIMIT 20) p), '[]'::jsonb),
            audit_chain_unsealed(chained)
        FROM span
        RETURNING problems INTO found_problems;
        total := total + found_problems;
    END LOOP;
    DELETE FROM audit_chain_links WHERE row_created_at < now() - interval '100 days';
    RETURN total;
END $$;

CREATE OR REPLACE TRIGGER ai_requests_audit_update
    BEFORE UPDATE ON ai_requests FOR EACH ROW EXECUTE FUNCTION audit_guard_ai_requests_update();
CREATE OR REPLACE TRIGGER ai_requests_audit_delete
    BEFORE DELETE ON ai_requests FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('90', 'created_at', 'id');
CREATE OR REPLACE TRIGGER ai_requests_audit_truncate
    BEFORE TRUNCATE ON ai_requests FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER ai_safety_findings_audit_update
    BEFORE UPDATE ON ai_safety_findings FOR EACH ROW EXECUTE FUNCTION audit_guard_append_only('id');
CREATE OR REPLACE TRIGGER ai_safety_findings_audit_delete
    BEFORE DELETE ON ai_safety_findings FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('90', 'created_at', 'id');
CREATE OR REPLACE TRIGGER ai_safety_findings_audit_truncate
    BEFORE TRUNCATE ON ai_safety_findings FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER ai_request_attributions_audit_update
    BEFORE UPDATE ON ai_request_attributions FOR EACH ROW EXECUTE FUNCTION audit_guard_append_only('request_id');
CREATE OR REPLACE TRIGGER ai_request_attributions_audit_delete
    BEFORE DELETE ON ai_request_attributions FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('never', '', 'request_id');
CREATE OR REPLACE TRIGGER ai_request_attributions_audit_truncate
    BEFORE TRUNCATE ON ai_request_attributions FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER governance_decisions_audit_delete
    BEFORE DELETE ON governance_decisions FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('30', 'created_at', 'id');
CREATE OR REPLACE TRIGGER governance_decisions_audit_truncate
    BEFORE TRUNCATE ON governance_decisions FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER mcp_tool_executions_audit_delete
    BEFORE DELETE ON mcp_tool_executions FOR EACH ROW
    EXECUTE FUNCTION audit_guard_delete('30', 'created_at', 'mcp_execution_id');
CREATE OR REPLACE TRIGGER mcp_tool_executions_audit_truncate
    BEFORE TRUNCATE ON mcp_tool_executions FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER plugin_usage_events_audit_delete
    BEFORE DELETE ON plugin_usage_events FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('90', 'created_at', 'id');
CREATE OR REPLACE TRIGGER plugin_usage_events_audit_truncate
    BEFORE TRUNCATE ON plugin_usage_events FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER secret_audit_log_audit_update
    BEFORE UPDATE ON secret_audit_log FOR EACH ROW EXECUTE FUNCTION audit_guard_append_only('id');
CREATE OR REPLACE TRIGGER secret_audit_log_audit_delete
    BEFORE DELETE ON secret_audit_log FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('365', 'created_at', 'id');
CREATE OR REPLACE TRIGGER secret_audit_log_audit_truncate
    BEFORE TRUNCATE ON secret_audit_log FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER platform_data_clear_audit_audit_update
    BEFORE UPDATE ON platform_data_clear_audit FOR EACH ROW EXECUTE FUNCTION audit_guard_append_only('id');
CREATE OR REPLACE TRIGGER platform_data_clear_audit_audit_delete
    BEFORE DELETE ON platform_data_clear_audit FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('never', '', 'id');
CREATE OR REPLACE TRIGGER platform_data_clear_audit_audit_truncate
    BEFORE TRUNCATE ON platform_data_clear_audit FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER audit_chain_links_audit_update
    BEFORE UPDATE ON audit_chain_links FOR EACH ROW EXECUTE FUNCTION audit_guard_append_only('row_id');
CREATE OR REPLACE TRIGGER audit_chain_links_audit_delete
    BEFORE DELETE ON audit_chain_links FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('90', 'row_created_at', 'row_id');
CREATE OR REPLACE TRIGGER audit_chain_links_audit_truncate
    BEFORE TRUNCATE ON audit_chain_links FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();

CREATE OR REPLACE TRIGGER audit_guard_violations_audit_update
    BEFORE UPDATE ON audit_guard_violations FOR EACH ROW EXECUTE FUNCTION audit_guard_append_only('id');
CREATE OR REPLACE TRIGGER audit_guard_violations_audit_delete
    BEFORE DELETE ON audit_guard_violations FOR EACH ROW EXECUTE FUNCTION audit_guard_delete('365', 'created_at', 'id');
CREATE OR REPLACE TRIGGER audit_guard_violations_audit_truncate
    BEFORE TRUNCATE ON audit_guard_violations FOR EACH STATEMENT EXECUTE FUNCTION audit_guard_truncate();
