-- The files of the Brain's articles. The bytes live in core's file storage
-- (S3 in production, the same store as chat uploads), under the `dynbrain/`
-- prefix; these tables keep only the storage reference, never the bytes.

-- One upload slot: issued to one person with a signed one-time URL, it takes
-- one POST of the file before `expires_at` (`received_at`), and then one
-- adoption as a file of an article by that same person (`consumed_at`).
-- `mimetype` is detected from the bytes, never taken from the client.
CREATE TABLE IF NOT EXISTS dynbrain_upload (
    id          TEXT PRIMARY KEY CHECK (id ~ '^[0-9a-f]{32}$'),
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    received_at TIMESTAMPTZ,
    storage_ref TEXT,
    size_bytes  BIGINT CHECK (size_bytes IS NULL OR size_bytes >= 0),
    sha256      TEXT,
    mimetype    TEXT,
    consumed_at TIMESTAMPTZ
);

-- A file of one article. `lang` is the variant it belongs to, or NULL when it
-- serves every language. Markdown names it `asset://<id>`. A file follows its
-- article into the trash; `trashed_at` takes out one file on its own.
CREATE TABLE IF NOT EXISTS dynbrain_asset (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    article_id  BIGINT NOT NULL REFERENCES dynbrain_article(id) ON DELETE CASCADE,
    lang        TEXT,
    name        TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 255),
    mimetype    TEXT NOT NULL,
    size_bytes  BIGINT NOT NULL CHECK (size_bytes >= 0),
    sha256      TEXT NOT NULL,
    storage_ref TEXT NOT NULL,
    upload_id   TEXT NOT NULL UNIQUE,
    created_by  TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    trashed_at  TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_dynbrain_asset_article ON dynbrain_asset(article_id);
