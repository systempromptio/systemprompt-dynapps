-- A routine names the template it was activated from. New nullable columns
-- only; no row is rewritten. Same columns as schema/01_dynwork_routines.sql;
-- the template tables (schema/02_dynwork_routine_templates.sql) and the
-- one-live-routine-per-template index are declarative and install around
-- this migration.
ALTER TABLE dynwork_routines ADD COLUMN IF NOT EXISTS template_id TEXT;
ALTER TABLE dynwork_routines ADD COLUMN IF NOT EXISTS template_revision BIGINT;
