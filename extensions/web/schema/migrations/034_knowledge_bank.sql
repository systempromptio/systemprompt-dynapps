-- Converge established databases onto schema/19_knowledge_bank.sql.
--
-- The declarative file creates these tables on a fresh install; this migration
-- is what an already-running instance applies. Every statement is idempotent,
-- so a database that somehow has part of the shape converges rather than
-- failing halfway.
--
-- CREATE EXTENSION requires a Postgres image that ships pgvector. The local
-- Docker image is switched to pgvector/pgvector:pg18 alongside this migration;
-- an instance whose server lacks the extension fails loudly here, which is the
-- intended outcome — silently degrading to no vector search would make the
-- knowledge bank answer with text matches only and never say so.
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS knowledge_documents (
    doc_id TEXT PRIMARY KEY,
    source_type TEXT NOT NULL,
    source TEXT NOT NULL,
    title TEXT NOT NULL,
    uri TEXT NOT NULL,
    doc_date TIMESTAMPTZ,
    date_source TEXT,
    content_hash TEXT NOT NULL,
    mtime TIMESTAMPTZ,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_knowledge_documents_source_type
    ON knowledge_documents (source_type);

CREATE TABLE IF NOT EXISTS knowledge_chunks (
    chunk_id TEXT PRIMARY KEY,
    doc_id TEXT NOT NULL REFERENCES knowledge_documents (doc_id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    embedding vector(1024) NOT NULL,
    lines TEXT,
    chunk_index INTEGER NOT NULL,
    chunk_total INTEGER NOT NULL,
    tsv tsvector GENERATED ALWAYS AS (to_tsvector('english', text)) STORED
);

CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_doc
    ON knowledge_chunks (doc_id);

CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_embedding
    ON knowledge_chunks USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_tsv
    ON knowledge_chunks USING gin (tsv);

CREATE TABLE IF NOT EXISTS knowledge_document_grants (
    doc_id TEXT NOT NULL REFERENCES knowledge_documents (doc_id) ON DELETE CASCADE,
    subject_kind TEXT NOT NULL,
    subject_value TEXT NOT NULL,
    PRIMARY KEY (doc_id, subject_kind, subject_value)
);

CREATE INDEX IF NOT EXISTS idx_knowledge_document_grants_subject
    ON knowledge_document_grants (subject_kind, subject_value);

CREATE TABLE IF NOT EXISTS knowledge_index_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
