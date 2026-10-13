-- Machine credentials held by service accounts: the non-interactive users a
-- workflow definition runs as. One row per (service account, provider). The
-- secret half (GitHub App private key, Salesforce client secret, Atlassian
-- API token) is sealed with the same master key as hosted MCP grants; only
-- non-secret identifiers live in `config`, so the console can show them.
-- The broker refuses a row whose user is not a service account; the table
-- cannot express that rule because `users.account_kind` belongs to core.
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
