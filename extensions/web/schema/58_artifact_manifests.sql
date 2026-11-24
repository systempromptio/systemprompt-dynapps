-- A live Drive page's manifest and what each viewer decided about it.
--
-- `artifact_capabilities`: the capabilities the agent declared for an HTML
-- document (`{mcp: {servers: [{server, tools}]}, artifact, user}`) and the
-- runtime contract it was declared against. One row per document; a new
-- declaration replaces it whole.
-- `artifact_consents`: per viewer, document and scope (`mcp:<server>`), the
-- viewer's standing answer. No row is "not asked yet".
-- `artifact_approvals`: one-shot approvals of a write, each bound to the
-- viewer, the document, the server, the tool and the hash of the input, and
-- spent by the call that uses it.
-- `artifact_call_cache`: a read's result per viewer, document and call (the
-- input hashed with its keys sorted), kept until `expires_at` (at most
-- 7 days). Writes are never stored.
-- No foreign keys: a document's rows go with it through the broker's own
-- checks, never through cascades on core's tables.
CREATE TABLE IF NOT EXISTS artifact_capabilities (
    document_id UUID PRIMARY KEY,
    capabilities JSONB NOT NULL,
    contract_version TEXT NOT NULL CHECK (length(contract_version) > 0),
    declared_by TEXT NOT NULL,
    declared_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS artifact_consents (
    user_id TEXT NOT NULL,
    document_id UUID NOT NULL,
    scope TEXT NOT NULL CHECK (length(scope) > 0),
    granted BOOLEAN NOT NULL,
    decided_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, document_id, scope)
);

CREATE TABLE IF NOT EXISTS artifact_approvals (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    document_id UUID NOT NULL,
    server TEXT NOT NULL,
    tool TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_artifact_approvals_expiry
    ON artifact_approvals (expires_at);

CREATE TABLE IF NOT EXISTS artifact_call_cache (
    user_id TEXT NOT NULL,
    document_id UUID NOT NULL,
    server TEXT NOT NULL,
    tool TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    result JSONB NOT NULL,
    stored_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id, document_id, server, tool, input_hash)
);

CREATE INDEX IF NOT EXISTS idx_artifact_call_cache_expiry
    ON artifact_call_cache (user_id, expires_at);
