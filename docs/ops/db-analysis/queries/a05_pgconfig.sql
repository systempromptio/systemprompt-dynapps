-- A5: Postgres config of the ANALYSIS container (prod config is not in a dump; recorded for the record only)
\echo == A5 note: these are the local analysis container's settings, not production's. The dump carries no server config.
SELECT name, setting, unit FROM pg_settings WHERE name IN ('server_version','shared_buffers','work_mem','maintenance_work_mem','max_connections','autovacuum','autovacuum_naptime','autovacuum_vacuum_scale_factor','autovacuum_analyze_scale_factor','effective_cache_size');
