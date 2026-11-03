-- DynWork relay: the devices that may hold a relay socket, and the one-time
-- pairing tokens a desktop hands to a phone.
--
-- The relay routes end-to-end-encrypted envelopes and stores no session
-- content. A device row holds its peer id (the first 16 bytes of SHA-256 of
-- the device's static X25519 key, as lowercase hex), never the key itself,
-- and a phone's credential is kept only as its SHA-256. A revoked row stays
-- for the audit trail; the partial unique index lets the same peer pair again
-- after a revocation.

CREATE TABLE IF NOT EXISTS dynwork_relay_devices (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL CHECK (kind IN ('desktop', 'phone')),
    peer_id         TEXT NOT NULL CHECK (peer_id ~ '^[0-9a-f]{32}$'),
    name            TEXT NOT NULL DEFAULT '',
    credential_hash TEXT UNIQUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at    TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    CHECK ((kind = 'phone') = (credential_hash IS NOT NULL))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_dynwork_relay_devices_live_peer
    ON dynwork_relay_devices(user_id, peer_id) WHERE revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_dynwork_relay_devices_revoked
    ON dynwork_relay_devices(revoked_at) WHERE revoked_at IS NOT NULL;

-- One row per pairing token, which is opaque (`dwp1_` + 32 random bytes) and
-- kept only as its SHA-256. A claim locks the row it finds by that hash and
-- stamps `claimed_at` in the same transaction, so a token is redeemed at most
-- once.
CREATE TABLE IF NOT EXISTS dynwork_relay_pairings (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    claimed_at  TIMESTAMPTZ,
    device_id   TEXT REFERENCES dynwork_relay_devices(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_dynwork_relay_pairings_open
    ON dynwork_relay_pairings(user_id, expires_at) WHERE claimed_at IS NULL;
