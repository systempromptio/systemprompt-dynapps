-- @cost: rows=0 measured=0.01s triggers=live
-- A live page's manifest, consents, write approvals and read cache. New
-- tables only; no rows rewritten. Same DDL as schema/58_artifact_manifests.sql.
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
