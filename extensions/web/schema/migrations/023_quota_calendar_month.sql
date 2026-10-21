-- Calendar-month quota windows move from this extension's interim to core's
-- `calendar: month`, and the current month's usage moves with them.
--
-- The interim declared a month as window_seconds 2678400 (31 days) and a
-- daily job rewrote the live policy row to 86400 x (31 + days left in the
-- month, today included), a value in 32..62 days that no hand-set window
-- uses. Each rewrite was a new bucket key, so the job carried yesterday's
-- bucket into today's: the bucket at the newest key held month-to-date usage.
--
-- 1. Policies. Every quota window whose window_seconds is the sentinel or a
--    whole number of days in 32..62 becomes {calendar: month,
--    window_seconds: 0}; core ignores window_seconds on a calendar window.
--    Window order and every other field are kept.
--
-- 2. Buckets. Core keys a calendar-month bucket on (window_seconds = the
--    month's length in seconds, window_start = 00:00 UTC on the first). For
--    each (subject_kind, subject_id), the interim rows updated this UTC
--    month are read and the newest daily key is taken: the smallest
--    window_seconds, because the days left only shrink through a month.
--    Before the day's rewrite runs the gateway still writes yesterday's key,
--    which is then the smallest one present, so the choice is right either
--    side of it. That key's counters, summed over its shards, are inserted
--    under the calendar key as one row.
--    Earlier days' rows are already inside it by carry-forward and are left
--    for retention. Rows are read and written without naming `shard`, so
--    this holds whether or not core's shard migration has run yet; the new
--    row lands on shard 0, and core's admission sums every shard.
UPDATE ai_gateway_policies p
   SET spec = jsonb_set(
           p.spec,
           '{quota_windows}',
           (SELECT jsonb_agg(
                       CASE
                           WHEN w->>'calendar' IS NULL
                            AND ((w->>'window_seconds')::int = 2678400
                                 OR ((w->>'window_seconds')::int % 86400 = 0
                                     AND (w->>'window_seconds')::int / 86400 BETWEEN 32 AND 62))
                           THEN (w - 'window_seconds')
                                || '{"calendar": "month", "window_seconds": 0}'::jsonb
                           ELSE w
                       END
                       ORDER BY ord)
              FROM jsonb_array_elements(p.spec->'quota_windows') WITH ORDINALITY AS t(w, ord))),
       updated_at = CURRENT_TIMESTAMP
 WHERE jsonb_typeof(p.spec->'quota_windows') = 'array'
   AND EXISTS (
       SELECT 1
         FROM jsonb_array_elements(p.spec->'quota_windows') AS e(w)
        WHERE w->>'calendar' IS NULL
          AND ((w->>'window_seconds')::int = 2678400
               OR ((w->>'window_seconds')::int % 86400 = 0
                   AND (w->>'window_seconds')::int / 86400 BETWEEN 32 AND 62)));

WITH month AS (
    SELECT (date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC') AS start
),
calendar_key AS (
    SELECT start,
           EXTRACT(EPOCH FROM (start + INTERVAL '1 month') - start)::int AS seconds
      FROM month
),
live AS (
    SELECT b.subject_kind, b.subject_id, b.window_seconds,
           b.requests, b.input_tokens, b.output_tokens, b.cost_microdollars
      FROM ai_quota_buckets b, month m
     WHERE b.window_seconds % 86400 = 0
       AND b.window_seconds / 86400 BETWEEN 32 AND 62
       AND b.updated_at >= m.start
),
newest AS (
    SELECT subject_kind, subject_id, MIN(window_seconds) AS window_seconds
      FROM live
     GROUP BY subject_kind, subject_id
),
carried AS (
    SELECT l.subject_kind, l.subject_id,
           SUM(l.requests) AS requests,
           SUM(l.input_tokens) AS input_tokens,
           SUM(l.output_tokens) AS output_tokens,
           SUM(l.cost_microdollars) AS cost_microdollars
      FROM live l
      JOIN newest n USING (subject_kind, subject_id, window_seconds)
     GROUP BY l.subject_kind, l.subject_id
)
INSERT INTO ai_quota_buckets
       (id, subject_kind, subject_id, window_seconds, window_start,
        requests, input_tokens, output_tokens, cost_microdollars, updated_at)
SELECT gen_random_uuid()::text, c.subject_kind, c.subject_id, k.seconds, k.start,
       c.requests, c.input_tokens, c.output_tokens, c.cost_microdollars, CURRENT_TIMESTAMP
  FROM carried c, calendar_key k
 WHERE NOT EXISTS (
       SELECT 1 FROM ai_quota_buckets x
        WHERE x.subject_kind = c.subject_kind
          AND x.subject_id = c.subject_id
          AND x.window_seconds = k.seconds
          AND x.window_start = k.start);
