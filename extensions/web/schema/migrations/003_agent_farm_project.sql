-- A farm may belong to a project, so a run's observed facts can be scoped
-- the way every other admin read is (`Scope::Projects`). Core's agent_runs
-- row carries no project; the farm linked to the run does. A nullable
-- column with no default: no rows are rewritten. Same DDL as
-- schema/52_agent_farm.sql.

ALTER TABLE agent_farms
    ADD COLUMN IF NOT EXISTS project_id TEXT;
