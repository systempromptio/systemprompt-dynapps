-- D27: every outbox
\echo == D27 event_outbox
SELECT channel, consumer, count(*), count(*) FILTER (WHERE processed_at IS NULL) pending, min(created_at)::timestamp(0) oldest, pg_size_pretty(max(octet_length(fact::text))::bigint) largest_fact FROM event_outbox GROUP BY 1,2;
\echo == D27 feedback_ingestion_outbox
SELECT kind, count(*), count(*) FILTER (WHERE pending) pending, min(created_at)::timestamp(0) oldest, max(attempts) attempts, count(*) FILTER (WHERE lease_token IS NOT NULL) leased FROM feedback_ingestion_outbox GROUP BY 1;
SELECT * FROM analytics_ingestion_producers;
\echo == D27 ingestion_outbox (plugin_usage_events -> plugin_session_summaries)
SELECT count(*), count(*) FILTER (WHERE processed_at IS NULL) pending, min(created_at)::timestamp(0) oldest_pending FROM ingestion_outbox WHERE processed_at IS NULL;
SELECT count(*) total, count(*) FILTER (WHERE processed_at IS NULL) pending FROM ingestion_outbox;
\echo == D27 ingestion_repairs (what got repaired)
SELECT repair_kind, count(*), min(repaired_at)::date, max(repaired_at)::date FROM ingestion_repairs GROUP BY 1;
\echo == D27 managed_distribution_outbox
SELECT count(*) FROM managed_distribution_outbox;
