-- Top statements since the run's pg_stat_statements_reset(): by total time,
-- then by calls. Tab-separated for report.py.
\pset format unaligned
\pset fieldsep '\t'
\pset footer off
\echo '## by_total_time'
SELECT round(total_exec_time::numeric, 1) AS total_ms, calls,
       round(mean_exec_time::numeric, 3) AS mean_ms, rows,
       regexp_replace(left(query, 160), '\s+', ' ', 'g') AS query
FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = 'systemprompt')
ORDER BY total_exec_time DESC LIMIT 10;
\echo '## by_calls'
SELECT calls, round(total_exec_time::numeric, 1) AS total_ms,
       round(mean_exec_time::numeric, 3) AS mean_ms,
       regexp_replace(left(query, 160), '\s+', ' ', 'g') AS query
FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = 'systemprompt')
ORDER BY calls DESC LIMIT 10;
