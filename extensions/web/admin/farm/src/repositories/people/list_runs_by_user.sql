-- The "By person" leaderboard over a window; agent_farm_runs_by_user is the
-- same shape over all time.
SELECT r.user_id AS "user_id!: UserId",
       COALESCE(NULLIF(u.display_name, ''), u.email, r.user_id) AS "display_name?",
       COUNT(*)::bigint AS "runs?",
       COUNT(*) FILTER (WHERE r.status = 'succeeded')::bigint AS "succeeded?",
       COUNT(*) FILTER (WHERE r.status IN ('failed', 'timed_out'))::bigint AS "failed?",
       COUNT(*) FILTER (WHERE r.status IN ('queued', 'preparing', 'running'))::bigint AS "active?",
       COUNT(*) FILTER (WHERE r.triggered_via IN ('dashboard', 'console', 'demo'))::bigint AS "dashboard_runs?",
       COUNT(*) FILTER (WHERE r.triggered_via IN ('api', 'mcp'))::bigint AS "api_runs?",
       COUNT(*) FILTER (WHERE r.triggered_via = 'eval')::bigint AS "eval_runs?",
       COALESCE(SUM(r.cost_microdollars), 0)::bigint AS "cost_microdollars?",
       MODE() WITHIN GROUP (ORDER BY r.agent_id) AS "favourite_agent?",
       MAX(r.created_at) AS "last_run_at?"
FROM agent_farm_runs r
LEFT JOIN users u ON u.id = r.user_id
WHERE r.created_at >= $1
  AND ($2::text IS NULL OR r.farm_id = $2)
GROUP BY r.user_id, u.display_name, u.email
ORDER BY COUNT(*) DESC, MAX(r.created_at) DESC, r.user_id
