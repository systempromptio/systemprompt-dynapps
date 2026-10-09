-- Run once after migrations: confirm the stats extension and expose the
-- audit-guard state the chain on/off scenario flips.
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
SELECT extname, extversion FROM pg_extension WHERE extname = 'pg_stat_statements';
SELECT * FROM audit_guard_settings;
