---
name: scout-switchers
description: Lista clientes Odoo que han roto recientemente con su partner (stop_date poblada). Son los leads más calientes — dolor fresco, alta probabilidad de migración. Uso, /scout-switchers <descripción libre>.
---

# /scout-switchers

Clientes cuya relación con un partner ha terminado en los últimos N meses y que no tienen partner activo (huérfanos). Excluye los que nos abandonaron a nosotros por defecto (no queremos llamarles, sabemos por qué se fueron). Vía la tool MCP `scraper_run_query`.

## Por qué este comando

Los switchers son **oro comercial**. Acaban de salir de un partner que les decepcionó, conocen Odoo y están dispuestos a escuchar. Si el `stop_date` es de hace 3-6 meses, el dolor está fresco y ya han tenido tiempo de evaluar opciones — es la ventana ideal.

## Flujo

1. Parsear el mensaje del usuario para país, ventana, tamaño.

2. Ejecutar la plantilla `switchers` de [references/aliases.md](../references/aliases.md):
   - Meses → default 12.
   - Excluir ex-partner Indaws (36) por defecto.
   - País por `country_of_origin`; tier opcional con el CASE de mapeo.

3. Mostrar tabla con `days_since_stop` para que el usuario priorice (5-30 días = ultra-fresco, 30-90 = ideal, 90+ = ya están en otro sitio).

4. Si el usuario quiere CSV/Excel ("Excel", "lista completa"), escribir el resultado completo a un fichero en outputs y compartirlo.

## Mapping

| Frase | Parámetro |
|---|---|
| "España" / "Francia" | `country_of_origin = 'ES'/'FR'` |
| "últimos 3/6/12 meses" | `INTERVAL 'N months'` |
| "frescos" / "recientes" | 6 meses |
| "mid-market" | tier rank >= 3 |
| "incluyendo los que nos dejaron a nosotros" | quitar `partner_id <> 36` |
| "los que dejaron a Adhoc" | `partner_id = <adhoc_id>` (resolver id antes) |

## Ejemplos

- `/scout-switchers España últimos 6 meses` → leads más calientes
- `/scout-switchers Francia 12 meses 51-200 empleados, exporta CSV`
- `/scout-switchers LATAM últimos 3 meses` → usa `c.country_of_origin IN ('MX','CO','AR','PE','CL')` en una sola query
