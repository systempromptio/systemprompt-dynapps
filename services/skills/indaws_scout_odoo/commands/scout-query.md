---
name: scout-query
description: SQL libre contra la BD del scraper Odoo vía MCP. Para preguntas no cubiertas por los otros comandos. Solo SELECT (lo impone el servidor). Uso, /scout-query <pregunta o SQL>.
---

# /scout-query

Escape hatch para cualquier consulta no cubierta por las recetas predefinidas. Si el usuario pasa SQL crudo, ejecútalo. Si pasa una pregunta en lenguaje natural, conviértela a SQL primero.

## Flujo

1. Detectar tipo de input:
   - Empieza por `SELECT` o `WITH` → SQL crudo, ejecutar directo.
   - Lenguaje natural → traducir a SQL usando el esquema de [`SKILL.md`](../SKILL.md).

2. Ejecutar con la tool MCP `scraper_run_query(sql)`.

3. Mostrar tabla resultado. Si el usuario quiere CSV/Excel: escribir el resultado a fichero en outputs y compartirlo.

## Recordatorios al traducir lenguaje natural a SQL

- **País del cliente** → `c.country_of_origin` (NO `c.country`, está vacío).
- **País del partner** → `odoo_partners.country` (tiene NULLs; Indaws id=36 es NULL).
- **Cliente verdaderamente nuevo** → CTE `first_seen` con `MIN(odoo_partner_customers.start_date)` GROUP BY customer_id.
- **Cliente activo de un partner** → `odoo_partner_customers.stop_date IS NULL`.
- **Switcher** → `odoo_partner_customers.stop_date IS NOT NULL`.
- **Seed inicial** → ignorar registros con `start_date < '2025-07-01'` salvo que el usuario los quiera.
- **`employee_count`** → texto: `'1-10', '11-50', '51-200', '201-500', '501-1000', '1001-5000', '5000+'`.
- **Indaws** → `partner_id = 36`.
- **Niveles** → `odoo_levels.level_name` (NO `name`); snapshot más reciente por `scrape_date`.
- **Versiones** → `odoo_versions.version_name` (NO `name`).
- **Strings de usuario** → escapar `'` como `''`.

## Restricciones del servidor MCP

El servidor rechaza automáticamente:
- DDL/DML (INSERT, UPDATE, DELETE, DROP, ALTER, CREATE, TRUNCATE, GRANT…).
- Multi-statement (más de un `;`).
- Cualquier cosa que no sea un único SELECT / WITH...SELECT.

Además añade `LIMIT` si falta y capa resultados (~1000 filas, `truncated: true`). Si el SQL del usuario contiene mutaciones, **NO intentes esquivarlo** — explica que la BD es solo lectura.

## Ejemplos lenguaje natural

- `/scout-query cuántos clientes activos hay por industria en España`
- `/scout-query distribución de versiones Odoo entre clientes de Indaws`
- `/scout-query partners ES con retention < 70`
- `/scout-query top 10 industrias en LATAM por nº de clientes nuevos últimos 6m`

## Ejemplos SQL crudo

```sql
/scout-query
SELECT c.industry, COUNT(*) AS n
FROM odoo_customers c
WHERE c.country_of_origin = 'ES'
GROUP BY c.industry
ORDER BY n DESC
LIMIT 20;
```

## Tip de oro

Si no estás seguro del esquema o de los valores reales de una columna, usa primero `scraper_describe_table('<tabla>')` o `scraper_sample_table('<tabla>', 5)` antes de escribir la query grande.
