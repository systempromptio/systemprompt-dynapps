-- Clients: who a farm run is for. A farm names its client in
-- agent-farms.yaml; every run, eval run and farm carries that client so runs
-- can be reported per client. An agent may name its own run-as service
-- account, preferred over the farm's. Nullable columns with no default: no
-- rows are rewritten. Same DDL as schema/52_agent_farm.sql.

CREATE TABLE IF NOT EXISTS clients (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    odoo_partner_id BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE agent_farms
    ADD COLUMN IF NOT EXISTS client_id TEXT REFERENCES clients(id) ON DELETE SET NULL;
ALTER TABLE agent_farm_runs
    ADD COLUMN IF NOT EXISTS client_id TEXT REFERENCES clients(id) ON DELETE SET NULL;
ALTER TABLE agent_farm_eval_runs
    ADD COLUMN IF NOT EXISTS client_id TEXT REFERENCES clients(id) ON DELETE SET NULL;
ALTER TABLE agent_farm_agents
    ADD COLUMN IF NOT EXISTS run_as_user_id TEXT REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_agent_farm_runs_client ON agent_farm_runs(client_id, created_at DESC);
