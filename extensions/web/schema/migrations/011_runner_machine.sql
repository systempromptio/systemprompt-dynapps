-- A farm run executed on its own Fly Machine (`runner.mode: fly_machine`)
-- records which machine and region ran it, so run detail can prove the run
-- did not execute on the API machine. Both stay NULL for a local run. Same
-- DDL as schema/52_agent_farm.sql.

ALTER TABLE agent_farm_runs ADD COLUMN IF NOT EXISTS runner_machine_id TEXT;
ALTER TABLE agent_farm_runs ADD COLUMN IF NOT EXISTS runner_region TEXT;
