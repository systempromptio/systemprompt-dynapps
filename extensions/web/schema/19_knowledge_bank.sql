-- Enterprise knowledge bank: the RAG index and its per-document access control.
--
-- These tables are read and written by the knowledge-bank MCP server
-- (extensions/mcp/knowledge-bank), which is a standalone binary outside the
-- extension registry and therefore cannot register schema of its own. They are
-- declared here because extensions/web is the only extension that carries a
-- schema inventory (src/schemas.rs).
--
-- knowledge_document_grants is the REQ-052 (P0) enforcement surface. Grants are
-- subject-scoped using the same vocabulary as access_control_rules — user,
-- department, adfs_group, role, organization — so a document and a marketplace
-- entitlement describe a principal identically. A document with no grants is
-- reachable by nobody: ingestion writes grants from the source system's own
-- permissions, so a document whose permissions could not be determined fails
-- closed rather than open.

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS knowledge_documents (
    doc_id TEXT PRIMARY KEY,
    -- The source category (meeting_notes, confluence, jira, ...), matching a
    -- key in services/knowledge-bank/config.yaml `sources:`.
    source_type TEXT NOT NULL,
    -- The originating system, for lineage in the REQ-048 sense.
    source TEXT NOT NULL,
    title TEXT NOT NULL,
    -- Canonical link back to the original so an agent can open it.
    uri TEXT NOT NULL,
    doc_date TIMESTAMPTZ,
    date_source TEXT,
    -- Together with mtime, decides whether an incremental sync re-chunks the
    -- document or leaves the existing chunks alone.
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
    text TEXT NOT NULL,
    -- Titan v2 at 1024 dimensions. Changing the embedding model changes this
    -- width, which is a full index rebuild, not a migration.
    embedding vector(1024) NOT NULL,
    -- Line span in the original, rendered into the hit's uri.
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
-- re-reads on every call so a rebuild is picked up without a restart, matching
-- the .index/.version contract of the Node system this is a migration of.
CREATE TABLE IF NOT EXISTS knowledge_index_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
