-- Every farm run can be evaluated when it is triggered, and an eval run
-- records the surface that started it (public API, MCP, console, demo) so
-- the eval history can be read per trigger. Widens the trigger check on
-- agent_farm_runs and adds the column, its check and the per-agent history
-- index to agent_farm_eval_runs. Existing eval runs were all started from
-- the console or the API; they take the column default. Same DDL as
-- schema/52_agent_farm.sql.

ALTER TABLE agent_farm_runs
    DROP CONSTRAINT IF EXISTS agent_farm_runs_triggered_via_check;
ALTER TABLE agent_farm_runs
    ADD CONSTRAINT agent_farm_runs_triggered_via_check
        CHECK (triggered_via IN ('dashboard', 'api', 'eval', 'mcp', 'console', 'demo'));

ALTER TABLE agent_farm_eval_runs
    ADD COLUMN IF NOT EXISTS triggered_via TEXT NOT NULL DEFAULT 'console';
ALTER TABLE agent_farm_eval_runs
    DROP CONSTRAINT IF EXISTS agent_farm_eval_runs_triggered_via_check;
ALTER TABLE agent_farm_eval_runs
    ADD CONSTRAINT agent_farm_eval_runs_triggered_via_check
        CHECK (triggered_via IN ('dashboard', 'api', 'eval', 'mcp', 'console', 'demo'));

CREATE INDEX IF NOT EXISTS idx_agent_farm_eval_runs_agent
    ON agent_farm_eval_runs(farm_id, agent_id, created_at DESC);
