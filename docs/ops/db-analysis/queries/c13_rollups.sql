-- C13: plugin_usage_daily vs plugin_usage_events; admin_usage_daily_rollups vs sources
\echo == C13 plugin_usage_daily vs events grouped by day/user/event_type/tool
WITH ev AS (SELECT created_at::date d, user_id, event_type, coalesce(tool_name,'') tool, count(*) n FROM plugin_usage_events GROUP BY 1,2,3,4),
     dy AS (SELECT date d, user_id, event_type, coalesce(tool_name,'') tool, sum(event_count) n FROM plugin_usage_daily GROUP BY 1,2,3,4)
SELECT count(*) AS groups, count(*) FILTER (WHERE ev.n IS DISTINCT FROM dy.n) AS mismatched, sum(ev.n) events, sum(dy.n) daily
FROM ev FULL JOIN dy USING (d, user_id, event_type, tool);
\echo == C13 plugin_usage_daily.plugin_id population
SELECT plugin_id IS NULL AS plugin_null, count(*), sum(event_count) FROM plugin_usage_daily GROUP BY 1;
\echo == C13 admin_usage_daily_rollups vs ai_requests by user/day (ai_requests_count, cost)
WITH r AS (SELECT user_id, created_at::date d, count(*) n, sum(cost_microdollars) c, sum(input_tokens) i, sum(output_tokens) o FROM ai_requests WHERE NOT synthetic GROUP BY 1,2)
SELECT count(*) AS rollup_rows, count(r.n) AS with_requests,
       count(*) FILTER (WHERE a.ai_requests_count IS DISTINCT FROM coalesce(r.n,0)) AS count_mismatch,
       count(*) FILTER (WHERE a.cost_microdollars IS DISTINCT FROM coalesce(r.c,0)) AS cost_mismatch,
       sum(a.ai_requests_count) rollup_requests, sum(r.n) actual_requests,
       (SELECT count(*) FROM r WHERE NOT EXISTS (SELECT 1 FROM admin_usage_daily_rollups a2 WHERE a2.user_id=r.user_id AND a2.date=r.d)) AS user_days_missing_from_rollups
FROM admin_usage_daily_rollups a LEFT JOIN r ON r.user_id = a.user_id AND r.d = a.date;
\echo == C13 rollups: date range and probe inclusion
SELECT min(date), max(date), count(*), sum(ai_requests_count) FROM admin_usage_daily_rollups;
SELECT (SELECT count(*) FROM ai_requests WHERE request_kind='probe' AND NOT synthetic) AS probes_not_synthetic,
       (SELECT count(*) FROM ai_requests WHERE actor_kind='job') AS job_actor,
       (SELECT count(*) FROM ai_requests WHERE context_id LIKE '%4c4547414359') AS sentinel_context;
\echo == C13 rollups tool_uses (hook plane) vs PostToolUse events, per day
WITH h AS (SELECT created_at::date d, count(*) n FROM plugin_usage_events WHERE event_type IN ('PostToolUse','PostToolUseFailure') GROUP BY 1)
SELECT a.date, sum(a.tool_uses) rollup_tool_uses, h.n hook_events, sum(a.sessions_count) rollup_sessions, sum(a.prompts) prompts FROM admin_usage_daily_rollups a LEFT JOIN h ON h.d=a.date GROUP BY 1, h.n ORDER BY 1 DESC LIMIT 8;
