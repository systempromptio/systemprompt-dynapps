INSERT INTO clients (id, name, slug, service_account_user_id)
VALUES ($1, $2, $1, $3)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    service_account_user_id = EXCLUDED.service_account_user_id
