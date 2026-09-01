-- Health report for the shared knowledge index. Read-only.
--   just knowledge-db-report
-- Matches what the admin-only knowledge_index_stats tool reports, plus the
-- per-user sync table that decides what prune_knowledge would drop.

\echo '== database'
SELECT current_database()                                   AS database,
       pg_size_pretty(pg_database_size(current_database())) AS size,
       (SELECT extversion FROM pg_extension WHERE extname = 'vector') AS pgvector;

\echo '== totals'
SELECT (SELECT COUNT(*) FROM knowledge_documents)                          AS documents,
       (SELECT COUNT(*) FROM knowledge_chunks)                             AS chunks,
       (SELECT COUNT(*) FROM knowledge_document_grants)                    AS grants,
       (SELECT COUNT(DISTINCT user_id) FROM knowledge_user_sync_state)     AS syncing_users,
       (SELECT COUNT(*) FROM knowledge_embedding_cache)                    AS embedding_cache_rows,
       (SELECT COUNT(*) FROM knowledge_date_cache)                         AS date_cache_rows,
       (SELECT value FROM knowledge_index_state WHERE key = 'version')     AS index_version,
       (SELECT value FROM knowledge_index_state WHERE key = 'last_built')  AS last_built;

\echo '== documents per source, and how many users hold each'
SELECT d.source_type,
       COUNT(*)                                             AS documents,
       COUNT(*) FILTER (WHERE g.holders = 0)               AS orphaned,
       ROUND(AVG(g.holders), 2)                            AS avg_holders
FROM knowledge_documents d
LEFT JOIN (
    SELECT doc_id, COUNT(*) AS holders
    FROM knowledge_document_grants WHERE subject_kind = 'user'
    GROUP BY doc_id
) g ON g.doc_id = d.doc_id
GROUP BY d.source_type ORDER BY d.source_type;

\echo '== per-user sync state (cursor, last run, last error, counters)'
SELECT source_type, user_id, last_sync, last_run_at, last_error,
       documents_seen, documents_written, documents_pruned, documents_failed
FROM knowledge_user_sync_state
ORDER BY last_run_at DESC NULLS LAST, source_type, user_id;

\echo '== grants that name a user with no sync state (manual or legacy grants)'
SELECT subject_value AS user_id, COUNT(*) AS grants
FROM knowledge_document_grants g
WHERE subject_kind = 'user'
  AND NOT EXISTS (SELECT 1 FROM knowledge_user_sync_state s WHERE s.user_id = g.subject_value)
GROUP BY subject_value ORDER BY grants DESC;
