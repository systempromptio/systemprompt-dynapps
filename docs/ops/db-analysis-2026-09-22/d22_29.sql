-- D22 ai_requests temporal
SELECT count(*) FILTER (WHERE completed_at < created_at) completed_before_created,
       count(*) FILTER (WHERE status='completed' AND latency_ms IS NULL) completed_latency_null,
       count(*) FILTER (WHERE status IN ('pending','streaming') AND created_at < now()-interval '1 hour') stuck,
       count(*) FILTER (WHERE created_at > now()) future,
       count(*) FILTER (WHERE status='completed' AND completed_at IS NULL) completed_no_completed_at,
       count(*) FILTER (WHERE status='completed' AND finish_reason IS NULL) completed_no_finish_reason,
       count(*) FILTER (WHERE latency_ms > 600000) latency_over_10m, max(latency_ms) max_latency_ms,
       count(*) FILTER (WHERE upstream_latency_ms IS NULL AND status='completed') no_upstream_latency,
       count(*) FILTER (WHERE upstream_latency_ms > latency_ms) upstream_gt_total,
       count(*) FILTER (WHERE accounting_failed_at IS NOT NULL) accounting_failed
FROM ai_requests;
SELECT count(*), count(DISTINCT request_id), count(*) - count(DISTINCT request_id) dup_request_ids FROM ai_requests;
SELECT id, status, created_at, completed_at, latency_ms FROM ai_requests WHERE completed_at < created_at LIMIT 3;
SELECT id, status, created_at, error_message IS NOT NULL has_error FROM ai_requests WHERE status IN ('pending','streaming') ORDER BY created_at LIMIT 5;
SELECT created_at::date d, status, finish_reason, count(*) FROM ai_requests GROUP BY 1,2,3 ORDER BY 1,2,3;
SELECT left(regexp_replace(regexp_replace(error_message, '[0-9a-f]{8}-[0-9a-f-]{27}', '<uuid>', 'g'), '\d+', '<n>', 'g'), 110) shape, count(*) FROM ai_requests WHERE error_message IS NOT NULL GROUP BY 1 ORDER BY 2 DESC LIMIT 20;
-- D23 sessions
SELECT count(*) FILTER (WHERE ended_at IS NULL AND last_activity_at < now()-interval '7 days') open_but_idle_7d,
       count(*) FILTER (WHERE expires_at < now() AND ended_at IS NULL) expired_not_ended,
       count(*) FILTER (WHERE ended_at IS NULL) open_total, count(*) total,
       count(*) FILTER (WHERE is_bot) bots, count(*) FILTER (WHERE is_ai_crawler) ai_crawlers, count(*) FILTER (WHERE is_scanner) scanners, count(*) FILTER (WHERE is_behavioral_bot) behavioral_bots,
       count(*) FILTER (WHERE revoked_at IS NOT NULL) revoked, count(*) FILTER (WHERE duration_seconds IS NULL AND ended_at IS NOT NULL) ended_no_duration
FROM user_sessions;
SELECT user_id, started_at::date, count(*) FROM user_sessions GROUP BY 1,2 HAVING count(*)>50 ORDER BY 3 DESC LIMIT 5;
SELECT session_source, count(*), count(*) FILTER (WHERE ended_at IS NULL) open, round(avg(EXTRACT(EPOCH FROM (COALESCE(ended_at,last_activity_at)-started_at)))) avg_secs FROM user_sessions GROUP BY 1;
-- D24 mcp executions
SELECT count(*) FILTER (WHERE completed_at IS NULL AND created_at < now()-interval '1 hour') incomplete_old, status, count(*), percentile_cont(0.5) WITHIN GROUP (ORDER BY execution_time_ms) p50_ms, percentile_cont(0.99) WITHIN GROUP (ORDER BY execution_time_ms) p99_ms, max(execution_time_ms) max_ms FROM mcp_tool_executions GROUP BY status;
SELECT server_name, count(*) FROM mcp_tool_executions GROUP BY 1 ORDER BY 2 DESC;
-- D25 tasks
SELECT (SELECT count(*) FROM agent_tasks) tasks, (SELECT count(*) FROM task_execution_steps) steps, (SELECT count(*) FROM task_execution_steps s WHERE NOT EXISTS (SELECT 1 FROM agent_tasks t WHERE t.task_id=s.task_id)) steps_without_task;
-- D26 logs
SELECT level, count(*) FROM logs GROUP BY 1 ORDER BY 2 DESC;
SELECT timestamp::date d, count(*) FILTER (WHERE level='ERROR') err, count(*) FILTER (WHERE level='WARN') warn, count(*) FILTER (WHERE level='INFO') info, count(*) FILTER (WHERE level='DEBUG') debug, count(*) total FROM logs GROUP BY 1 ORDER BY 1;
SELECT module, count(*) FROM logs GROUP BY 1 ORDER BY 2 DESC LIMIT 20;
SELECT left(regexp_replace(regexp_replace(regexp_replace(message, '[0-9a-f]{8}-[0-9a-f-]{27}', '<uuid>', 'g'), '\d+', '<n>', 'g'), '''[^'']*''', '<q>', 'g'), 120) shape, module, count(*) FROM logs WHERE level='ERROR' GROUP BY 1,2 ORDER BY 3 DESC LIMIT 25;
SELECT left(regexp_replace(regexp_replace(message, '[0-9a-f]{8}-[0-9a-f-]{27}', '<uuid>', 'g'), '\d+', '<n>', 'g'), 120) shape, module, count(*) FROM logs WHERE level='WARN' GROUP BY 1,2 ORDER BY 3 DESC LIMIT 12;
SELECT left(regexp_replace(regexp_replace(message, '[0-9a-f]{8}-[0-9a-f-]{27}', '<uuid>', 'g'), '\d+', '<n>', 'g'), 100) shape, module, count(*) FROM logs WHERE level='INFO' GROUP BY 1,2 ORDER BY 3 DESC LIMIT 12;
SELECT count(*) FILTER (WHERE user_id IN (SELECT id FROM users WHERE status='deleted')) logs_of_deleted_users FROM logs;
-- D28 scheduled jobs
\d scheduled_jobs
SELECT job_name, enabled, status, last_run_at, last_duration_ms, run_count, failure_count, next_run_at, left(last_error,80) last_error FROM scheduled_jobs ORDER BY job_name;
-- D29 anomalies & buckets
SELECT anomaly_type, severity, count(*), min(detected_at), max(detected_at) FROM usage_anomalies GROUP BY 1,2 ORDER BY 3 DESC;
SELECT count(*) FROM anomaly_thresholds;
\d ai_quota_buckets
SELECT count(*), min(window_start), max(window_start), count(*) FILTER (WHERE window_end < now()-interval '7 days') stale_7d FROM ai_quota_buckets;
\d user_rate_limit_buckets
SELECT count(*), min(window_start), max(window_start), count(*) FILTER (WHERE window_start < now()-interval '1 day') older_1d FROM user_rate_limit_buckets;
