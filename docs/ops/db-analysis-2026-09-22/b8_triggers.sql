-- B8: every non-internal trigger, with the row count of its table
SELECT c.relname tbl, t.tgname, 
       CASE WHEN t.tgtype & 2 = 2 THEN 'BEFORE' WHEN t.tgtype & 64 = 64 THEN 'INSTEAD' ELSE 'AFTER' END timing,
       CASE WHEN t.tgtype & 1 = 1 THEN 'ROW' ELSE 'STMT' END level,
       concat_ws('|', CASE WHEN t.tgtype & 4 = 4 THEN 'INS' END, CASE WHEN t.tgtype & 8 = 8 THEN 'DEL' END, CASE WHEN t.tgtype & 16 = 16 THEN 'UPD' END) events,
       p.proname fn, t.tgenabled enabled, c.reltuples::bigint est_rows
FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_proc p ON p.oid=t.tgfoid
WHERE NOT t.tgisinternal ORDER BY c.reltuples DESC, c.relname, t.tgname;
