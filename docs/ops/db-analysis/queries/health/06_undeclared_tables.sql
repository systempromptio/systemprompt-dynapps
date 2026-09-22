-- Tables that exist but are declared by no registered extension (fill :declared from the schema files).
-- Usage: psql -v declared="'a','b'" -f 06_undeclared_tables.sql
SELECT n.nspname, c.relname, s.n_live_tup
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
LEFT JOIN pg_stat_user_tables s ON s.relid = c.oid
WHERE c.relkind = 'r' AND n.nspname IN ('public','marketplace')
  AND c.relname <> ALL (string_to_array(:'declared', ','))
ORDER BY 1, 2;
