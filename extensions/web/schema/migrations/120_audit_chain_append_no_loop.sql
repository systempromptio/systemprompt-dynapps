-- Seed the chain head with one insert-or-skip, then lock it, instead of
-- looping until a SELECT finds it. Under READ COMMITTED a concurrent seed is
-- waited for by ON CONFLICT and visible to the SELECT that follows, so the
-- head is always found once; the loop only made every chained row write look
-- like a per-row fan-out. Same body as schema/51_audit_integrity.sql.

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
