-- The release whose content tree was last ingested. Ingestion refuses to run
-- from an older release than the one recorded, so a pod of the previous
-- image restarting mid-rollout cannot ingest its older tree over the newer
-- one (and delete the newer pages as orphans); a site render ingests first
-- when nothing, or an older release, is recorded.
CREATE TABLE IF NOT EXISTS content_ingestion_stamp (
    id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    version TEXT NOT NULL,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
