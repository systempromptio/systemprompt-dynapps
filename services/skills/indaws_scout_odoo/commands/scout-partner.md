---
name: scout-partner
description: Ficha completa de un partner Odoo (Indaws, Adhoc, QubiQ…) — métricas, clientes, versiones, niveles, retención. Acepta id o nombre. Uso, /scout-partner <id|nombre>.
---

# /scout-partner

Devuelve la ficha completa de un partner del directorio público de Odoo. Combina la receta `partner-detail` (métricas) con `partner-customers` (lista de clientes) en una respuesta consolidada. Todo vía la tool MCP `scraper_run_query`.

## Flujo

1. Detectar si el argumento es un id numérico o un nombre.

2. **Si es nombre**: localizar el id con la query de resolución:

   ```sql
   SELECT p.id, p.name, p.country,
          (SELECT COUNT(*) FROM odoo_partner_customers pc
           WHERE pc.partner_id = p.id AND pc.stop_date IS NULL) AS active_customers
   FROM odoo_partners p
   WHERE p.name ILIKE '%<nombre>%'
   ORDER BY active_customers DESC LIMIT 10;
   ```

   Si hay match único, sigue. Si hay varios candidatos, mostrar la lista y pedir al usuario que elija un id.

3. **Una vez tienes id**, ejecutar las 6 queries de la receta `partner-detail` de [references/aliases.md](../references/aliases.md) (una llamada `scraper_run_query` por query: básicos, counts, top industrias, versiones, niveles, top clientes) y, si se quiere la cartera completa, la receta `partner-customers`.

4. Consolidar en una sola respuesta con secciones:
   - **Datos básicos** (id, nombre, país, start_date, retention).
   - **Tamaño** (largest_users, average_users).
   - **Niveles e historia** (Gold/Silver/Ready y desde cuándo — snapshot más reciente primero).
   - **Versiones Odoo certificadas**.
   - **Top industrias entre sus clientes**.
   - **Clientes recientes** (tabla con los últimos 10-20).
   - **Total clientes activos** y comparativa con sus competidores cercanos.

5. Cierre proactivo:
   > ¿Quieres ver los clientes que ha captado este partner en los últimos 6 meses? `/scout-new` filtrando por este partner. ¿Switchers que vienen *de* este partner? `/scout-switchers` con la variante `ex_partner = <ID>`.

## Argumentos opcionales

- País: filtra clientes mostrados por `country_of_origin`.
- Since: solo clientes con relación iniciada después de esa fecha.
- Histórico: incluye relaciones terminadas (quitar `stop_date IS NULL`).
- Export: escribir la lista de clientes a CSV/XLSX en outputs.

## Ejemplos

- `/scout-partner Indaws` → ficha completa (ver gotcha abajo)
- `/scout-partner 36` → directo por id
- `/scout-partner QubiQ --csv` → ficha + export de sus clientes
- `/scout-partner Adhoc --country ES --since 2025-01-01`

## Gotcha

`Indaws` matchea 2 partners (`Indaws Business Solutions SL` id=36 y `INDAWS` id=3112). Si pasa esto, mostrar ambos y preguntar cuál. Para evitarlo, usar el id directamente.
