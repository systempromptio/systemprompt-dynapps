-- The conversation contexts of one Claude Code session ($1 = the farm task's
-- session_ref without its sess_ prefix).
SELECT context_id
FROM conversation_session_contexts
WHERE client_session_id = $1
ORDER BY context_id
LIMIT 10
