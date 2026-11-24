-- One row per MCP tool call a Drive page asked the artifacts broker to make
-- as the person viewing it: who, which document, which server and tool, the
-- outcome (`ok` or the refusal's error key) and how long it took. Never the
-- call's input or its result. Append-only and without foreign keys, so the
-- trail outlives the document and the person it names.
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
