-- The farm is cut down to the one nightly audit flow. A run the scheduler
-- queues records `triggered_via = 'scheduler'`, so the run and its report say
-- the scheduler started it rather than a person; the runs check is widened
-- in place and no rows are rewritten. Clients no longer carry an Odoo
-- partner id: nothing reads it since the custom Odoo server was retired.
-- Same DDL as schema/52_agent_farm.sql.

ALTER TABLE agent_farm_runs
    DROP CONSTRAINT IF EXISTS agent_farm_runs_triggered_via_check;
ALTER TABLE agent_farm_runs
    ADD CONSTRAINT agent_farm_runs_triggered_via_check
        CHECK (triggered_via IN ('dashboard', 'api', 'eval', 'mcp', 'console', 'demo', 'scheduler'));

ALTER TABLE clients DROP COLUMN IF EXISTS odoo_partner_id;
