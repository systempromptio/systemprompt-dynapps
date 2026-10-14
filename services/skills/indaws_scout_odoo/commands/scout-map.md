---
name: scout-map
description: Mapa de partners Odoo activos en un país, ordenados por número de clientes activos. Para entender el terreno competitivo y encontrar nichos. Uso, /scout-map <país>.
---

# /scout-map

Lista los partners Odoo activos en un país con métricas de tamaño (clientes activos, largest_users, average_users, retention). Da una foto del terreno competitivo. Vía la tool MCP `scraper_run_query`.

## Flujo

1. Parsear país (ISO o nombre largo → ISO). Si el usuario dice "España" → `ES`, "Portugal" → `PT`, "México" → `MX`, "LATAM" → ejecutar varios países y combinar (o un solo SQL con `country IN (...)`).

2. Ejecutar la plantilla `partners-by-country` de [references/aliases.md](../references/aliases.md), con la variante LATERAL de nivel si se pide Gold/Silver/Ready.

3. Mostrar tabla.

4. Análisis breve añadido:
   - Top 3 por clientes activos.
   - Mediana de clientes/partner (mide concentración del mercado).
   - Cuántos son Gold vs Silver vs Ready (si filtrable).
   - ¿Dónde está Indaws en el ranking? — OJO: Indaws (id=36) tiene `country = NULL` y no sale en `country='ES'`. Consultar su fila aparte (`WHERE p.id = 36` en la misma plantilla sin filtro de país) y posicionarla manualmente.

5. Cierre proactivo:
   > ¿Quieres ver los clientes captados por alguno en los últimos 6 meses? `/scout-new` filtrando por ese partner. O su ficha completa: `/scout-partner <id>`.

## Mapping

| Frase | Parámetro |
|---|---|
| "España" / "ES" | `country = 'ES'` |
| "Gold partners" | variante LATERAL + `level_name ILIKE 'Gold'` |
| "incluyendo muertos" | quitar `customers_active > 0` |
| "top 5" | `LIMIT 5` |

## Ejemplos

- `/scout-map España` → partners ES con clientes activos
- `/scout-map Portugal --level Gold` → solo Gold partners en PT
- `/scout-map México` → mapa LATAM-MX

## Gotcha

`odoo_partners.country` tiene muchos NULLs. Un partner que opera en España puede estar sin país asignado en el directorio (ej. Indaws id=36 tiene country=NULL). Para verlos todos, complementar con búsqueda por nombre.
