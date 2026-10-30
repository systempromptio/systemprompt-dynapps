-- Distinct documents that gained a revision in the window, by an author in
-- scope.
SELECT COUNT(DISTINCT r.document_id)::bigint AS "touched!"
FROM report_document_revisions r
WHERE r.created_at >= $1 AND r.created_at < $2
  AND ($3::text[] IS NULL OR r.author_user_id = ANY($3))
