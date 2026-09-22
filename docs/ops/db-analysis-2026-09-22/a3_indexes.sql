-- A3a: duplicate / overlapping indexes (same table, one's column list is a prefix of another's)
WITH ix AS (
  SELECT i.indrelid::regclass tbl, i.indexrelid::regclass idx, i.indkey::text cols, i.indisunique uniq, i.indpred IS NOT NULL partial,
         pg_relation_size(i.indexrelid) bytes
  FROM pg_index i JOIN pg_class c ON c.oid=i.indexrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname IN ('public','marketplace'))
SELECT a.tbl, a.idx AS narrower, a.cols AS narrower_cols, b.idx AS wider, b.cols AS wider_cols, pg_size_pretty(a.bytes) narrower_size
FROM ix a JOIN ix b ON a.tbl=b.tbl AND a.idx<>b.idx AND (b.cols=a.cols OR b.cols LIKE a.cols||' %') AND NOT a.uniq AND NOT a.partial AND NOT b.partial
ORDER BY a.tbl::text, a.idx::text;
-- A3b: index count and size per large table
SELECT t.relname, count(i.indexrelid) n_idx, pg_size_pretty(pg_indexes_size(t.oid)) idx_size, pg_size_pretty(pg_relation_size(t.oid)) heap
FROM pg_class t JOIN pg_namespace n ON n.oid=t.relnamespace LEFT JOIN pg_index i ON i.indrelid=t.oid
WHERE n.nspname='public' AND t.relkind='r' GROUP BY t.oid,t.relname HAVING pg_relation_size(t.oid)>1000000 ORDER BY pg_indexes_size(t.oid) DESC;
-- A3c: bloat estimate via pgstattuple if available
SELECT extname FROM pg_extension WHERE extname='pgstattuple';
SELECT name FROM pg_available_extensions WHERE name='pgstattuple';
