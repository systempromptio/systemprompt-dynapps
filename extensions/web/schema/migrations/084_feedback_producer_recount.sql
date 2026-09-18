-- Replaces the blind pending_count decrement in the feedback drain with a
-- recount from the queue; the live counter had already drifted below the
-- pending rows and tripped its own CHECK on every batch.
-- pending_count is derived state. It is recounted from the queue whenever a
-- drain settles, so a counter that drifted (a re-enqueue that missed the
-- was_pending check, a backfill source added after the seed) can never wedge
-- the drain on its own CHECK. An undone backfill source counts as pending so
-- retention keeps waiting for it. Caller holds the producer row FOR UPDATE.
CREATE OR REPLACE FUNCTION feedback_producer_recount(drained BOOLEAN)
RETURNS void LANGUAGE sql AS $$
UPDATE analytics_ingestion_producers p SET
    pending_count=c.n,
    oldest_pending_at=CASE WHEN c.n=0 THEN NULL ELSE COALESCE(p.oldest_pending_at,clock_timestamp()) END,
    checkpoint=CASE WHEN drained THEN p.checkpoint+1 ELSE p.checkpoint END,
    last_drained_at=CASE WHEN drained THEN clock_timestamp() ELSE p.last_drained_at END,
    updated_at=clock_timestamp()
FROM (SELECT (SELECT count(*) FROM feedback_ingestion_outbox WHERE pending)
           + (SELECT count(*) FROM feedback_ingestion_backfill WHERE NOT done) AS n) c
WHERE p.producer='astound.feedback'
$$;
SELECT feedback_producer_recount(false);
