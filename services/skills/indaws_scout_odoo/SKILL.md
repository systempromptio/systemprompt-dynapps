---
name: indaws-scout-odoo
description: >
  Consulta inteligencia competitiva Odoo via BD publica del scraper (MCP odoo-scraper-mcp, solo
  SELECT): clientes por pais, partners, switchers, ranking de competidores. NOT para CRM/ventas
  interno → common-skills:odoo-pilot. NOT para enriquecer decisores → lead-prospector.
  Triggers: nuevos clientes de Odoo, quien esta comprando Odoo en Espana, clientes de la
  competencia, cuantos clientes tiene Adhoc/Dynapps/Indaws, switchers Odoo,
  clientes huerfanos, ranking competidores Odoo, mapa competitivo Odoo.
version: 2.1.3
effort: medium
model: sonnet
---

# Odoo References — Consultas a la BD del scraper (vía MCP)

## Qué es

Base de datos PostgreSQL con datos públicos de partners de Odoo (directorio web odoo.com) y los clientes que cada partner declara. Indaws = `partner_id = 36`.

El acceso es **exclusivamente vía MCP** — servidor `odoo-scraper-mcp` en `https://odoo-scraper-mcp.xv.infra.dyncloud.io/mcp`. **No hay usuario, contraseña ni `.env`**: el MCP gestiona la conexión. Solo lectura: un único `SELECT` / `WITH...SELECT` por llamada; DDL/DML y multi-statement los rechaza el servidor; añade `LIMIT` automático si falta, con cap ~1000 filas.

## Herramientas MCP

Nombres cualificados: `odoo-scraper-mcp:<tool>`.

| Tool | Para qué |
|---|---|
| `odoo-scraper-mcp:scraper_run_query(sql)` | Ejecuta un SELECT y devuelve filas en JSON. La herramienta principal — el 95% del uso. |
| `odoo-scraper-mcp:scraper_list_tables()` | Lista tablas con row counts. Para descubrir el esquema. |
| `odoo-scraper-mcp:scraper_describe_table(table)` | Columnas, tipos, PKs, FKs e índices de una tabla. |
| `odoo-scraper-mcp:scraper_sample_table(table, limit)` | Filas de muestra. Para inspeccionar valores reales. |

> **¿No aparecen las tools?** [STOP] El MCP `odoo-scraper-mcp` no está conectado en esta sesión. Pide al usuario que lo conecte con la URL `https://odoo-scraper-mcp.xv.infra.dyncloud.io/mcp` y espera. La ruta depende del agente: en Claude Code, Ajustes → Conectores; en otros agentes, el bloque de configuración MCP correspondiente. NO intentes conectar por psql/psycopg2 ni pidas credenciales — no existen para este flujo.

## Workflow

1. Identifica qué pregunta hace el usuario y localiza la **receta SQL** correspondiente en [references/aliases.md](references/aliases.md) (new-clients, switchers, competitor-ranking, etc.).
2. Adapta la plantilla (país, meses, tier de empleados, partner) sustituyendo los placeholders.
3. [STOP] Si una resolución por nombre (partner o cliente) devuelve más de un candidato, muestra la lista y pide al usuario el id antes de seguir. Nunca elijas tú entre candidatos ambiguos.
4. Ejecuta con `scraper_run_query`.
5. Presenta el resultado como tabla en chat. Si el usuario pide Excel/CSV ("dame un Excel", "exporta", "lista completa"), escribe el resultado a un `.csv` o `.xlsx` en la carpeta de outputs con las file tools y compártelo.

No hay flags de CLI: el `LIMIT` va dentro del SQL y la exportación la haces tú con Write. Si el resultado viene con `truncated: true`, pagina con `OFFSET` o estrecha filtros.

## NEVER / ALWAYS

**NEVER**
1. Conectar a PostgreSQL directamente (psql, psycopg2, DSN) ni pedir/almacenar credenciales — el único acceso es el MCP. *Razón: v1 usaba .env con password; v2 lo eliminó por diseño.*
2. Intentar esquivar las restricciones del servidor (DDL/DML, multi-statement) — la BD es solo lectura by design.
3. Elegir entre candidatos ambiguos en una resolución por nombre — preguntar siempre.
4. Interpolar strings de usuario sin escapar `'` como `''` — riesgo de SQL roto.

**ALWAYS**
1. Excluir Indaws (`partner_id = 36`) por defecto en new-clients, switchers y rankings, salvo que el usuario pida lo contrario.
2. Aplicar el seed cutoff `first_seen >= '2025-07-01'` en métricas de "nuevos" y captación.
3. Usar `country_of_origin` para país de cliente y `odoo_partners.country` para país de partner.
4. Avisar al usuario cuando un resultado venga `truncated` y ofrecer paginar o exportar.

## Esquema (resumen)

Tablas núcleo: `odoo_partners` (~6k), `odoo_customers` (~92k), `odoo_partner_customers` (~108k, relación N:N con fechas). Satélites: versiones, niveles Gold/Silver/Ready (histórico por `scrape_date`), categorías. Esquema completo con columnas en [references/schema.md](references/schema.md) — o usa `scraper_describe_table` en vivo.

