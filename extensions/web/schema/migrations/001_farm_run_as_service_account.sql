-- A farm may run as a service account (`run_as` in agent-farms.yaml) instead
-- of the person who launched the run. The farm records which account it runs
-- as; core's agent_runs row records the account each run's credentials were
-- minted for, and agent_farm_runs.user_id stays the launcher. A nullable
-- column with no default: no rows are rewritten. Same DDL as
-- schema/52_agent_farm.sql.

ALTER TABLE agent_farms
    ADD COLUMN IF NOT EXISTS run_as_user_id TEXT REFERENCES users(id) ON DELETE SET NULL;
