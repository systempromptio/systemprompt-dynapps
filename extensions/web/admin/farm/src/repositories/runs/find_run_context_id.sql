SELECT context_id AS "context_id: ContextId"
FROM conversation_session_contexts
WHERE user_id = $1 AND client_session_id = $2
