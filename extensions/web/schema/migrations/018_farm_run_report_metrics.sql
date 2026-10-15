-- The run report records what the nightly audit flow is judged on. The Odoo
-- writes column joined the retired odoo server's audit rows and is dropped;
-- in its place the observer stores the run's MCP calls per server split into
-- reads and writes (`mcp_calls`, with the totals beside it), the governance
-- `warn` findings its calls produced, and whether its summary reached
-- DynBrain. Trigger, start and end, turns, tokens and cost are read from the
-- run rows, not copied. Existing reports take the defaults. Same DDL as
-- schema/52_agent_farm.sql.

ALTER TABLE agent_farm_run_reports DROP COLUMN IF EXISTS odoo_writes;
ALTER TABLE agent_farm_run_reports
    ADD COLUMN IF NOT EXISTS mcp_calls JSONB NOT NULL DEFAULT '[]';
ALTER TABLE agent_farm_run_reports
    ADD COLUMN IF NOT EXISTS mcp_reads INTEGER NOT NULL DEFAULT 0;
ALTER TABLE agent_farm_run_reports
    ADD COLUMN IF NOT EXISTS mcp_writes INTEGER NOT NULL DEFAULT 0;
ALTER TABLE agent_farm_run_reports
    ADD COLUMN IF NOT EXISTS governance_warnings INTEGER NOT NULL DEFAULT 0;
ALTER TABLE agent_farm_run_reports
    ADD COLUMN IF NOT EXISTS delivery_status TEXT NOT NULL DEFAULT 'not_attempted';
ALTER TABLE agent_farm_run_reports
    DROP CONSTRAINT IF EXISTS agent_farm_run_reports_delivery_status_check;
ALTER TABLE agent_farm_run_reports
    ADD CONSTRAINT agent_farm_run_reports_delivery_status_check
        CHECK (delivery_status IN ('delivered', 'failed', 'not_attempted'));
