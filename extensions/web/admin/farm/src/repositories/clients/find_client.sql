SELECT id AS "id: ClientId", name, slug,
       service_account_user_id AS "service_account_user_id: UserId", created_at
FROM clients
WHERE slug = $1 OR id = $1
