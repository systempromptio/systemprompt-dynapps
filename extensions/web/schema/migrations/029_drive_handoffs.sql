-- @cost: rows=0 measured=0.01s triggers=live
-- The session handoff codes. New table only; no rows rewritten.
-- Same DDL as schema/59_drive_handoffs.sql.
CREATE TABLE IF NOT EXISTS drive_handoffs (
    code_hash TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    path TEXT NOT NULL CHECK (path LIKE '/drive/%'),
    authenticated_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_drive_handoffs_expiry
    ON drive_handoffs (expires_at);
