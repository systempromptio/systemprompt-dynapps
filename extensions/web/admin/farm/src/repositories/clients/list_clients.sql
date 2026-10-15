SELECT id AS "id: ClientId", name, slug,
       service_account_user_id AS "service_account_user_id: UserId", created_at
FROM clients
ORDER BY name, id
