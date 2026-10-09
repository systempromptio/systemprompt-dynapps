-- The audit hash chain moves off the insert path (NFR-5.6 / REQ-026).
--
-- The AFTER INSERT row triggers on ai_requests and ai_safety_findings locked
-- audit_chain_heads FOR UPDATE until the inserting transaction committed, so
-- every gateway request in the cluster queued on one row per table. They are
-- dropped; the `audit_chain_seal` job links new rows every 10 s instead,
-- single writer under an advisory lock, reading from audit_chain_cursors.
-- The UPDATE/DELETE/TRUNCATE guards are unchanged, so tamper refusal stays
-- immediate; only the seal is eventually consistent (within one interval).
-- Each cursor starts at the newest row already linked (or now(), when the
-- chain was off), so nothing is re-linked and no backlog is chained.
-- verify_audit_chain stops reporting "no chain link" for rows the sealer has
-- not passed yet, and each verification records how many are waiting.
-- New table, one new column with a constant default, functions only; no
-- audit rows rewritten. Same end state as schema/51_audit_integrity.sql.

DROP TRIGGER IF EXISTS ai_requests_audit_chain ON ai_requests;
DROP TRIGGER IF EXISTS ai_safety_findings_audit_chain ON ai_safety_findings;
DROP FUNCTION IF EXISTS audit_chain_link_ai_requests();
DROP FUNCTION IF EXISTS audit_chain_link_ai_safety_findings();

ALTER TABLE audit_chain_verifications ADD COLUMN IF NOT EXISTS unsealed_rows BIGINT NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS audit_chain_cursors (
    table_name TEXT PRIMARY KEY,
    last_created_at TIMESTAMPTZ NOT NULL,
    last_id TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO audit_chain_cursors (table_name, last_created_at, last_id)
SELECT t.table_name,
    COALESCE(newest.row_created_at, now()),
    COALESCE(newest.row_id, '')
FROM (VALUES ('ai_requests'), ('ai_safety_findings')) AS t(table_name)
LEFT JOIN LATERAL (
    SELECT k.row_created_at, k.row_id FROM audit_chain_links k
    WHERE k.table_name = t.table_name
    ORDER BY k.row_created_at DESC, k.row_id DESC LIMIT 1
) newest ON TRUE
ON CONFLICT (table_name) DO NOTHING;

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
