-- What the admin's Brain section reads beyond the content: one row per search
-- or answer (counts only: no person, no device, no address) and the audit of
-- what an administrator did that is not a plain read, "Run as" included.
-- `article_ids` are the documents the search returned or the answer cited.
CREATE TABLE IF NOT EXISTS dynbrain_search_log (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    kind        TEXT NOT NULL CHECK (kind IN ('search', 'answer')),
    query_text  TEXT NOT NULL,
    space_id    BIGINT,
    article_ids BIGINT[] NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_dynbrain_search_log_created
    ON dynbrain_search_log(created_at);

CREATE TABLE IF NOT EXISTS dynbrain_admin_audit (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor_id   TEXT NOT NULL,
    action     TEXT NOT NULL,
    subject_id TEXT,
    detail     TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_dynbrain_admin_audit_created
    ON dynbrain_admin_audit(created_at);
