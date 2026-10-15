-- One row per UTC day from $3 to today, zero-filled: runs created, how many
-- finished and succeeded, cost, and the mean eval score of that day's runs,
-- infrastructure failures left out of the mean and counted separately. $1
-- narrows to one client, $2 to one user's runs.
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
       AVG(s.score)::float8 AS "eval_mean?",
       COUNT(s.score)::bigint AS "eval_scored!",
       COUNT(r.id) FILTER (WHERE s.infra_failure)::bigint AS "infra_failures!"
FROM days d
LEFT JOIN agent_farm_runs r
       ON (r.created_at AT TIME ZONE 'UTC')::date = d.day
      AND ($1::text IS NULL OR r.client_id = $1)
      AND ($2::text IS NULL OR r.user_id = $2)
LEFT JOIN agent_farm_run_eval_scores s ON s.run_id = r.id
GROUP BY d.day
ORDER BY d.day
