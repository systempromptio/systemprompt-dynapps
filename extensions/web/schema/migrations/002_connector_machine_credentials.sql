-- Machine credentials for workflow service accounts. New table only; no rows
-- rewritten. Same DDL as schema/53_connector_machine_credentials.sql.
CREATE TABLE IF NOT EXISTS mcp_connector_machine_credentials (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider ~ '^[A-Za-z0-9_-]{1,128}$'),
    kind TEXT NOT NULL CHECK (kind IN (
        'github_app', 'salesforce_client_credentials', 'atlassian_api_token')),
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    secret_ciphertext BYTEA NOT NULL,
    secret_nonce BYTEA NOT NULL CHECK (octet_length(secret_nonce) = 12),
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, provider)
);
