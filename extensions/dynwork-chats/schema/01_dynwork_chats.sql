-- DynWork cloud chats: threads, messages and the live events of a running
-- reply, per user.
--
-- This is product data, separate from core's 30-day `ai_requests` audit.
-- Every row is reached through its thread, and every thread through its
-- `user_id`: no query reads across users.
--
-- `change_seq` orders the per-user changes feed. It comes from one sequence
-- and is taken while the writer holds the owner's `users` row lock, so for one
-- user the sequence order is the commit order and a reader that resumes from a
-- cursor never skips a change. A deleted thread stays as a tombstone
-- (`deleted_at`) so clients that cache it can drop it; its messages and events
-- are removed with it.
--
-- `revision` guards what a person edits (title, model, archived, deleted).
-- Activity (a message, the end of a reply) moves `updated_at` and
-- `change_seq` but not `revision`, so a rename is never refused because a
-- reply was streaming.
--
-- `last_message_seq` and `last_event_seq` are the per-thread counters the
-- `seq` columns below are taken from. They live on the thread, not as
-- MAX(seq), so pruning events never makes a seq repeat.

CREATE SEQUENCE IF NOT EXISTS dynwork_chat_change_seq;

CREATE TABLE IF NOT EXISTS dynwork_chat_threads (
    id               TEXT PRIMARY KEY CHECK (id ~ '^ses_[A-Za-z0-9]{8,40}$'),
    user_id          TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title            TEXT NOT NULL DEFAULT '',
    model            JSONB,
    revision         BIGINT NOT NULL DEFAULT 1,
    archived_at      TIMESTAMPTZ,
    deleted_at       TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL,
    updated_at       TIMESTAMPTZ NOT NULL,
    change_seq       BIGINT NOT NULL,
    last_message_seq BIGINT NOT NULL DEFAULT 0,
    last_event_seq   BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_dynwork_chat_threads_user_change
    ON dynwork_chat_threads(user_id, change_seq);

-- OpenCode message info and parts, kept as jsonb and never interpreted beyond
-- `role`. An assistant message is inserted when its prompt is accepted, with
-- no parts, and filled in when its reply ends.
CREATE TABLE IF NOT EXISTS dynwork_chat_messages (
    id           TEXT NOT NULL CHECK (id ~ '^msg_[A-Za-z0-9]{8,40}$'),
    thread_id    TEXT NOT NULL REFERENCES dynwork_chat_threads(id) ON DELETE CASCADE,
    role         TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
    info         JSONB NOT NULL,
    parts        JSONB NOT NULL DEFAULT '[]'::jsonb,
    seq          BIGINT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    PRIMARY KEY (thread_id, id),
    UNIQUE (thread_id, seq)
);

-- The live events of a reply, so a late reader resumes after a `seq`.
-- `message_id` is the reply an event belongs to. Compacting a reply into its
-- message stamps `compacted_at`; pruning removes events 24 h after that, and
-- events that belong to no reply by their own age.
CREATE TABLE IF NOT EXISTS dynwork_chat_events (
    thread_id    TEXT NOT NULL REFERENCES dynwork_chat_threads(id) ON DELETE CASCADE,
    seq          BIGINT NOT NULL,
    message_id   TEXT,
    type         TEXT NOT NULL,
    data         JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL,
    compacted_at TIMESTAMPTZ,
    PRIMARY KEY (thread_id, seq)
);

CREATE INDEX IF NOT EXISTS idx_dynwork_chat_events_compacted
    ON dynwork_chat_events(compacted_at) WHERE compacted_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_dynwork_chat_events_loose
    ON dynwork_chat_events(created_at) WHERE message_id IS NULL;
