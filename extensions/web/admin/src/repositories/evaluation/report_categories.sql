-- Conversation categories in a scope and window with the judge's means and
-- the token and cost load per conversation: where "category Y burns tokens
-- without achieving more" comes from.
WITH scoped AS (
    SELECT a.context_id, a.category, a.outcome, m.client_session_id,
           m.total_input_tokens + m.total_output_tokens AS tokens, m.total_cost_microdollars AS cost
      FROM conversation_analyses a
      JOIN LATERAL conversation_metrics_for(ARRAY[a.context_id]) m ON true
     WHERE a.status = 'classified' AND a.classified_at >= $2 AND a.classified_at < $3
       AND ($1::text IS NULL OR EXISTS (
             SELECT 1 FROM evaluation_session_skills k
              WHERE k.session_id = m.client_session_id AND k.marketplace_id = $1))
)
SELECT s.category AS "category!",
       COUNT(*)::bigint AS "conversations!",
       COUNT(*) FILTER (WHERE s.outcome = 'achieved')::bigint AS "achieved!",
       COUNT(c.session_id)::bigint AS "scored!",
       AVG(c.intent)::float8 AS intent, AVG(c.quality)::float8 AS quality,
       AVG(c.speed)::float8 AS speed, AVG(c.cost)::float8 AS cost,
       AVG(s.tokens)::float8 AS tokens_avg, AVG(s.cost)::float8 AS cost_avg_microdollars
  FROM scoped s
  LEFT JOIN evaluation_session_scorecard c ON c.session_id = s.client_session_id AND c.scored_criteria > 0
 GROUP BY s.category
 ORDER BY "conversations!" DESC
