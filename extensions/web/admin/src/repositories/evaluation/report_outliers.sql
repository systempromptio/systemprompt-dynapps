-- The worst and best judged sessions in a scope and window: the sessions the
-- report quotes. Scope is the whole instance ($1 NULL) or one marketplace
-- (sessions that invoked one of its skills). Each side is capped at $4 rows.
WITH scoped AS (
    SELECT c.session_id, c.user_id, c.intent, c.quality, c.speed, c.cost, c.last_scored_at
      FROM evaluation_session_scorecard c
     WHERE c.scored_criteria > 0 AND c.quality IS NOT NULL
       AND c.last_scored_at >= $2 AND c.last_scored_at < $3
       AND ($1::text IS NULL OR EXISTS (
             SELECT 1 FROM evaluation_session_skills k
              WHERE k.session_id = c.session_id AND k.marketplace_id = $1))
), worst AS (
    SELECT s.*, true AS is_worst FROM scoped s
     ORDER BY s.quality ASC, s.intent ASC NULLS LAST, s.last_scored_at DESC LIMIT $4
), best AS (
    SELECT s.*, false AS is_worst FROM scoped s
     WHERE s.session_id NOT IN (SELECT session_id FROM worst)
     ORDER BY s.quality DESC, s.intent DESC NULLS LAST, s.last_scored_at DESC LIMIT $4
), picked AS (
    SELECT * FROM worst UNION ALL SELECT * FROM best
)
SELECT p.session_id AS "session_id!: SessionId", p.is_worst AS "is_worst!",
       p.intent, p.quality, p.speed, p.cost,
       (SELECT ar.context_id FROM ai_requests ar
         WHERE ar.client_session_id = p.session_id AND ar.actor_kind <> 'job'
         ORDER BY ar.created_at DESC LIMIT 1) AS "context_id: ContextId",
       ps.ai_title AS title,
       (SELECT a.category FROM conversation_analyses a
         JOIN ai_requests ar ON ar.context_id = a.context_id
        WHERE ar.client_session_id = p.session_id LIMIT 1) AS category,
       COALESCE((SELECT array_agg(DISTINCT k.skill) FROM evaluation_session_skills k
                  WHERE k.session_id = p.session_id), ARRAY[]::text[]) AS "skills!",
       q.rationale AS quality_rationale,
       (SELECT e->>'quote' FROM jsonb_array_elements(qr.evidence) e LIMIT 1) AS evidence_quote,
       i.rationale AS intent_rationale
  FROM picked p
  LEFT JOIN plugin_session_summaries ps ON ps.session_id = p.session_id
  LEFT JOIN evaluation_session_scores q ON q.session_id = p.session_id AND q.criterion = 'quality' AND q.status = 'succeeded'
  LEFT JOIN evaluation_runs qr ON qr.id = q.run_id
  LEFT JOIN evaluation_session_scores i ON i.session_id = p.session_id AND i.criterion = 'intent' AND i.status = 'succeeded'
 ORDER BY p.is_worst DESC, p.quality
