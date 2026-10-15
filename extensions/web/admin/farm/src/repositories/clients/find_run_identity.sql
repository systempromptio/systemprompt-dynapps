-- Who a run was for, who launched it and which identity its credentials
-- were minted for (core's agent_runs.user_id).
SELECT c.id AS "client_id?: ClientId",
       c.name AS "client_name?",
       COALESCE(NULLIF(lu.display_name, ''), lu.email, r.user_id) AS "launcher_name!",
       ar.user_id AS "run_as_id?: UserId",
       COALESCE(NULLIF(ru.display_name, ''), ru.email, ar.user_id) AS "run_as_name?"
FROM agent_farm_runs r
LEFT JOIN clients c ON c.id = r.client_id
LEFT JOIN users lu ON lu.id = r.user_id
LEFT JOIN agent_runs ar ON ar.id = r.agent_run_id
LEFT JOIN users ru ON ru.id = ar.user_id
WHERE r.id = $1
