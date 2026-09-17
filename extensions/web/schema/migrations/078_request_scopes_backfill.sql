-- Twin of schema/40_request_scopes.sql: stamp history.
--
-- The trigger only sees requests inserted after it exists. Every request
-- already on the books is attributed here from the person's primary group
-- and project as they stand today — the best answer available, and the same
-- one the old membership join gave. From this point on a request's scope is
-- fixed when it lands and moving a person never rewrites it.

INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
SELECT r.id,
       r.user_id,
       CASE WHEN r.actor_kind = 'user' THEN d.primary_group_id END,
       CASE WHEN r.actor_kind = 'user' THEN d.primary_project_id END,
       'primary'
FROM ai_requests r
LEFT JOIN user_scope_defaults d ON d.user_id = r.user_id
ON CONFLICT (request_id) DO NOTHING;
