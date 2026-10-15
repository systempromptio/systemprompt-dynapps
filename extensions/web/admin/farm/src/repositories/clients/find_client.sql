SELECT id AS "id: ClientId", name, slug, odoo_partner_id, created_at
FROM clients
WHERE slug = $1 OR id = $1
