-- Projects carry a practice.
--
-- The `practice` subject dimension reads it so core attributes each gateway
-- request to one and exports `systemprompt.scope.practice` on its OTLP span.
-- Nullable and empty on every existing project: an operator sets it from the
-- project's settings tab. Adding a nullable column without a default rewrites
-- no rows.

ALTER TABLE projects ADD COLUMN IF NOT EXISTS practice TEXT
    CHECK (practice IS NULL OR length(practice) BETWEEN 1 AND 64);
