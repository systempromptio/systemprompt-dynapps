-- One row per UTC day from $3 to today, zero-filled: runs created, how many
-- finished and succeeded, cost, and the mean eval score of the eval runs
-- that finished that day. $1 narrows to one client, $2 to one user's runs.
WITH days AS (
    SELECT generate_series(($3::timestamptz AT TIME ZONE 'UTC')::date,
                           (NOW() AT TIME ZONE 'UTC')::date,
                           INTERVAL '1 day')::date AS day
)
SELECT d.day AS "day!",
       COUNT(r.id)::bigint AS "runs!",
       COUNT(r.id) FILTER (WHERE r.status = 'succeeded')::bigint AS "succeeded!",
       COUNT(r.id) FILTER (WHERE r.status IN ('succeeded', 'failed', 'timed_out'))::bigint AS "finished!",
       COALESCE(SUM(r.cost_microdollars), 0)::bigint AS "cost_microdollars!",
       (SELECT AVG(e.mean_score)
          FROM agent_farm_eval_runs e
         WHERE ($1::text IS NULL OR e.client_id = $1)
           AND ($2::text IS NULL OR e.user_id = $2)
           AND e.mean_score IS NOT NULL
           AND (COALESCE(e.finished_at, e.created_at) AT TIME ZONE 'UTC')::date = d.day)::float8
           AS "eval_mean?"
FROM days d
LEFT JOIN agent_farm_runs r
       ON (r.created_at AT TIME ZONE 'UTC')::date = d.day
      AND ($1::text IS NULL OR r.client_id = $1)
      AND ($2::text IS NULL OR r.user_id = $2)
GROUP BY d.day
ORDER BY d.day
