-- A scheduled farm run is launched as an eval case, so its one-case eval run
-- records the trigger too: the eval runs check gains 'scheduler', as the runs
-- check did in 017. Widened in place; no rows are rewritten. Same DDL as
-- schema/52_agent_farm.sql.

ALTER TABLE agent_farm_eval_runs
    DROP CONSTRAINT IF EXISTS agent_farm_eval_runs_triggered_via_check;
ALTER TABLE agent_farm_eval_runs
    ADD CONSTRAINT agent_farm_eval_runs_triggered_via_check
        CHECK (triggered_via IN ('dashboard', 'api', 'eval', 'mcp', 'console', 'demo', 'scheduler'));
