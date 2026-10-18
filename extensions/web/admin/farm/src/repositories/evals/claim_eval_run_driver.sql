UPDATE agent_farm_eval_runs r
SET driver_instance_id = $2
WHERE r.id = $1
  AND r.status IN ('queued', 'running')
  AND (
      r.driver_instance_id IS NULL
      OR r.driver_instance_id = $2
      OR NOT EXISTS (
          SELECT 1 FROM services s
          WHERE s.instance_id = r.driver_instance_id
            AND s.heartbeat_at > NOW() - INTERVAL '90 seconds'
      )
  )
