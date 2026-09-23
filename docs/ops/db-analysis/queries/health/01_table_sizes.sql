-- Top tables by total size with toast/index split, dead tuples and vacuum state.
SELECT c.relname AS table_name,
       s.n_live_tup AS live_rows,
       s.n_dead_tup AS dead_rows,
       pg_size_pretty(pg_total_relation_size(c.oid)) AS total,
       pg_size_pretty(pg_relation_size(c.oid)) AS heap,
       pg_size_pretty(COALESCE(pg_total_relation_size(c.reltoastrelid), 0)) AS toast,
       pg_size_pretty(pg_indexes_size(c.oid)) AS indexes,
       s.last_autovacuum, s.last_analyze
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
LEFT JOIN pg_stat_user_tables s ON s.relid = c.oid
WHERE c.relkind = 'r' AND n.nspname IN ('public', 'marketplace')
ORDER BY pg_total_relation_size(c.oid) DESC
LIMIT 30;
