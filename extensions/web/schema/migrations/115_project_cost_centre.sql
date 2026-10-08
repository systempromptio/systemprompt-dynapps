-- Projects carry a cost centre.
--
-- The `cost_centre` subject dimension reads it so core attributes each
-- gateway request to one and exports `systemprompt.scope.cost_centre` on its
-- OTLP span. Nullable and empty on every existing project: an operator sets
-- it from the project's settings tab. Adding a nullable column without a
-- default rewrites no rows.

ALTER TABLE projects ADD COLUMN IF NOT EXISTS cost_centre TEXT
    CHECK (cost_centre IS NULL OR length(cost_centre) BETWEEN 1 AND 64);
