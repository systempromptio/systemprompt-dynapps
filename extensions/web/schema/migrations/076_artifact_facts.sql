-- Artifacts join the feedback plane.
--
-- Twin of schema/39_artifact_facts.sql: the outbox gains an artifact_id
-- column and admits the `artifact` kind. The functions and trigger are
-- declared in the schema file and re-created at boot.

ALTER TABLE feedback_ingestion_outbox ADD COLUMN IF NOT EXISTS artifact_id TEXT;
ALTER TABLE feedback_ingestion_outbox DROP CONSTRAINT IF EXISTS feedback_ingestion_outbox_kind_check;
ALTER TABLE feedback_ingestion_outbox ADD CONSTRAINT feedback_ingestion_outbox_kind_check
    CHECK (kind IN ('invocation','request','assessment','resource_association','artifact'));
