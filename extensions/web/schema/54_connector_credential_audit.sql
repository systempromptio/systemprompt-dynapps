-- Who attached, rotated or removed a service account's machine credential.
-- Append-only and without a foreign key, so the trail outlives the account it
-- describes. `link`, `relink` and `unlink` are kept for the rows the retired
-- Odoo account link wrote.
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
