-- Documents a skill's runs created or edited in the window. A skill run is a
-- harness session that invoked it (`analysis_skill_version_events`, the source
-- the skill page reads); the session's conversations are the
-- `conversation_facts` rows stamped with that client session, and a
-- revision belongs to the run when it carries one of their context ids
-- (MCP fills it from the request, REST from `x-context-id`).
WITH sessions AS (
    SELECT DISTINCT e.session_id
    FROM analysis_skill_version_events e
    WHERE e.skill = $3 AND e.invoked_at >= $1 AND e.invoked_at < $2
      AND ($4::text[] IS NULL OR e.user_id = ANY($4))
),
contexts AS (
    SELECT DISTINCT f.context_id
    FROM sessions s
    JOIN conversation_facts f ON f.client_session_id = s.session_id
),
revisions AS (
    SELECT r.document_id, r.created_at,
           NOT EXISTS (SELECT 1 FROM report_document_revisions p
                       WHERE p.document_id = r.document_id AND p.created_at < r.created_at) AS created
    FROM report_document_revisions r
    JOIN contexts c ON c.context_id = r.context_id
    WHERE r.created_at >= $1 AND r.created_at < $2
)
SELECT r.document_id::text AS "document_id!", d.name AS "name!", d.format AS "format!",
       BOOL_OR(r.created) AS "created!", COUNT(*)::bigint AS "revisions!",
       MAX(r.created_at) AS "last_at!"
FROM revisions r
JOIN documents d ON d.id = r.document_id
GROUP BY r.document_id, d.name, d.format
ORDER BY MAX(r.created_at) DESC
LIMIT $5