### GOTCHAS

1. **`odoo_customers.country` está vacío en TODA la tabla.** El país real está en `country_of_origin` (ISO: `ES`, `FR`, `US`).
2. **`odoo_customers.start_date` NO es la fecha de alta del cliente** — es cuándo el scraper lo vio. "Nuevo de verdad" = `MIN(odoo_partner_customers.start_date)` por `customer_id`.
3. **Seed inicial 2025-06** (~51k filas). Filtrar `first_seen >= '2025-07-01'` para descartar ruido.
4. **`employee_count` es un rango en texto** (`'1-10'`…`'5000+'`). Para comparar, usar el CASE de mapeo a ranks (aliases.md).
5. **`stop_date IS NULL` = relación vigente** (tablas slowly-changing). Tablas con `scrape_date` son snapshots → filtrar al más reciente.
6. **`odoo_partners.stop_date` NO significa "partner cesado"** — es un timestamp de refresco. "Partner activo" = ≥ 1 cliente activo.
7. **Indaws (id=36) tiene `country = NULL`** → no sale en mapas filtrados por `country='ES'`. Consultar su fila aparte. Además "Indaws" matchea 2 registros (36 y 3112).
8. **Resultados capados ~1000 filas** (`truncated: true`) → paginar con `OFFSET` o estrechar filtro.
9. **Una sola query por llamada** — el servidor rechaza multi-statement. Fichas 360 = varias llamadas (ver partner-detail).

## Recetas disponibles

Detalle completo con el SQL listo para ejecutar en [references/aliases.md](references/aliases.md).

| Receta | Qué hace |
|---|---|
| `new-clients` | Clientes verdaderamente nuevos (primera aparición en últimos N meses), con filtros opcionales |
| `partner-customers` | Lista de clientes de un partner concreto |
| `partners-by-country` | Partners activos en un país, con métricas de tamaño |
| `switchers` | Clientes huérfanos que han roto con un partner (`stop_date IS NOT NULL`, sin relación activa) |
| `competitor-ranking` | Top competidores por nº de clientes nuevos captados |
| `partner-detail` | Ficha completa de un partner (clientes, versiones, niveles, retención) |
| `customer-detail` | Ficha completa de un cliente con historial de partners |
| SQL libre | Cualquier otra pregunta → escribe el SELECT directamente |

## Cuándo usar este skill

- "Dame los nuevos clientes de Odoo en España de los últimos 6 meses"
- "¿Qué partners hay en Portugal / Italia / LATAM?"
- "¿Cuántos clientes tiene Adhoc / Dynapps / Indaws?"
- "Quiero los switchers del último año"
- "Top 20 competidores que más nos están quitando clientes"
- "Quién está comprando Odoo en España"
- "Dame un Excel con cuentas Odoo mid-market en Francia"
- "Ficha del partner XYZ" / "ficha del cliente XYZ"
- "Mapa competitivo de España" / "ranking de competidores Odoo"
- Cualquier pregunta sobre el directorio público de partners/clientes de Odoo.

## Cuándo NO usar este skill

- **CRM, ventas, proyectos, facturación de Indaws** → es otro Odoo (el ERP interno). Usa `common-skills:odoo-pilot`.
- **Enriquecer leads con contactos / decisores / emails** → `marketing-skills:lead-prospector`.
- **Crear leads en el CRM de Indaws** → `common-skills:odoo-pilot` + `sales-skills:lead-data-collection`.
- **Battlecards y posicionamiento contra competidores Odoo** (narrativa comercial, no datos) → `marketing-skills:competitive-brief` / `competitive-analysis`. Este skill aporta DATOS (qué clientes captó tal partner); el de marketing aporta NARRATIVA. Combínalos.
- **Búsqueda de empresas fuera del ecosistema Odoo** → búsquedas web / lead-prospector, no esta BD.

## Delegates to

| Skill | Para qué |
|---|---|
| `marketing-skills:lead-prospector` | Enriquecer las cuentas encontradas con contactos/decisores |
| `common-skills:odoo-pilot` | Crear los leads resultantes en el CRM interno |

## Consumed by

| Skill | Para qué |
|---|---|
| `sales-skills:qualification-form` | Tier 3 competitive intel del prospecto |
| `sales-skills:competitive-intelligence` | Datos win/loss y cartera de competidores |
| `marketing-skills:competitive-analysis` | Datos de captación para positioning maps |
| `sales-skills:qualification-preflight` | Tier 3: inteligencia competitiva (cliente Odoo actual, switcher) |

## REFERENCES

- [references/aliases.md](references/aliases.md) — plantillas SQL listas para `scraper_run_query` (las 7 recetas + variantes + recetas compuestas)
- [references/schema.md](references/schema.md) — esquema completo tabla a tabla con columnas y semántica temporal
- `references/shared/odoo-connection.md` — connection patterns (for callers that chain to odoo-pilot)
