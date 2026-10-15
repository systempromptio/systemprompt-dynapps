-- One row per client: runs, outcomes, people, cost, completion reports and
-- the mean eval score over the window. $1 narrows to one client, $2 to the
-- runs one user launched. Clients with no run in scope still list.
SELECT c.id AS "client_id!: ClientId",
       c.name AS "name!",
       c.slug AS "slug!",
       c.odoo_partner_id AS "odoo_partner_id?",
       COUNT(r.id)::bigint AS "runs!",
       COUNT(r.id) FILTER (WHERE r.status = 'succeeded')::bigint AS "succeeded!",
       COUNT(r.id) FILTER (WHERE r.status IN ('succeeded', 'failed', 'timed_out'))::bigint AS "finished!",
       COUNT(r.id) FILTER (WHERE r.status IN ('queued', 'preparing', 'running'))::bigint AS "active!",
       COUNT(DISTINCT r.user_id)::bigint AS "people!",
       COALESCE(SUM(r.cost_microdollars), 0)::bigint AS "cost_microdollars!",
       COUNT(rr.run_id) FILTER (WHERE rr.completed)::bigint AS "completed!",
       COUNT(rr.run_id)::bigint AS "reported!",
       (SELECT AVG(e.mean_score)
          FROM agent_farm_eval_runs e
         WHERE e.client_id = c.id
           AND ($2::text IS NULL OR e.user_id = $2)
           AND e.mean_score IS NOT NULL
           AND e.created_at >= $3)::float8 AS "eval_mean?",
       MAX(r.created_at) AS "last_run_at?"
FROM clients c
LEFT JOIN agent_farm_runs r
       ON r.client_id = c.id
      AND r.created_at >= $3
      AND ($2::text IS NULL OR r.user_id = $2)
LEFT JOIN agent_farm_run_reports rr ON rr.run_id = r.id
WHERE ($1::text IS NULL OR c.id = $1)
GROUP BY c.id, c.name, c.slug, c.odoo_partner_id
ORDER BY COUNT(r.id) DESC, c.name
