---
name: scout-rank
description: Ranking de partners Odoo por número de clientes nuevos captados en los últimos N meses. Para saber a quién pegamos y quién nos está pegando. Uso, /scout-rank <descripción libre>.
---

# /scout-rank

Top partners ordenados por nº de clientes nuevos captados en una ventana temporal. Vía la tool MCP `scraper_run_query`. Sirve para:

- **Identificar competidores en alza** (quién está creciendo más rápido).
- **Priorizar campañas anti-competidor** (a quién pegamos primero).
- **Ver dónde está Indaws** (re-ejecutar sin excluir el 36).

## Flujo

1. Parsear país, ventana, tamaño mínimo de clientes captados.

2. Ejecutar la plantilla `competitor-ranking` de [references/aliases.md](../references/aliases.md):
   - País → `country_of_origin` de los CLIENTES captados, no del partner.
   - Meses → default 6. Seed cutoff `'2025-07-01'` SIEMPRE activo.
   - Indaws excluido por defecto (`p.id <> 36`).
   - LIMIT default 20.

3. Mostrar tabla con columnas: `partner_id, name, partner_country, customers_captured, midmarket_captured`.

4. Si el usuario pregunta "¿y nosotros dónde estamos?" → re-ejecuta quitando `p.id <> 36` y localiza la fila de Indaws.

## Mapping

| Frase | Parámetro |
|---|---|
| "España" | `country_of_origin = 'ES'` |
| "últimos 3/6/12 meses" | `INTERVAL 'N months'` |
| "mid-market" | tier rank >= 3 en el filtro |
| "incluyendo Indaws" | quitar `p.id <> 36` |
| "top 5/10/30" | `LIMIT N` |

## Ejemplos

- `/scout-rank España últimos 6 meses` → mapa competitivo ES estándar
- `/scout-rank España últimos 12 meses mid-market top 10`
- `/scout-rank México 6 meses incluyendo Indaws` → para ver si estamos en el ranking LATAM

## Output útil para narrativa

Después del ranking, ofrece al usuario:

> ¿Quieres que abra una ficha de algún competidor del top? Indica el partner_id y lanzamos `/scout-partner ID`.
