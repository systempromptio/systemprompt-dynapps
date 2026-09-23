-- Session summaries are maintained inline, not drained from an outbox.
--
-- Every hook event inserted an ingestion_outbox row, and the drain that
-- followed recomputed the whole session's summary from every event it had
-- ever recorded. `/hooks/track` now adds the one event's counts to
-- plugin_session_summaries in the transaction that inserts it, so the
-- summary can neither miss an event nor count one twice, and a write costs
-- one row regardless of the session's length.
--
-- Whatever is still queued is folded in first, by the drain being retired.
DO $$
BEGIN
    IF to_regclass('ingestion_outbox') IS NOT NULL
       AND to_regprocedure('drain_ingestion_outbox(integer)') IS NOT NULL THEN
        WHILE drain_ingestion_outbox(10000) > 0 LOOP
        END LOOP;
    END IF;
END $$;

DROP TRIGGER IF EXISTS ingestion_enqueue ON plugin_usage_events;
DROP FUNCTION IF EXISTS enqueue_ingestion_event();
DROP FUNCTION IF EXISTS drain_ingestion_outbox(INTEGER);
DROP TABLE IF EXISTS ingestion_outbox;
