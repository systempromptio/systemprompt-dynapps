---
name: scout-customer
description: Ficha de un cliente del directorio Odoo — datos, industria, tamaño, historial de partners con fechas. Acepta id o referencia (nombre comercial). Uso, /scout-customer <id|referencia>.
---

# /scout-customer

Ficha completa de un cliente público de Odoo, con su historial de partners (qué partners lo han tocado, cuándo empezaron, cuándo terminaron). Todo vía la tool MCP `scraper_run_query`.

## Por qué útil

- Antes de una llamada a frío, ver con quién ha trabajado el cliente.
- Detectar clientes que han rotado entre varios partners (=insatisfechos = oportunidad).
- Cruzar con LinkedIn el nombre comercial (`reference`) para preparar la llamada.

## Flujo

1. Detectar si el argumento es id numérico o referencia textual.

2. **Si es texto**: localizar candidatos:

   ```sql
   SELECT id, reference, industry, employee_count, country_of_origin
   FROM odoo_customers
   WHERE reference ILIKE '%<texto>%'
   ORDER BY id DESC LIMIT 10;
   ```

   Si hay varios, mostrar lista y preguntar.

3. Una vez con id, ejecutar las 2 queries de la receta `customer-detail` de [references/aliases.md](../references/aliases.md) (datos del cliente + historial de partners).

4. Mostrar respuesta consolidada:
   - **Datos básicos** (reference, industry, employee_count, revenue, country_of_origin).
   - **Description** (si la hay — descripción larga del scraper).
   - **Historial de partners** (tabla con todas las relaciones histórico/activas, ordenadas cronológicamente).
   - **Estado actual** (¿con quién está ahora? ¿huérfano? — relación activa = `stop_date IS NULL`).
   - **Antigüedad en el ecosistema Odoo** (primera aparición).

5. Cierre proactivo:
   > ¿Quieres enriquecer este cliente con decisores? Lanza `marketing-skills:lead-prospector` con la referencia "<referencia>".

## Ejemplos

- `/scout-customer 10`
- `/scout-customer Mercadona`
- `/scout-customer "ASLAM NEW IT"` (con comillas si tiene espacios)

## Gotcha

`reference` puede ser un nombre corto, ambiguo o un comercial (no la razón social formal). Si el match no es claro, mostrar candidatos. Escapar `'` como `''` al interpolar en el ILIKE.
