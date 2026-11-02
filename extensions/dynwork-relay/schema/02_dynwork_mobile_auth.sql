-- DynWork mobile sign-in: one row per consent the instance showed a signed-in
-- user for a phone.
--
-- The authorize page inserts the row with the phone's PKCE S256 challenge and
-- its `state`, keyed by the SHA-256 of the consent request token the page's
-- form carries back. Allow stamps the SHA-256 of a one-time code and moves
-- `expires_at` to ten minutes from then; Deny stamps the decision only. The
-- token exchange locks the row by the code's hash and stamps `redeemed_at` in
-- the same transaction that creates the phone's device row, so a code is
-- redeemed at most once, and only with the verifier of its challenge. No
-- secret is stored: the request token, the code and the device credential are
-- kept as hashes, and the challenge and `state` are public by design.

CREATE TABLE IF NOT EXISTS dynwork_relay_mobile_grants (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    request_hash    TEXT NOT NULL UNIQUE,
    code_hash       TEXT UNIQUE,
    code_challenge  TEXT NOT NULL CHECK (code_challenge ~ '^[A-Za-z0-9_-]{43}$'),
    state           TEXT NOT NULL,
    device_name     TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    decided_at      TIMESTAMPTZ,
    approved        BOOLEAN,
    redeemed_at     TIMESTAMPTZ,
    device_id       TEXT REFERENCES dynwork_relay_devices(id) ON DELETE SET NULL,
    CHECK ((decided_at IS NULL) = (approved IS NULL)),
    CHECK (code_hash IS NULL OR approved)
);

CREATE INDEX IF NOT EXISTS idx_dynwork_relay_mobile_grants_open
    ON dynwork_relay_mobile_grants(user_id, expires_at) WHERE decided_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_dynwork_relay_mobile_grants_expiry
    ON dynwork_relay_mobile_grants(expires_at);
