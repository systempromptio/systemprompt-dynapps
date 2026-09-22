-- A3: unused indexes (idx_scan = 0) larger than 1 MB (10 MB in the brief; the DB is small), duplicate indexes
\echo == A3 unused indexes > 1 MB (stats are from the dump source only if pg_stat survived; on a restore idx_scan is 0 for all, so this lists size only)
SELECT s.relname AS table, s.indexrelname AS index, pg_size_pretty(pg_relation_size(s.indexrelid)) AS size, s.idx_scan
FROM pg_stat_user_indexes s JOIN pg_index i ON i.indexrelid = s.indexrelid
WHERE NOT i.indisunique AND pg_relation_size(s.indexrelid) > 1024*1024
ORDER BY pg_relation_size(s.indexrelid) DESC LIMIT 25;
\echo == A3 duplicate indexes (identical column lists on the same table)
SELECT a.indrelid::regclass AS table, a.indexrelid::regclass AS idx1, b.indexrelid::regclass AS idx2
FROM pg_index a JOIN pg_index b ON a.indrelid = b.indrelid AND a.indexrelid < b.indexrelid
 AND a.indkey::text = b.indkey::text AND coalesce(pg_get_expr(a.indpred, a.indrelid),'') = coalesce(pg_get_expr(b.indpred, b.indrelid),'')
JOIN pg_class c ON c.oid = a.indrelid JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname IN ('public','marketplace');
\echo == A3 overlapping indexes (one index's columns are a prefix of another's, same table)
SELECT a.indrelid::regclass AS table, a.indexrelid::regclass AS prefix_idx, b.indexrelid::regclass AS wider_idx
FROM pg_index a JOIN pg_index b ON a.indrelid = b.indrelid AND a.indexrelid <> b.indexrelid
 AND array_length(a.indkey,1) < array_length(b.indkey,1)
 AND b.indkey::text LIKE a.indkey::text || ' %'
 AND a.indpred IS NULL AND b.indpred IS NULL
JOIN pg_class c ON c.oid = a.indrelid JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname IN ('public','marketplace')
ORDER BY 1;
\echo == A3 dead tuple ratio > 10% on tables > 1000 rows
SELECT relname, n_live_tup, n_dead_tup, round(100.0*n_dead_tup/greatest(n_live_tup,1),1) AS dead_pct FROM pg_stat_user_tables WHERE n_live_tup > 1000 AND n_dead_tup > 0.1*n_live_tup ORDER BY 4 DESC;
