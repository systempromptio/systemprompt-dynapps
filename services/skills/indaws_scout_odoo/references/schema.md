# Esquema completo de la BD del scraper

Schema `public`. Todas las FKs apuntan a `<tabla>.id`. Verificable en vivo con `odoo-scraper-mcp:scraper_describe_table('<tabla>')`.

## Contents

- Tablas núcleo (partners, customers, relación)
- Tablas satélite (versiones, niveles, categorías, avgusers)
- Semántica temporal (slowly-changing vs snapshots)
- Valores de dominio

## Tablas núcleo

| Tabla | Filas | Columnas | Notas |
|---|---|---|---|
| `odoo_partners` | ~6k | `id, web_id, name, start_date, stop_date, country, largest_users, average_users, customer_retention` | Partners de Odoo (Indaws = id 36; ojo, hay un segundo `INDAWS` id 3112). `country` es el país del partner, pero tiene NULLs (Indaws es NULL). `stop_date` es timestamp de refresco, NO "partner cesado". |
| `odoo_customers` | ~92k | `id, web_id, reference, start_date, stop_date, country, description, industry, revenue, employee_count, country_of_origin, last_enrichment_date` | Clientes finales / referencias. `country` está VACÍO en toda la tabla — usar `country_of_origin` (ISO 2). `start_date` = primera vez que el scraper lo vio, no alta real. `last_enrichment_date` NULL = nunca enriquecido. |
| `odoo_partner_customers` | ~108k | `id, start_date, stop_date, partner_id, customer_id` | Relación N:N quién implementó para quién. `stop_date IS NULL` = relación vigente. Seed inicial 2025-06 con ~51k filas. |

## Tablas satélite

| Tabla | Columnas | Notas |
|---|---|---|
| `odoo_versions` | `id, version_name` | Catálogo de versiones (v18, v17…). Campo `version_name`, NO `name`. |
| `odoo_partner_versions` | `partner_id, version_id, expert_count` | Expertise certificada por versión y partner (~6k). |
| `odoo_levels` | `id, level_name` | 3 filas: Gold / Silver / Ready. Campo `level_name`, NO `name`. |
| `odoo_partner_levels` | `partner_id, level_id, scrape_date` | Histórico de tier (~1.1M, un snapshot por scrape). Nivel ACTUAL = fila con `MAX(scrape_date)` por partner. |
| `odoo_reference_categories` | `id, category_name` | Catálogo de industrias / categorías (~1k). |
| `odoo_partner_reference_categories` | `partner_id, category_id, count, scrape_date` | Conteo de referencias por categoría y partner, por snapshot (~32k). |
| `odoo_partner_avgusers` | `partner_id, average_users, start_date, stop_date` | Histórico de average_users (~16k, slowly-changing). |

## Semántica temporal

- **Slowly-changing** (`start_date` / `stop_date`): la fila VIGENTE tiene `stop_date IS NULL`. Aplica a `odoo_partner_customers`, `odoo_partner_avgusers`.
- **Snapshots** (`scrape_date`): un registro por scrape. Para el estado actual, filtrar al `MAX(scrape_date)` (por partner o global). Aplica a `odoo_partner_levels`, `odoo_partner_reference_categories`.

## Valores de dominio

- `employee_count` (texto, ordinal): `'1-10'`, `'11-50'`, `'51-200'`, `'201-500'`, `'501-1000'`, `'1001-5000'`, `'5000+'`. Para comparar usar el CASE de mapeo a ranks (ver aliases.md → Constantes).
- `country_of_origin` / `country`: ISO 2 letras en mayúsculas (`ES`, `FR`, `PT`, `MX`, `US`).
- `level_name`: `Gold`, `Silver`, `Ready`.
- Constantes operativas: Indaws `partner_id = 36`; seed cutoff `'2025-07-01'`.
