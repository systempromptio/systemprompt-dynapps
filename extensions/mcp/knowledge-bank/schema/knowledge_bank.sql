-- Knowledge bank: the project-context RAG index and its per-document access control.
--
-- Owned and applied by the knowledge-bank MCP binary (src/schema.rs), not by the
-- web extension's schema inventory: it needs pgvector, and a deployment whose
-- Postgres lacks the extension must boot with the knowledge bank inert rather
-- than fail platform-wide. Every statement is idempotent so the file can be
-- re-applied on each start.
--
-- knowledge_document_grants is the REQ-052 (P0) enforcement surface. Grants are
-- subject-scoped using the same vocabulary as the principal module (user, role,
-- and the attribute bands such as adfs_group), so a document and a marketplace
-- entitlement describe a principal identically. A document with no grants is
-- reachable by nobody.

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS knowledge_documents (
    doc_id TEXT PRIMARY KEY,
    -- The source category (meeting_notes, confluence, jira, ...), matching a
    -- key in services/knowledge-bank/config.yaml `sources:`.
    source_type TEXT NOT NULL,
    -- The originating system, for lineage in the REQ-048 sense.
    source TEXT NOT NULL,
    -- The source system's own identifier: page id, issue key, relative path.
    source_ref TEXT NOT NULL,
    title TEXT NOT NULL,
    -- Canonical pointer back to the original so an agent can open it.
    uri TEXT NOT NULL,
    doc_date DATE,
    -- frontmatter | llm | mtime — the provenance the agent policy ranks by.
    date_source TEXT,
    -- Decides whether an incremental sync re-chunks the document or leaves
    -- the existing chunks alone.
    content_hash TEXT NOT NULL,
    mtime TIMESTAMPTZ,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Archive categories are opt-in: excluded unless the caller names them.
    archived BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_knowledge_documents_source_type
    ON knowledge_documents (source_type);

CREATE TABLE IF NOT EXISTS knowledge_chunks (
    chunk_id TEXT PRIMARY KEY,
    doc_id TEXT NOT NULL REFERENCES knowledge_documents (doc_id) ON DELETE CASCADE,
    -- What was embedded: breadcrumb and enrichment prefix included.
    text TEXT NOT NULL,
    -- Titan v2 at 1024 dimensions. Changing the embedding model changes this
    -- width, which is a full index rebuild, not a migration.
    embedding vector(1024) NOT NULL,
    -- Heading trail the chunk sits under, kept apart from the pointer.
    breadcrumb TEXT NOT NULL DEFAULT '',
    -- "start-end" line span in the original text.
    lines TEXT,
    chunk_index INTEGER NOT NULL,
    chunk_total INTEGER NOT NULL,
    tsv tsvector GENERATED ALWAYS AS (to_tsvector('english', text)) STORED
);

CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_doc
    ON knowledge_chunks (doc_id);

-- Cosine distance: Titan embeddings are requested with normalize=true.
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

-- Index-wide state, keyed by name. `version` carries the stamp the retriever
-- reports so a rebuild is observable without a restart, matching the
-- .index/.version contract of the Node system this is a migration of.
CREATE TABLE IF NOT EXISTS knowledge_index_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Per-source incremental cursor: the Atlassian sync skips pages and issues
-- not updated since last_sync, and the report of the last run is kept for
-- list_knowledge_sources.
CREATE TABLE IF NOT EXISTS knowledge_sync_state (
    source_type TEXT PRIMARY KEY,
    last_sync TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    last_error TEXT,
    documents_seen INTEGER NOT NULL DEFAULT 0,
    documents_written INTEGER NOT NULL DEFAULT 0,
    documents_pruned INTEGER NOT NULL DEFAULT 0,
    documents_failed INTEGER NOT NULL DEFAULT 0
);

-- LLM-derived dates are non-deterministic, so the resolved date is frozen per
-- content hash: an unchanged document never re-asks the model (NFR-COST-1).
CREATE TABLE IF NOT EXISTS knowledge_date_cache (
    doc_id TEXT PRIMARY KEY,
    content_hash TEXT NOT NULL,
    doc_date DATE,
    date_source TEXT NOT NULL,
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Embeddings keyed by model and text hash, so a rebuild or a re-chunked
-- document only pays for text that has never been embedded before.
CREATE TABLE IF NOT EXISTS knowledge_embedding_cache (
    embed_model TEXT NOT NULL,
    text_hash TEXT NOT NULL,
    embedding vector(1024) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (embed_model, text_hash)
);
