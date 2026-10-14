-- A person's own Odoo account, linked from the connectors page. One row per
-- user. The API key is sealed with the encryption master key in an envelope
-- bound to the user, like machine credentials; `odoo_uid` is what Odoo's
-- `common.authenticate` answered when the link was verified, and `odoo_url`
-- / `odoo_db` are the instance it was verified against.
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

-- Who attached, rotated or removed a service account's machine credential,
-- and who linked or unlinked a person's Odoo account. Append-only and without
-- a foreign key, so the trail outlives the account it describes.
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
