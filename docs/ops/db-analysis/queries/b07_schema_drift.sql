-- B7: live schema vs the recorded 0.58.0 fresh-install baseline (tests/fixtures/schema/release-baseline-0.58.0.sql).
-- The textual diff was produced offline (report §B7); these queries reproduce the concrete differences found.
\echo == B7 tables present in prod that a fresh 0.58.0 install does not create (deleted crates, no drop migration)
SELECT split_part(tablename,'_',1) AS family, count(*) AS tables, pg_size_pretty(sum(pg_total_relation_size(('public.'||tablename)::regclass))) AS size,
       sum((xpath('/row/c/text()', query_to_xml('select count(*) as c from public.'||tablename, false, true, '')))[1]::text::int) AS rows
FROM pg_tables WHERE schemaname='public' AND (tablename LIKE 'eval\_%' OR tablename LIKE 'knowledge\_%' OR tablename LIKE 'requirements\_%') GROUP BY 1 ORDER BY 1;
\echo == B7 unique constraints declared by the 0.58.0 schema files but absent from prod
SELECT 'mcp_tool_executions_user_id_mcp_execution_id_key' AS expected, EXISTS (SELECT 1 FROM pg_constraint WHERE conname='mcp_tool_executions_user_id_mcp_execution_id_key') AS present
UNION ALL SELECT 'plugin_usage_events_user_id_id_key', EXISTS (SELECT 1 FROM pg_constraint WHERE conname='plugin_usage_events_user_id_id_key');
\echo == B7 objects in prod not in the baseline
SELECT 'event_outbox_actor_id_nonempty (CHECK)' AS obj, EXISTS (SELECT 1 FROM pg_constraint WHERE conname='event_outbox_actor_id_nonempty') AS present
UNION ALL SELECT 'idx_ai_safety_findings_blocked (INDEX)', EXISTS (SELECT 1 FROM pg_indexes WHERE indexname='idx_ai_safety_findings_blocked');
\echo == B7 object counts (baseline: 200 tables / 32 views / 74 functions / 35 triggers / 493 indexes)
SELECT (SELECT count(*) FROM pg_tables WHERE schemaname IN ('public','marketplace')) tables,
       (SELECT count(*) FROM pg_views WHERE schemaname IN ('public','marketplace')) views,
       (SELECT count(*) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname IN ('public','marketplace')) functions,
       (SELECT count(*) FROM pg_trigger WHERE NOT tgisinternal) triggers,
       (SELECT count(*) FROM pg_indexes WHERE schemaname IN ('public','marketplace')) indexes;
