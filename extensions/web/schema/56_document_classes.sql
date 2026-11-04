-- The document class set on a folder or document: how far an agent may act
-- on it (services/documents/classes.yaml declares the classes). A target
-- with no row inherits from its nearest classed ancestor folder, then the
-- creator default. `reason` is required when an admin loosens a class.
-- No foreign key into core's document tables: this extension's schema may
-- install before core's documents module, and a row for a deleted target is
-- inert.
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
