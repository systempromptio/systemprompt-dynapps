-- Spend and requests per bucket, split by one dimension of `ai_requests`:
-- the model that served the request or the client that made it. The top six
-- keys by window cost keep their own series and the tail folds into 'Other',
-- so the seven chart colours always suffice and a key's colour agrees with
-- the model board's ranking. Rows are sparse — the view pivots them onto the
-- same $3-bucket spine the overview series draws.
WITH keyed AS (
    SELECT LEAST($3::INT - 1,
                 FLOOR(EXTRACT(EPOCH FROM (r.created_at - $1)) / $4::FLOAT8)::INT) AS bucket,
           CASE WHEN $5::TEXT = 'model' THEN COALESCE(r.model, 'unrouted')
                ELSE r.client_kind END AS key,
           r.cost_microdollars
    FROM ai_requests r
    WHERE NOT r.synthetic
      AND r.created_at >= $1
      AND r.created_at < $2
), ranked AS (
    SELECT key
    FROM keyed
    GROUP BY key
    ORDER BY SUM(cost_microdollars) DESC, COUNT(*) DESC, key
    LIMIT 6
)
SELECT k.bucket AS "bucket!",
       CASE WHEN ranked.key IS NULL THEN 'Other' ELSE k.key END AS "key!",
       COALESCE(SUM(k.cost_microdollars), 0)::BIGINT AS "cost!",
       COUNT(*)::BIGINT AS "requests!"
FROM keyed k
LEFT JOIN ranked ON ranked.key = k.key
GROUP BY 1, 2
ORDER BY 1, 2
