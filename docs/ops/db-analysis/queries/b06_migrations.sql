-- B6: extension_migrations ledger. Cross-checked offline (scripts in report §B6) against every
-- schema/migrations/NNN_*.sql at v0.58.0 in both repos using Migration::checksum (xxh64 of the SQL).
\echo == B6 rows per extension
SELECT extension_id, count(*) AS applied, min(version) AS first, max(version) AS last, max(applied_at)::timestamp(0) AS last_applied FROM extension_migrations GROUP BY 1 ORDER BY 1;
\echo == B6 rows for extensions that no longer exist in either repo
SELECT extension_id, version, name, applied_at::timestamp(0) FROM extension_migrations WHERE extension_id IN ('evaluation') ORDER BY 2;
\echo == B6 when was the last migration batch applied (upgrade timestamps)
SELECT applied_at::timestamp(0) AS batch, count(*) FROM extension_migrations GROUP BY 1 ORDER BY 1 DESC LIMIT 8;
