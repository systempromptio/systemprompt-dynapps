INSERT INTO clients (id, name, slug, odoo_partner_id)
VALUES ($1, $2, $1, $3)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    odoo_partner_id = EXCLUDED.odoo_partner_id
