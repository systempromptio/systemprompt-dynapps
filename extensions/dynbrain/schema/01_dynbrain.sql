-- The Brain: documentation spaces, their article trees, one variant per
-- language, an append-only revision history, typed properties, permissions
-- and the retrieval index (chunks, eligibility rules, the indexing queue and
-- the query-embedding cache).
--
-- pgvector is required. On a managed Postgres where `vector` is not a
-- trusted extension the operator creates it beforehand (the chart's CNPG
-- `Database`); with the extension present this statement needs no privilege.
CREATE EXTENSION IF NOT EXISTS vector;

-- A documentation space. `langs` are the languages its articles may be
-- written in; `property_schema` declares the typed properties every article
-- variant of the space may carry (validated on write by the extension).
-- `created_by` sees and manages every article of the space, as administrators
-- do, whatever the grants say.
CREATE TABLE IF NOT EXISTS dynbrain_space (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    slug            TEXT NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9][a-z0-9-]{0,63}$'),
    name            TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
    default_lang    TEXT NOT NULL,
    langs           TEXT[] NOT NULL,
    property_schema JSONB NOT NULL DEFAULT '[]'::jsonb CHECK (jsonb_typeof(property_schema) = 'array'),
    created_by      TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (default_lang = ANY(langs))
);

-- The article tree of a space. A parent always belongs to the same space
-- (the composite foreign key). Trashing stamps `trashed_at` on an article and
-- its whole subtree; nothing is ever hard-deleted by the extension.
-- `rag_mode` forces an article in or out of the retrieval index; `default`
-- leaves it to `dynbrain_rag_rule`.
CREATE TABLE IF NOT EXISTS dynbrain_article (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    space_id   BIGINT NOT NULL REFERENCES dynbrain_space(id) ON DELETE CASCADE,
    parent_id  BIGINT,
    position   INTEGER NOT NULL DEFAULT 0,
    slug       TEXT NOT NULL CHECK (slug ~ '^[a-z0-9][a-z0-9-]{0,127}$'),
    icon       TEXT CHECK (icon IS NULL OR length(icon) BETWEEN 1 AND 64),
    rag_mode   TEXT NOT NULL DEFAULT 'default' CHECK (rag_mode IN ('default', 'always', 'never')),
    created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    trashed_at TIMESTAMPTZ,
    UNIQUE (space_id, id),
    FOREIGN KEY (space_id, parent_id) REFERENCES dynbrain_article(space_id, id) ON DELETE CASCADE,
    CHECK (parent_id IS NULL OR parent_id <> id)
);

CREATE INDEX IF NOT EXISTS idx_dynbrain_article_parent
    ON dynbrain_article(space_id, parent_id, position);

-- Two live siblings never share a slug; a trashed one keeps its slug until
-- it is restored.
CREATE UNIQUE INDEX IF NOT EXISTS idx_dynbrain_article_sibling_slug
    ON dynbrain_article(space_id, COALESCE(parent_id, 0), slug) WHERE trashed_at IS NULL;

-- One variant per article and language: the Markdown body, the free front
-- matter (kept as written) and the typed property values. `content_hash`
-- identifies what the indexer reads (title, description and body) and
-- `revision` counts the writes of this variant.
CREATE TABLE IF NOT EXISTS dynbrain_article_variant (
    article_id   BIGINT NOT NULL REFERENCES dynbrain_article(id) ON DELETE CASCADE,
    lang         TEXT NOT NULL,
    title        TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 500),
    description  TEXT NOT NULL DEFAULT '',
    body         TEXT NOT NULL DEFAULT '',
    front_matter JSONB NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(front_matter) = 'object'),
    properties   JSONB NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(properties) = 'object'),
    content_hash TEXT NOT NULL,
    revision     INTEGER NOT NULL DEFAULT 1,
    updated_by   TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (article_id, lang)
);

-- Every write of a variant, appended in the same transaction. Append-only:
-- the trigger refuses updates. `author_id` carries no foreign key so the
-- history outlives the account that wrote it.
CREATE TABLE IF NOT EXISTS dynbrain_article_revision (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    article_id   BIGINT NOT NULL REFERENCES dynbrain_article(id) ON DELETE CASCADE,
    lang         TEXT NOT NULL,
    revision     INTEGER NOT NULL,
    title        TEXT NOT NULL,
    description  TEXT NOT NULL,
    body         TEXT NOT NULL,
    front_matter JSONB NOT NULL,
    properties   JSONB NOT NULL,
    content_hash TEXT NOT NULL,
    author_id    TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (article_id, lang, revision)
);

CREATE OR REPLACE FUNCTION dynbrain_revision_append_only() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'dynbrain_article_revision is append-only';
END
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_dynbrain_revision_append_only ON dynbrain_article_revision;
CREATE TRIGGER trg_dynbrain_revision_append_only BEFORE UPDATE ON dynbrain_article_revision
    FOR EACH ROW EXECUTE FUNCTION dynbrain_revision_append_only();

