-- Documents a conversation created or edited: revisions that carry its
-- context id (MCP fills it from the request, REST from `x-context-id`).
WITH revisions AS (
    SELECT r.document_id, r.created_at,
           NOT EXISTS (SELECT 1 FROM report_document_revisions p
                       WHERE p.document_id = r.document_id AND p.created_at < r.created_at) AS created
    FROM report_document_revisions r
    WHERE r.context_id = $1
)
SELECT r.document_id::text AS "document_id!", d.name AS "name!", d.format AS "format!",
       BOOL_OR(r.created) AS "created!", COUNT(*)::bigint AS "revisions!",
       MAX(r.created_at) AS "last_at!"
FROM revisions r
JOIN documents d ON d.id = r.document_id
GROUP BY r.document_id, d.name, d.format
ORDER BY MAX(r.created_at) DESC
LIMIT $2
