-- An eval run records the replica driving it, so a replica that boots resumes
-- only eval runs whose driver is gone (no services heartbeat in 90 s), never
-- one another live replica is still driving. Same DDL as
-- schema/52_agent_farm.sql.

ALTER TABLE agent_farm_eval_runs ADD COLUMN IF NOT EXISTS driver_instance_id TEXT;
