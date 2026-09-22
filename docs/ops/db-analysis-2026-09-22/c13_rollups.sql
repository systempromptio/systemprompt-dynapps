-- C13a: plugin_usage_daily vs plugin_usage_events grouped by (date, user, plugin, event_type, tool_name)
WITH ev AS (SELECT created_at::date d, user_id, plugin_id, event_type, tool_name, count(*) n, count(*) FILTER (WHERE (metadata->>'is_error')::bool OR metadata->>'error' IS NOT NULL) errs FROM plugin_usage_events GROUP BY 1,2,3,4,5),
     both_ AS (SELECT COALESCE(e.d,r.date) d, COALESCE(e.user_id,r.user_id) u, COALESCE(e.plugin_id,r.plugin_id) p, COALESCE(e.event_type,r.event_type) et, COALESCE(e.tool_name,r.tool_name) t, e.n ev_n, r.event_count roll_n
                FROM ev e FULL JOIN plugin_usage_daily r ON r.date=e.d AND r.user_id=e.user_id AND r.plugin_id IS NOT DISTINCT FROM e.plugin_id AND r.event_type=e.event_type AND r.tool_name IS NOT DISTINCT FROM e.tool_name)
SELECT count(*) keys, count(*) FILTER (WHERE ev_n IS NULL) rollup_only, count(*) FILTER (WHERE roll_n IS NULL) events_only, count(*) FILTER (WHERE ev_n<>roll_n) count_differs,
       sum(ev_n) events_total, sum(roll_n) rollup_total FROM both_;
SELECT d, u, p, et, t, ev_n, roll_n FROM (SELECT COALESCE(e.d,r.date) d, COALESCE(e.user_id,r.user_id) u, COALESCE(e.plugin_id,r.plugin_id) p, COALESCE(e.event_type,r.event_type) et, COALESCE(e.tool_name,r.tool_name) t, e.n ev_n, r.event_count roll_n
  FROM (SELECT created_at::date d, user_id, plugin_id, event_type, tool_name, count(*) n FROM plugin_usage_events GROUP BY 1,2,3,4,5) e FULL JOIN plugin_usage_daily r ON r.date=e.d AND r.user_id=e.user_id AND r.plugin_id IS NOT DISTINCT FROM e.plugin_id AND r.event_type=e.event_type AND r.tool_name IS NOT DISTINCT FROM e.tool_name) x
WHERE ev_n IS DISTINCT FROM roll_n ORDER BY d LIMIT 8;
-- by day totals
SELECT d, sum(ev) events, sum(roll) rollup FROM (SELECT created_at::date d, count(*) ev, 0 roll FROM plugin_usage_events GROUP BY 1 UNION ALL SELECT date, 0, sum(event_count) FROM plugin_usage_daily GROUP BY 1) x GROUP BY 1 ORDER BY 1;
-- C13b: admin_usage_daily_rollups vs ai_requests by (user, day)
WITH src AS (SELECT user_id, created_at::date d, count(*) n, sum(input_tokens) i, sum(output_tokens) o, sum(cost_microdollars) c FROM ai_requests GROUP BY 1,2),
     j AS (SELECT COALESCE(s.user_id,r.user_id) u, COALESCE(s.d,r.date) d, s.n src_n, r.ai_requests_count roll_n, s.c src_c, r.cost_microdollars roll_c, s.i src_i, r.input_tokens roll_i FROM src s FULL JOIN admin_usage_daily_rollups r ON r.user_id=s.user_id AND r.date=s.d)
SELECT count(*) keys, count(*) FILTER (WHERE roll_n IS NULL) src_only, count(*) FILTER (WHERE src_n IS NULL) roll_only, count(*) FILTER (WHERE src_n<>roll_n) n_differs, count(*) FILTER (WHERE src_c<>roll_c) cost_differs,
       sum(src_n) src_requests, sum(roll_n) roll_requests, round(sum(src_c)/1e6,2) src_usd, round(sum(roll_c)/1e6,2) roll_usd FROM j;
SELECT u, d, src_n, roll_n, src_c, roll_c FROM (SELECT COALESCE(s.user_id,r.user_id) u, COALESCE(s.d,r.date) d, s.n src_n, r.ai_requests_count roll_n, s.c src_c, r.cost_microdollars roll_c FROM (SELECT user_id, created_at::date d, count(*) n, sum(cost_microdollars) c FROM ai_requests GROUP BY 1,2) s FULL JOIN admin_usage_daily_rollups r ON r.user_id=s.user_id AND r.date=s.d) j WHERE src_n IS DISTINCT FROM roll_n OR src_c IS DISTINCT FROM roll_c ORDER BY d DESC LIMIT 8;
SELECT min(date), max(date), count(*), max(updated_at) FROM admin_usage_daily_rollups;
