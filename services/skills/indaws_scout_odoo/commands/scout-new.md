---
name: scout-new
description: Extrae clientes verdaderamente nuevos de Odoo (primera aparición en últimos N meses) con filtros opcionales por país, tamaño, industria, cooldown y partner. Uso, /scout-new <descripción libre>.
---

# /scout-new

Devuelve clientes que aparecen por primera vez en el directorio de Odoo en los últimos N meses, con filtros opcionales. Excluye Indaws (partner 36) por defecto. Se ejecuta vía la tool MCP `scraper_run_query` — sin credenciales.

## Flujo

1. Parsear la descripción del usuario para extraer parámetros (país, ventana, tamaño, industria, cooldown, excluir/incluir partners, etc.). Si algo no queda claro, pregunta UNA cosa antes de ejecutar.

2. Tomar la plantilla `new-clients` de [references/aliases.md](../references/aliases.md), sustituir los placeholders y ajustar las condiciones opcionales.

3. Ejecutar con `scraper_run_query(sql)`. Mostrar la tabla resultante en chat.

4. Si el usuario pidió Excel/CSV explícita o implícitamente ("dame Excel", "exporta", "lista completa"), escribir el resultado completo a un `.csv`/`.xlsx` en la carpeta de outputs (subir el LIMIT lo necesario; si `truncated: true`, paginar con OFFSET) y compartir el fichero.

5. Si hay 0 resultados, sugerir relajar filtros concretos: ampliar meses, quitar país/industria, quitar tier, incluir seed.

## Mapping rápido lenguaje → parámetros

| Frase del usuario | Parámetro |
|---|---|
| "España" / "ES" | `country_of_origin = 'ES'` |
| "Portugal", "Francia", "México"… | `'PT' / 'FR' / 'MX'` |
| "últimos 3/6/12 meses" | `INTERVAL 'N months'` |
| "mid-market" / "medianas" | tier rank >= 3 (51-200) |
| "grandes" / "enterprise" | tier rank >= 4 (201-500) |
| "pyme" / "SMB" | tier rank = 2 (11-50) |
| "manufactura", "retail", "salud" | `industry ILIKE '%Manufacturing%'` etc. |
| "post luna de miel" / "maduros" | cooldown: `first_seen <= now() - 6 months` |
| "que nunca hayan sido nuestros" | variante never-with-partner 36 |
| "solo clientes de Adhoc" | EXISTS con `partner_id = <id>` (resolver id antes) |
| "todos, sin excluir Indaws" | quitar el bloque EXISTS `<> 36` |

## Ejemplos

- `/scout-new España últimos 6 meses mid-market que nunca hayan sido nuestros, exporta CSV`
- `/scout-new Francia 12 meses manufactura 51-200 empleados`
- `/scout-new clientes nuevos de QubiQ últimos 6 meses` → primero resolver el id de QubiQ (query de resolución en aliases.md), luego esta receta con `partner_id = <id>`
