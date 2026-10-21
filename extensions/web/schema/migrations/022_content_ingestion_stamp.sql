-- The content ingestion stamp: which release last ingested services/content,
-- so an older release never ingests over a newer one.
-- Same DDL as schema/55_content_ingestion_stamp.sql.

CREATE TABLE IF NOT EXISTS content_ingestion_stamp (
    id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    version TEXT NOT NULL,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
