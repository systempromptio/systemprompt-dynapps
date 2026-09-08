CREATE TABLE IF NOT EXISTS knowledge_drive_sources (
    source_type TEXT PRIMARY KEY,
    drive_id TEXT NOT NULL UNIQUE,
    page_token TEXT,
    last_inventory_at TIMESTAMPTZ,
    last_sync_at TIMESTAMPTZ,
    requested BIGINT NOT NULL DEFAULT 0,
    completed BIGINT NOT NULL DEFAULT 0,
    force_requested BOOLEAN NOT NULL DEFAULT FALSE,
    rebuild_requested BOOLEAN NOT NULL DEFAULT FALSE,
    requested_by TEXT NOT NULL,
    last_error TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS knowledge_drive_files (
    source_type TEXT NOT NULL REFERENCES knowledge_drive_sources(source_type),
    file_id TEXT NOT NULL,
    metadata JSONB NOT NULL,
    content_version TEXT NOT NULL,
    project_id TEXT NOT NULL DEFAULT 'unassigned',
    project_name TEXT NOT NULL DEFAULT 'Unassigned',
    folder_path TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    removed BOOLEAN NOT NULL DEFAULT FALSE,
    seen_generation TEXT,
    extracted_text TEXT,
    extractor_version TEXT NOT NULL DEFAULT '',
    rebuild BOOLEAN NOT NULL DEFAULT FALSE,
    extraction_key TEXT,
    invocation_arn TEXT,
    output_uri TEXT,
    error TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    extraction_attempt INTEGER NOT NULL DEFAULT 0,
    retry_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    indexed_at TIMESTAMPTZ,
    PRIMARY KEY(source_type, file_id)
);
CREATE INDEX IF NOT EXISTS knowledge_drive_work
    ON knowledge_drive_files(source_type, status, retry_at);
CREATE INDEX IF NOT EXISTS knowledge_drive_project
    ON knowledge_drive_files(source_type, project_id);

CREATE TABLE IF NOT EXISTS knowledge_drive_document_projects (
    doc_id TEXT PRIMARY KEY REFERENCES knowledge_documents(doc_id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    file_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    project_name TEXT NOT NULL,
    folder_path TEXT NOT NULL,
    UNIQUE(source_type, file_id)
);
CREATE INDEX IF NOT EXISTS knowledge_drive_document_project
    ON knowledge_drive_document_projects(project_id);
