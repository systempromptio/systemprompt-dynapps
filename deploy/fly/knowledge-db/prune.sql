-- Prune the shared knowledge index. The same statements the admin-only
-- prune_knowledge tool runs, for when the server is down or psql is preferred.
--   just knowledge-db-prune                 # dry run, STALE_DAYS=90
--   DRY_RUN=0 just knowledge-db-prune       # commit
-- Bind :stale_days (integer). The whole run is one transaction; the recipe
-- appends ROLLBACK for a dry run and COMMIT otherwise.
--
-- What it cannot do here that the tool does: drop grants of users whose
-- `users` row is gone — that table lives in the INSTANCE database, not this
-- one. Run the tool for that part, or pass the ids by hand.

BEGIN;

\echo '== stale users (no sync in :stale_days days)'
CREATE TEMP TABLE stale_users ON COMMIT DROP AS
SELECT user_id
FROM knowledge_user_sync_state
GROUP BY user_id
HAVING MAX(last_run_at) < NOW() - (:stale_days || ' days')::interval;
SELECT * FROM stale_users;

\echo '== grants dropped for stale users'
WITH dropped AS (
    DELETE FROM knowledge_document_grants
    WHERE subject_kind = 'user' AND subject_value IN (SELECT user_id FROM stale_users)
    RETURNING 1
) SELECT COUNT(*) AS grants_dropped FROM dropped;

WITH dropped AS (
    DELETE FROM knowledge_user_sync_state
    WHERE user_id IN (SELECT user_id FROM stale_users)
    RETURNING 1
) SELECT COUNT(*) AS sync_rows_dropped FROM dropped;

\echo '== documents left with zero grants (chunks cascade)'
WITH dropped AS (
    DELETE FROM knowledge_documents d
    WHERE NOT EXISTS (SELECT 1 FROM knowledge_document_grants g WHERE g.doc_id = d.doc_id)
    RETURNING 1
) SELECT COUNT(*) AS documents_deleted FROM dropped;

\echo '== date-cache rows for documents that no longer exist'
WITH dropped AS (
    DELETE FROM knowledge_date_cache c
    WHERE NOT EXISTS (SELECT 1 FROM knowledge_documents d WHERE d.doc_id = c.doc_id)
    RETURNING 1
) SELECT COUNT(*) AS date_cache_deleted FROM dropped;

\echo '== embedding-cache rows older than :stale_days days that no chunk references'
WITH dropped AS (
    DELETE FROM knowledge_embedding_cache c
    WHERE c.created_at < NOW() - (:stale_days || ' days')::interval
      AND NOT EXISTS (
          SELECT 1 FROM knowledge_chunks k
          WHERE encode(sha256(convert_to(k.text, 'UTF8')), 'hex') = c.text_hash
      )
    RETURNING 1
) SELECT COUNT(*) AS embedding_cache_deleted FROM dropped;

-- The recipe appends one of these; running the file by hand, choose:
-- ROLLBACK;   -- dry run
-- COMMIT;     -- apply
