-- One-shot session handoff codes: a person's `ui` session (DynWork desktop)
-- asks for one, and the client's view redeems it for a web session
-- (`/drive/handoff`). Only the SHA-256 of a code is stored, with its person,
-- the `/drive/` path it opens and the `ui` session's sign-in time, which the
-- web session carries. A code lives 60 s and is deleted when it is spent;
-- storing a code sweeps the expired ones, so nothing else is kept.
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
