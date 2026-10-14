-- Odoo credentials: the `odoo_api_key` machine credential kind, a person's
-- own linked Odoo account, and the audit trail for both. The CHECK is widened
-- in place; the new tables are new; no rows are rewritten. Same DDL as
-- schema/53_connector_machine_credentials.sql and schema/54_odoo_credentials.sql.

ALTER TABLE mcp_connector_machine_credentials
    DROP CONSTRAINT IF EXISTS mcp_connector_machine_credentials_kind_check;
ALTER TABLE mcp_connector_machine_credentials
    ADD CONSTRAINT mcp_connector_machine_credentials_kind_check CHECK (kind IN (
        'github_app', 'salesforce_client_credentials', 'atlassian_api_token',
        'odoo_api_key'));

CREATE TABLE IF NOT EXISTS odoo_user_links (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    odoo_url TEXT NOT NULL,
    odoo_db TEXT NOT NULL,
    odoo_login TEXT NOT NULL,
    odoo_uid BIGINT NOT NULL CHECK (odoo_uid > 0),
    api_key_ciphertext BYTEA NOT NULL,
    api_key_nonce BYTEA NOT NULL CHECK (octet_length(api_key_nonce) = 12),
    linked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS connector_credential_audit (
    id BIGSERIAL PRIMARY KEY,
    subject_user_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN (
        'attach', 'rotate', 'remove', 'link', 'relink', 'unlink')),
    actor_user_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_connector_credential_audit_subject
    ON connector_credential_audit (subject_user_id, created_at DESC);
