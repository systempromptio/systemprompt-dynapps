-- @cost: rows=0 measured=0.01s triggers=live
-- The artifacts broker's audit rows. New table only; no rows rewritten.
-- Same DDL as schema/57_artifact_tool_calls.sql.
CREATE TABLE IF NOT EXISTS artifact_tool_calls (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    document_id UUID NOT NULL,
    server TEXT NOT NULL,
    tool TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK (length(outcome) > 0),
    duration_ms INTEGER NOT NULL CHECK (duration_ms >= 0),
    called_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_artifact_tool_calls_document
    ON artifact_tool_calls (document_id, called_at DESC);

CREATE INDEX IF NOT EXISTS idx_artifact_tool_calls_user
    ON artifact_tool_calls (user_id, called_at DESC);
