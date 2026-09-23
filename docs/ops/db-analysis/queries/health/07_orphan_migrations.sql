-- extension_migrations rows whose extension id is not in the registered set.
-- Usage: psql -v extensions="ai,agent,..." -f 07_orphan_migrations.sql
SELECT extension, count(*) AS rows, min(applied_at) AS first, max(applied_at) AS last
FROM extension_migrations
WHERE extension <> ALL (string_to_array(:'extensions', ','))
GROUP BY 1 ORDER BY 1;