-- What a subject may do with an article and, through `inherit`, with the
-- descendants that have no grants of their own. Subjects are Governance's
-- own: a user id, a role name (`users.roles`) or a group id (`groups`).
-- `level`: 1 read, 2 write, 3 manage.
CREATE TABLE IF NOT EXISTS dynbrain_grant (
    article_id   BIGINT NOT NULL REFERENCES dynbrain_article(id) ON DELETE CASCADE,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('user', 'role', 'group')),
    subject_id   TEXT NOT NULL CHECK (length(subject_id) BETWEEN 1 AND 255),
    level        SMALLINT NOT NULL CHECK (level BETWEEN 1 AND 3),
    inherit      BOOLEAN NOT NULL DEFAULT true,
    granted_by   TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (article_id, subject_kind, subject_id)
);

-- The access closure: every subject that reaches an article, at its highest
-- level, after inheritance. It is the only permission source retrieval reads,
-- so it is rewritten in the same transaction as the grant change or move that
-- alters it, never in the background.
CREATE TABLE IF NOT EXISTS dynbrain_access (
    article_id   BIGINT NOT NULL REFERENCES dynbrain_article(id) ON DELETE CASCADE,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('user', 'role', 'group')),
    subject_id   TEXT NOT NULL,
    level        SMALLINT NOT NULL CHECK (level BETWEEN 1 AND 3),
    PRIMARY KEY (article_id, subject_kind, subject_id)
);

CREATE INDEX IF NOT EXISTS idx_dynbrain_access_subject
    ON dynbrain_access(subject_kind, subject_id, article_id);

-- The retrieval index of a variant. `search_vector` is derived from the
-- clean `content` with the variant language's text-search configuration;
-- the contextual prefix only ever goes into `embedding`.
-- `embedding_fingerprint` (`provider:model:dims`) says which configuration
-- produced the vector, so a model change switches the vector arm off instead
-- of comparing incomparable vectors.
CREATE TABLE IF NOT EXISTS dynbrain_chunk (
    id                    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    article_id            BIGINT NOT NULL,
    lang                  TEXT NOT NULL,
    sequence              INTEGER NOT NULL,
    heading_path          TEXT NOT NULL DEFAULT '',
    content               TEXT NOT NULL,
    content_hash          TEXT NOT NULL,
    synopsis              TEXT,
    synopsis_hash         TEXT,
    context_hash          TEXT,
    fts_config            REGCONFIG NOT NULL DEFAULT 'simple',
    search_vector         TSVECTOR GENERATED ALWAYS AS (to_tsvector(fts_config, content)) STORED,
    embedding             VECTOR(1536),
    embedding_fingerprint TEXT,
    embedded_at           TIMESTAMPTZ,
    has_error             BOOLEAN NOT NULL DEFAULT false,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (article_id, lang, sequence),
    FOREIGN KEY (article_id, lang) REFERENCES dynbrain_article_variant(article_id, lang) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dynbrain_chunk_embedding
    ON dynbrain_chunk USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS idx_dynbrain_chunk_search_vector
    ON dynbrain_chunk USING gin (search_vector);

CREATE INDEX IF NOT EXISTS idx_dynbrain_chunk_pending
    ON dynbrain_chunk(id) WHERE embedding IS NULL;

-- Eligibility for the retrieval index, denied by default: an article whose
-- `rag_mode` is `default` is indexed only when the first active rule that
-- matches it, in `sequence` order, says `include`.
CREATE TABLE IF NOT EXISTS dynbrain_rag_rule (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name       TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
    sequence   INTEGER NOT NULL DEFAULT 10,
    active     BOOLEAN NOT NULL DEFAULT true,
    scope      TEXT NOT NULL DEFAULT 'all' CHECK (scope IN ('all', 'space', 'article')),
    space_id   BIGINT REFERENCES dynbrain_space(id) ON DELETE CASCADE,
    article_id BIGINT REFERENCES dynbrain_article(id) ON DELETE CASCADE,
    mode       TEXT NOT NULL DEFAULT 'exclude' CHECK (mode IN ('include', 'exclude')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (
        (scope = 'all' AND space_id IS NULL AND article_id IS NULL)
        OR (scope = 'space' AND space_id IS NOT NULL AND article_id IS NULL)
        OR (scope = 'article' AND space_id IS NULL AND article_id IS NOT NULL)
    )
);

-- The indexing queue. One job per variant content: enqueueing a content
-- hash that is already queued, running or done is a no-op.
CREATE TABLE IF NOT EXISTS dynbrain_index_job (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    article_id   BIGINT NOT NULL REFERENCES dynbrain_article(id) ON DELETE CASCADE,
    lang         TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    state        TEXT NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'running', 'done', 'failed')),
    attempts     INTEGER NOT NULL DEFAULT 0,
    last_error   TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (article_id, lang, content_hash)
);

CREATE INDEX IF NOT EXISTS idx_dynbrain_index_job_pending
    ON dynbrain_index_job(created_at) WHERE state = 'pending';

-- Query embeddings, keyed by sha1(provider:model:normalised query). Only the
-- query vector is cached, never a result set; unused rows age out.
CREATE TABLE IF NOT EXISTS dynbrain_query_cache (
    query_hash TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    provider   TEXT NOT NULL,
    model      TEXT NOT NULL,
    embedding  VECTOR(1536) NOT NULL,
    hits       INTEGER NOT NULL DEFAULT 0,
    last_used  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_dynbrain_query_cache_last_used
    ON dynbrain_query_cache(last_used);
