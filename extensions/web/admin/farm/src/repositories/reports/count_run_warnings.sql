-- Governance `warn` findings a run's tool calls produced: the same join as
-- the run page's governance tab (the run's gateway and host sessions, or its
-- conversation context).
SELECT COUNT(*) AS "warnings!"
FROM governance_decisions
WHERE decision = 'warn'
  AND (session_id = ANY($1::text[]) OR ($2::text IS NOT NULL AND context_id = $2))
