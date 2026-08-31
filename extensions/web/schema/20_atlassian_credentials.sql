-- Per-user Atlassian OAuth 2.0 (3LO) credential.
--
-- One row per linked user. The refresh token is rotating: every use returns a
-- new one and kills the old, so this row IS the user's Atlassian session and a
-- lost write logs them out. Any read that may refresh takes
-- `SELECT ... FOR UPDATE`, because two tool calls in one agent turn (a Jira
-- lookup and a Confluence read) is the ordinary case, not an edge case.
--
-- Both tokens are ChaCha20-Poly1305 sealed under the caller's per-user DEK from
-- `user_encryption_keys` (09_secrets.sql) — the same envelope `plugin_env_vars`
-- uses. `key_version` exists so a DEK rotation can find rows it must re-wrap;
-- `secret_keys::rotate_user_dek` walks this table for that reason.
--
-- The access token is cached until `access_token_expires_at` so the ordinary
-- tool call is a decrypt rather than a token-endpoint round trip. That cache is
-- what keeps refreshes rare enough for the row lock to be uncontended.
--
-- `cloud_id`/`cloud_url` are captured free at link time from Atlassian's
-- accessible-resources endpoint. The hosted Remote MCP server resolves the site
-- from the token's own consent, but a direct REST path
-- (api.atlassian.com/ex/jira/{cloud_id}/...) would otherwise cost an extra round
-- trip per call, and the site URL is what lets the profile page say which
-- Atlassian site a user actually connected.

CREATE TABLE IF NOT EXISTS atlassian_user_credentials (
    user_id                 TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    atlassian_account_id    TEXT NOT NULL,
    encrypted_refresh_token BYTEA NOT NULL,
    refresh_token_nonce     BYTEA NOT NULL,
    encrypted_access_token  BYTEA,
    access_token_nonce      BYTEA,
    access_token_expires_at TIMESTAMPTZ,
    cloud_id                TEXT,
    cloud_url               TEXT,
    granted_scopes          TEXT NOT NULL DEFAULT '',
    key_version             INTEGER NOT NULL DEFAULT 1,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
