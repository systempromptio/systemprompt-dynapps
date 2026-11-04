-- @cost: rows=0 measured=0.01s triggers=live
-- Document classes per folder and document. New table only; no rows
-- rewritten. Same DDL as schema/56_document_classes.sql.
CREATE TABLE IF NOT EXISTS document_classes (
    target_kind TEXT NOT NULL CHECK (target_kind IN ('folder', 'document')),
    target_id UUID NOT NULL,
    class_id TEXT NOT NULL CHECK (length(class_id) > 0),
    set_by VARCHAR(255) NOT NULL,
    reason TEXT,
    loosened BOOLEAN NOT NULL DEFAULT FALSE,
    set_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (target_kind, target_id),
    CHECK (NOT loosened OR (reason IS NOT NULL AND length(reason) > 0))
);
