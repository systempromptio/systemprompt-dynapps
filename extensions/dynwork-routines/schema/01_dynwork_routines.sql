-- DynWork routines: each person's routine definitions, kept here so all of
-- their DynWork desktops share them, and a summary of each routine's runs.
--
-- A routine's id is chosen by the desktop that created it and is scoped to
-- its user. `revision` is the optimistic-concurrency token a write must name
-- (If-Match). `change_seq` orders the user's changes for the feed: writes for
-- one user take that user's row lock and number their change one past the
-- user's highest, so a change becomes visible only after every lower one
-- has. A delete keeps the row as a tombstone (`deleted_at`, the definition
-- emptied) so every desktop learns of it. `owner_device` is the computer
-- that runs the routine; the conversation of a run never leaves it.

CREATE TABLE IF NOT EXISTS dynwork_routines (
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    id            TEXT NOT NULL,
    revision      BIGINT NOT NULL CHECK (revision > 0),
    change_seq    BIGINT NOT NULL CHECK (change_seq > 0),
    owner_device  TEXT NOT NULL,
    owner_name    TEXT NOT NULL DEFAULT '',
    definition    JSONB NOT NULL,
    deleted_at    TIMESTAMPTZ,
    updated_at    TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id, id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_dynwork_routines_changes
    ON dynwork_routines(user_id, change_seq);

-- One row per run, written by the routine's owner device: its state and
-- timestamps only, never the conversation.
CREATE TABLE IF NOT EXISTS dynwork_routine_runs (
    user_id     TEXT NOT NULL,
    id          TEXT NOT NULL,
    routine_id  TEXT NOT NULL,
    state       TEXT NOT NULL CHECK (state IN (
        'queued', 'running', 'ok', 'late', 'failed', 'blocked', 'missed', 'skipped', 'cancelled'
    )),
    reason      TEXT,
    started_at  TIMESTAMPTZ,
    ended_at    TIMESTAMPTZ,
    updated_at  TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id, id),
    FOREIGN KEY (user_id, routine_id)
        REFERENCES dynwork_routines(user_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dynwork_routine_runs_routine
    ON dynwork_routine_runs(user_id, routine_id, updated_at DESC);
