-- Run identity becomes a mode (`run_as: {mode: user|client|service_account}`
-- in agent-farms.yaml), on a farm and per agent. NULL is the earlier
-- behaviour: the run_as_user_id account when one is set, else the launcher.
-- A client may name the service account `client` mode runs act as. Nullable
-- columns with no default: no rows are rewritten. Same DDL as
-- schema/52_agent_farm.sql.

ALTER TABLE agent_farms
    ADD COLUMN IF NOT EXISTS run_as_mode TEXT
        CHECK (run_as_mode IN ('user', 'client', 'service_account'));
ALTER TABLE agent_farm_agents
    ADD COLUMN IF NOT EXISTS run_as_mode TEXT
        CHECK (run_as_mode IN ('user', 'client', 'service_account'));
ALTER TABLE clients
    ADD COLUMN IF NOT EXISTS service_account_user_id TEXT REFERENCES users(id) ON DELETE SET NULL;
