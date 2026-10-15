INSERT INTO clients (id, name, slug, odoo_partner_id, service_account_user_id)
VALUES ($1, $2, $1, $3, $4)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    odoo_partner_id = EXCLUDED.odoo_partner_id,
    service_account_user_id = EXCLUDED.service_account_user_id
