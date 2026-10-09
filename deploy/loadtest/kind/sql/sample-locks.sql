-- One sample of what every systemprompt backend is doing, one row per
-- (wait, relation) pair; run.sh appends it to waits.tsv every second.
SELECT now()::time(0) AS t,
       coalesce(a.wait_event_type, 'CPU') AS wait_type,
       coalesce(a.wait_event, a.state) AS wait_event,
       coalesce(c.relname, '') AS relation,
       count(*) AS n
FROM pg_stat_activity a
LEFT JOIN pg_locks l ON l.pid = a.pid AND NOT l.granted
LEFT JOIN pg_class c ON c.oid = l.relation
WHERE a.datname = 'systemprompt' AND a.pid <> pg_backend_pid() AND a.state <> 'idle'
GROUP BY 1, 2, 3, 4
ORDER BY n DESC;
