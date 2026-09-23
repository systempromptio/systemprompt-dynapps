-- D28: scheduled_jobs; D29: anomalies and buckets
\echo == D28 scheduled jobs
SELECT job_name, schedule, enabled, last_status, last_run::timestamp(0), next_run::timestamp(0), run_count, left(last_error, 80) last_error FROM scheduled_jobs ORDER BY 1;
\echo == D29 usage anomalies and thresholds
SELECT metric, count(*), max(detected_at)::timestamp(0) FROM usage_anomalies GROUP BY 1;
SELECT * FROM anomaly_thresholds;
\echo == D29 quota / rate-limit buckets: age
SELECT subject_kind, window_seconds, count(*), min(window_start)::date oldest, max(window_start)::date newest, count(*) FILTER (WHERE window_start + make_interval(secs => window_seconds) < now() - interval '7 days') stale_7d FROM ai_quota_buckets GROUP BY 1,2 ORDER BY 3 DESC;
SELECT scope, count(*), min(window_start)::date oldest, count(*) FILTER (WHERE window_start < now() - interval '7 days') stale_7d FROM user_rate_limit_buckets GROUP BY 1 ORDER BY 2 DESC;
