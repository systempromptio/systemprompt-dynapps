-- Farms agent-farms.yaml no longer declares, so their agents stop being
-- listed and nothing launches them. Their agents, fixtures, suites and run
-- rows go with them (ON DELETE CASCADE); core's agent_runs rows stay. A farm
-- with a run still in flight is kept until the next seed.
DELETE FROM agent_farms f
WHERE f.id <> ALL($1::text[])
  AND NOT EXISTS (
      SELECT 1 FROM agent_farm_runs r
      WHERE r.farm_id = f.id AND r.status IN ('queued', 'preparing', 'running'))
RETURNING f.id
