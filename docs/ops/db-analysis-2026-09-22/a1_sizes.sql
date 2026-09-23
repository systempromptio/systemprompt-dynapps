-- A1: table inventory, top 30 by total size (post-restore stats: dead tuples/vacuum are restore-side, prod values are lost in a plain dump)
SELECT n.nspname||'.'||c.relname AS tbl,
       c.reltuples::bigint AS est_rows,
       pg_size_pretty(pg_total_relation_size(c.oid)) AS total,
       pg_size_pretty(pg_relation_size(c.oid)) AS heap,
       pg_size_pretty(COALESCE(pg_relation_size(c.reltoastrelid),0)) AS toast,
       pg_size_pretty(pg_indexes_size(c.oid)) AS idx,
       pg_total_relation_size(c.oid) AS total_bytes
FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
WHERE c.relkind='r' AND n.nspname IN ('public','marketplace')
ORDER BY total_bytes DESC LIMIT 30;
