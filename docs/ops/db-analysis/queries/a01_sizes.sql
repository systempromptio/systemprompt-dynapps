-- A1: every table with rows, sizes, dead tuples, vacuum/analyze; top 30 by total size
\echo == A1 top 30 by size
SELECT s.relname, n_live_tup AS rows, n_dead_tup AS dead,
       pg_size_pretty(pg_total_relation_size(relid)) AS total,
       pg_size_pretty(pg_relation_size(relid)) AS heap,
       pg_size_pretty(pg_indexes_size(relid)) AS idx,
       pg_size_pretty(coalesce(pg_total_relation_size(reltoastrelid),0)) AS toast,
       last_autovacuum::timestamp(0), last_analyze::timestamp(0), last_autoanalyze::timestamp(0)
FROM pg_stat_user_tables s JOIN pg_class c ON c.oid = s.relid
ORDER BY pg_total_relation_size(relid) DESC LIMIT 30;
\echo == A1 table count and total size
SELECT count(*) AS tables, pg_size_pretty(sum(pg_total_relation_size(relid))) AS total FROM pg_stat_user_tables;
\echo == A1 empty tables
SELECT count(*) FILTER (WHERE n_live_tup = 0) AS empty_tables FROM pg_stat_user_tables;
