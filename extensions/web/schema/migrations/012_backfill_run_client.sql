-- Every run, eval run and run report carries a client. Runs launched before
-- clients existed take their farm's client. An eval run takes the client of
-- the runs its results launched when they agree on one (a triggered eval of
-- an Acme run is Acme's, whatever the farm's default), else its farm's; this
-- also corrects triggered eval runs stamped with the farm's client. Reports
-- follow their run. Idempotent: every statement only touches rows that differ.
-- The eval-score view is the same DDL as schema/52_agent_farm.sql.

UPDATE agent_farm_runs r
   SET client_id = f.client_id
  FROM agent_farms f
 WHERE f.id = r.farm_id
   AND r.client_id IS NULL
   AND f.client_id IS NOT NULL;

WITH resolved AS (
    SELECT e.id,
           COALESCE((SELECT MIN(r.client_id)
                       FROM agent_farm_eval_results x
                       JOIN agent_farm_runs r ON r.id = x.run_id
                      WHERE x.eval_run_id = e.id AND r.client_id IS NOT NULL
                     HAVING COUNT(DISTINCT r.client_id) = 1),
                    f.client_id) AS client_id
    FROM agent_farm_eval_runs e
    JOIN agent_farms f ON f.id = e.farm_id
)
UPDATE agent_farm_eval_runs e
   SET client_id = resolved.client_id
  FROM resolved
 WHERE resolved.id = e.id
   AND resolved.client_id IS NOT NULL
   AND e.client_id IS DISTINCT FROM resolved.client_id;

UPDATE agent_farm_run_reports rr
   SET client_id = r.client_id
  FROM agent_farm_runs r
 WHERE r.id = rr.run_id
   AND rr.client_id IS DISTINCT FROM r.client_id;

CREATE OR REPLACE VIEW agent_farm_run_eval_scores AS
SELECT r.id AS run_id,
       COALESCE(r.error_class IN ('host_crashed', 'connector_unavailable')
                OR x.status = 'errored', false) AS infra_failure,
       CASE WHEN x.status IN ('passed', 'failed')
             AND COALESCE(r.error_class IN ('host_crashed', 'connector_unavailable'), false) = false
            THEN COALESCE(x.judge_score, CASE WHEN x.status = 'passed' THEN 100.0 ELSE 0.0 END)
       END AS score
FROM agent_farm_runs r
LEFT JOIN LATERAL (
    SELECT e.status, e.judge_score
    FROM agent_farm_eval_results e
    WHERE e.run_id = r.id
    ORDER BY e.updated_at DESC
    LIMIT 1
) x ON true;
