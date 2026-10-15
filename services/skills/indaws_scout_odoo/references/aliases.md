# Recetas SQL para `scraper_run_query` — referencia detallada

Cheatsheet operativo. Cada receta documenta: pregunta del usuario que la dispara, la plantilla SQL lista para adaptar, los parámetros con default y semántica, combinaciones útiles, y gotchas. Todo se ejecuta con la tool MCP `odoo-scraper-mcp:scraper_run_query(sql)` — un solo SELECT por llamada, sin credenciales. Esquema completo en [schema.md](schema.md).

## Contents

- Constantes y CASE de tiers
- Recetas: new-clients, partner-customers, partners-by-country, switchers, competitor-ranking, partner-detail, customer-detail, SQL libre
- Catálogo de pistas (pregunta → receta)
- Casos de uso compuestos (3 recetas reales)

Convenciones de las plantillas:

- Los placeholders van como `{{PARAM}}`. Sustitúyelos por valores literales antes de ejecutar (no hay bind params en el MCP). Escapa comillas simples en strings de usuario (`'` → `''`).
- `LIMIT` siempre dentro del SQL. El servidor capa a ~1000 filas; si `truncated: true`, pagina con `OFFSET`.
- Para exportar a CSV/XLSX: ejecuta la query, escribe el JSON resultante a fichero con las file tools.

## Constantes

```
INDAWS_PARTNER_ID = 36
SEED_CUTOFF       = '2025-07-01'
EMPLOYEE_TIERS    = 1-10 | 11-50 | 51-200 | 201-500 | 501-1000 | 1001-5000 | 5000+
```

CASE de mapeo de tiers a rank numérico (úsalo cuando necesites `>=` / `<=` sobre `employee_count`):

```sql
CASE c.employee_count
  WHEN '1-10' THEN 1 WHEN '11-50' THEN 2 WHEN '51-200' THEN 3
  WHEN '201-500' THEN 4 WHEN '501-1000' THEN 5 WHEN '1001-5000' THEN 6
  WHEN '5000+' THEN 7 ELSE 0
END
```

## Tabla resumen

| Receta | Uso típico | Parámetros clave |
|---|---|---|
| `new-clients` | Clientes verdaderamente nuevos por país/periodo | país, meses, excluir partner, tier empleados, industria, never-with-partner |
| `partner-customers` | Cartera de un partner | partner_id / nombre, activos-only, país, since |
| `partners-by-country` | Mapa competitivo de un país | país, level |
| `switchers` | Cuentas que rompieron con su partner (oro comercial) | meses, país, excluir ex-partner |
| `competitor-ranking` | Quién está captando más clientes | país, meses, tier empleados |
| `partner-detail` | Ficha 360 de un partner | partner_id (varias queries) |
| `customer-detail` | Ficha 360 de un cliente | reference o customer_id |
| SQL libre | Pregunta no cubierta | — |

---

## new-clients

**Dispara con:** "Dame los nuevos clientes españoles de los últimos 6 meses", "Cuentas nuevas en Francia mid-market", "Quién ha entrado en Odoo este trimestre en industria X".

**Lógica:** CTE `first_seen` con `MIN(odoo_partner_customers.start_date)` por `customer_id` (porque `odoo_customers.start_date` es la fecha del scraper, no del alta real). Filtra `first_seen >= now() - N meses` y excluye por defecto el seed inicial (`first_seen >= '2025-07-01'`). Por defecto excluye clientes cuyo único partner activo es Indaws (36).

**Plantilla:**

```sql
WITH first_seen AS (
    SELECT customer_id, MIN(start_date) AS first_seen
    FROM odoo_partner_customers
    WHERE start_date IS NOT NULL
    GROUP BY customer_id
),
current_rel AS (
    SELECT DISTINCT ON (pc.customer_id)
        pc.customer_id, pc.partner_id, pc.start_date
    FROM odoo_partner_customers pc
    WHERE pc.stop_date IS NULL
    ORDER BY pc.customer_id, pc.start_date DESC NULLS LAST
)
SELECT
    c.id, c.web_id, c.reference, c.industry, c.employee_count, c.revenue,
    fs.first_seen,
    p.name AS current_partner, p.country AS partner_country
FROM odoo_customers c
JOIN first_seen fs ON fs.customer_id = c.id
LEFT JOIN current_rel cr ON cr.customer_id = c.id
LEFT JOIN odoo_partners p ON p.id = cr.partner_id
WHERE fs.first_seen >= CURRENT_DATE - INTERVAL '{{MONTHS}} months'   -- default 6
  AND fs.first_seen >= '2025-07-01'                                  -- exclude-seed (default ON)
  AND c.country_of_origin = '{{COUNTRY}}'                            -- opcional; QUITAR línea si no aplica
  -- AND c.industry ILIKE '%{{INDUSTRY}}%'                           -- opcional
  -- AND (CASE c.employee_count WHEN '1-10' THEN 1 WHEN '11-50' THEN 2 WHEN '51-200' THEN 3 WHEN '201-500' THEN 4 WHEN '501-1000' THEN 5 WHEN '1001-5000' THEN 6 WHEN '5000+' THEN 7 ELSE 0 END) >= {{MIN_TIER_RANK}}  -- opcional
  -- Excluir Indaws (default ON): el cliente debe tener al menos un partner activo distinto de 36
  AND EXISTS (
      SELECT 1 FROM odoo_partner_customers pc_any
      WHERE pc_any.customer_id = c.id
        AND pc_any.stop_date IS NULL
        AND pc_any.partner_id <> 36
  )
ORDER BY fs.first_seen DESC, c.id DESC
LIMIT {{LIMIT}};   -- default 30
```

**Variantes (añadir/quitar condiciones):**

- Incluir Indaws → eliminar el bloque `AND EXISTS (... <> 36)`.
- Solo clientes de un partner concreto → cambiar `<> 36` por `= {{PARTNER_ID}}`.
- "Cuentas vírgenes" (nunca pasaron por Indaws, ni activas ni cerradas):
  ```sql
  AND NOT EXISTS (
      SELECT 1 FROM odoo_partner_customers pc_never
      WHERE pc_never.customer_id = c.id AND pc_never.partner_id = 36
  )
  ```
- Cooldown ("post luna de miel", dejar madurar N meses):
  ```sql
  AND fs.first_seen <= CURRENT_DATE - INTERVAL '{{COOLDOWN}} months'
  ```
- Incluir seed → eliminar `AND fs.first_seen >= '2025-07-01'`.

**Gotchas:**

- Si subes la ventana por encima de 11 meses, casi todo lo anterior a 2025-07 es seed. Mantén el cutoff salvo análisis arqueológico.
- "Nuevo" = primera vez que el scraper lo asocia a un partner, no fundación de la empresa.
- El país del cliente sale de `country_of_origin`, no de `country`.

---

## partner-customers

**Dispara con:** "¿Cuántos clientes tiene Adhoc?", "Lista de cuentas de Indaws", "Cartera activa de Dynapps".

**Paso previo si solo tienes nombre** — resolver el id:

```sql
SELECT p.id, p.name, p.country,
       (SELECT COUNT(*) FROM odoo_partner_customers pc
        WHERE pc.partner_id = p.id AND pc.stop_date IS NULL) AS active_customers
FROM odoo_partners p
WHERE p.name ILIKE '%{{NOMBRE}}%'
ORDER BY active_customers DESC
LIMIT 10;
```

Si hay >1 candidato, mostrar la lista y pedir al usuario que elija el id. (Gotcha conocido: "Indaws" matchea id=36 `Indaws Business Solutions SL` y id=3112 `INDAWS`.)

**Plantilla principal:**

```sql
SELECT
    c.id, c.reference, c.industry, c.employee_count, c.revenue,
    c.country_of_origin,
    pc.start_date AS relationship_start,
    pc.stop_date  AS relationship_end
FROM odoo_partner_customers pc
JOIN odoo_customers c ON c.id = pc.customer_id
WHERE pc.partner_id = {{PARTNER_ID}}
  AND pc.stop_date IS NULL                       -- active-only (default ON); quitar para histórico
  -- AND c.country_of_origin = '{{COUNTRY}}'     -- opcional
  -- AND pc.start_date >= '{{YYYY-MM-DD}}'       -- opcional (since)
ORDER BY pc.start_date DESC NULLS LAST, c.id DESC
LIMIT {{LIMIT}};
```

**Gotchas:**

- `odoo_customers.country` vacío → filtrar por `country_of_origin`.
- Si el partner viene del seed inicial, el `start_date` de relaciones antiguas será 2025-06.

---

## partners-by-country

**Dispara con:** "¿Qué partners hay en Portugal?", "Mapa competitivo de España", "Top partners en Italia".

**Lógica:** agrega `odoo_partners` con COUNT de clientes activos. "Activo" se interpreta como "partner con ≥ 1 cliente activo" (NO uses `odoo_partners.stop_date` — está poblada para casi todos como timestamp de refresco).

**Plantilla:**

```sql
SELECT * FROM (
    SELECT
        p.id AS partner_id, p.name, p.country,
        COALESCE(act.cnt, 0) AS customers_active,
        p.largest_users, p.average_users, p.customer_retention
    FROM odoo_partners p
    LEFT JOIN (
        SELECT partner_id, COUNT(*) AS cnt
        FROM odoo_partner_customers
        WHERE stop_date IS NULL
        GROUP BY partner_id
    ) act ON act.partner_id = p.id
    WHERE p.country = '{{COUNTRY}}'
) q
WHERE customers_active > 0      -- active-only (default ON); quitar para ver todos
ORDER BY customers_active DESC, name
LIMIT {{LIMIT}};
```

**Variante con nivel (Gold/Silver/Ready)** — añadir dentro del subquery:

```sql
    JOIN LATERAL (
        SELECT lv.level_name
        FROM odoo_partner_levels pl2
        JOIN odoo_levels lv ON lv.id = pl2.level_id
        WHERE pl2.partner_id = p.id
        ORDER BY pl2.scrape_date DESC NULLS LAST
        LIMIT 1
    ) lv ON TRUE
```
y en el WHERE: `AND lv.level_name ILIKE '{{LEVEL}}'`.

**Gotchas:**

- Aquí `country` SÍ se usa: es `odoo_partners.country` (país del partner, bien poblado en general). PERO tiene NULLs — ej. Indaws (id=36) tiene `country = NULL`, así que puede no salir en `country='ES'`. Para "¿dónde está Indaws en el ranking?", añade `OR p.id = 36` o consulta su fila aparte.
- `customer_retention` y `largest_users` vienen del scrape — pueden estar desactualizados unos días.

---

## switchers

**Esto es oro comercial.** Clientes cuya última relación terminó (`stop_date IS NOT NULL`) en los últimos N meses y que NO tienen ninguna relación activa → huérfanos buscando partner.

**Dispara con:** "Clientes que han dejado un partner el último año", "Quién acaba de romper con su Odoo partner en ES", "Switchers de la competencia".

**Plantilla:**

```sql
WITH last_rel_per_customer AS (
    SELECT DISTINCT ON (pc.customer_id)
        pc.customer_id, pc.partner_id, pc.start_date, pc.stop_date
    FROM odoo_partner_customers pc
    ORDER BY pc.customer_id, COALESCE(pc.stop_date, pc.start_date) DESC NULLS LAST
)
SELECT
    c.id, c.reference, c.industry, c.employee_count,
    p.name AS ex_partner,
    last_rel.stop_date AS stopped_at,
    (CURRENT_DATE - last_rel.stop_date) AS days_since_stop,
    c.country_of_origin
FROM last_rel_per_customer last_rel
JOIN odoo_customers c ON c.id = last_rel.customer_id
LEFT JOIN odoo_partners p ON p.id = last_rel.partner_id
WHERE last_rel.stop_date IS NOT NULL
  AND last_rel.stop_date >= CURRENT_DATE - INTERVAL '{{MONTHS}} months'  -- default 12
  AND last_rel.partner_id <> 36          -- excluir ex-clientes de Indaws (default ON)
  -- AND c.country_of_origin = '{{COUNTRY}}'   -- opcional
  -- filtro tier empleados opcional (usar el CASE de mapeo)
  AND NOT EXISTS (
      SELECT 1 FROM odoo_partner_customers pc2
      WHERE pc2.customer_id = c.id AND pc2.stop_date IS NULL
  )
ORDER BY last_rel.stop_date DESC
LIMIT {{LIMIT}};
```

**Variantes:**

- Quién dejó al competidor X concretamente → cambiar `<> 36` por `= {{EX_PARTNER_ID}}`.
- Incluir ex-clientes de Indaws (auditoría interna) → quitar la línea `<> 36`.
- "Moved" (cambiaron a otro partner, no huérfanos) → quitar el `NOT EXISTS` y añadir un JOIN a la relación activa actual.

**Gotchas:**

- Un `stop_date` reciente puede ser ruido del scraper (relación reescrita). Si `stop_date - start_date` < 30 días, sospechar.
- No tener nuevo partner público no implica que no lo tengan. Validar antes de llamar.
- Priorización por `days_since_stop`: 5-30 días = ultra-fresco, 30-90 = ideal, 90+ = probablemente ya están en otro sitio.

---

## competitor-ranking

**Dispara con:** "¿Quién se está llevando más clientes?", "Top partners captando cuentas en ES", "Ranking de competidores".

**Lógica:** por cada cliente "nuevo" (CTE `first_seen` con seed cutoff SIEMPRE aplicado), atribuye la captación al primer partner. COUNT por partner.

**Plantilla:**

```sql
WITH first_seen AS (
    SELECT customer_id, MIN(start_date) AS first_seen
    FROM odoo_partner_customers
    WHERE start_date IS NOT NULL
    GROUP BY customer_id
),
first_partner AS (
    SELECT DISTINCT ON (pc.customer_id)
        pc.customer_id, pc.partner_id
    FROM odoo_partner_customers pc
    WHERE pc.start_date IS NOT NULL
    ORDER BY pc.customer_id, pc.start_date ASC
)
SELECT
    p.id AS partner_id, p.name, p.country AS partner_country,
    COUNT(*) AS customers_captured,
    COUNT(*) FILTER (
        WHERE (CASE c.employee_count
            WHEN '1-10' THEN 1 WHEN '11-50' THEN 2 WHEN '51-200' THEN 3
            WHEN '201-500' THEN 4 WHEN '501-1000' THEN 5 WHEN '1001-5000' THEN 6
            WHEN '5000+' THEN 7 ELSE 0 END) >= 3
    ) AS midmarket_captured
FROM first_partner fp
JOIN first_seen fs ON fs.customer_id = fp.customer_id
JOIN odoo_customers c ON c.id = fp.customer_id
JOIN odoo_partners p ON p.id = fp.partner_id
WHERE fs.first_seen >= CURRENT_DATE - INTERVAL '{{MONTHS}} months'  -- default 6
  AND fs.first_seen >= '2025-07-01'    -- seed cutoff: NO quitar en esta receta
  AND p.id <> 36                       -- excluir Indaws del ranking (default ON)
  -- AND c.country_of_origin = '{{COUNTRY}}'   -- país de los CLIENTES captados
GROUP BY p.id, p.name, p.country
ORDER BY customers_captured DESC, midmarket_captured DESC
LIMIT {{LIMIT}};   -- default 20
```

**Gotchas:**

- El seed cutoff se aplica siempre — evita el sesgo del scrape inicial.
- Captaciones ≠ clientes activos. Mide flujo, no stock.
- "¿Y nosotros dónde estamos?" → quitar `p.id <> 36` y re-ejecutar.

---

## partner-detail

**Dispara con:** "Ficha de Adhoc", "Dame todo sobre el partner X", "Resumen 360 de Dynapps".

Son **6 queries** independientes (una llamada a `scraper_run_query` cada una — el MCP no permite multi-statement). Ejecuta las que apliquen y consolida en una sola respuesta.

```sql
-- 1. Datos básicos
SELECT id, web_id, name, country, start_date, stop_date,
       largest_users, average_users, customer_retention
FROM odoo_partners WHERE id = {{PARTNER_ID}};

-- 2. Counts de clientes
SELECT
    COUNT(*) FILTER (WHERE stop_date IS NULL) AS active,
    COUNT(*) FILTER (WHERE stop_date IS NOT NULL) AS terminated,
    COUNT(*) AS total
FROM odoo_partner_customers WHERE partner_id = {{PARTNER_ID}};

-- 3. Top industrias (clientes activos)
SELECT c.industry, COUNT(*) AS n
FROM odoo_partner_customers pc
JOIN odoo_customers c ON c.id = pc.customer_id
WHERE pc.partner_id = {{PARTNER_ID}} AND pc.stop_date IS NULL
  AND c.industry IS NOT NULL AND c.industry <> ''
GROUP BY c.industry ORDER BY n DESC LIMIT 10;

-- 4. Versiones Odoo certificadas
SELECT v.version_name, pv.expert_count
FROM odoo_partner_versions pv
JOIN odoo_versions v ON v.id = pv.version_id
WHERE pv.partner_id = {{PARTNER_ID}}
ORDER BY v.version_name DESC;

-- 5. Nivel actual + histórico (snapshots)
SELECT lv.level_name, pl.scrape_date
FROM odoo_partner_levels pl
JOIN odoo_levels lv ON lv.id = pl.level_id
WHERE pl.partner_id = {{PARTNER_ID}}
ORDER BY pl.scrape_date DESC NULLS LAST LIMIT 20;

-- 6. Top 10 clientes activos recientes
SELECT c.id, c.reference, c.industry, c.employee_count,
       c.country_of_origin, pc.start_date
FROM odoo_partner_customers pc
JOIN odoo_customers c ON c.id = pc.customer_id
WHERE pc.partner_id = {{PARTNER_ID}} AND pc.stop_date IS NULL
ORDER BY pc.start_date DESC NULLS LAST LIMIT 10;
```

Para la lista COMPLETA de clientes usa la receta `partner-customers`.

---

## customer-detail

**Dispara con:** "Ficha del cliente Mercadona", "Cuéntame todo sobre la empresa X", "Histórico de partners de este cliente".

```sql
-- 1. Resolver por referencia (fuzzy). Si hay >1, mostrar candidatos y pedir id.
SELECT id, reference, industry, employee_count, country_of_origin
FROM odoo_customers
WHERE reference ILIKE '%{{REFERENCIA}}%'
ORDER BY id DESC LIMIT 10;

-- 2. Datos del cliente
SELECT id, web_id, reference, country_of_origin, industry, revenue,
       employee_count, start_date, last_enrichment_date, description
FROM odoo_customers WHERE id = {{CUSTOMER_ID}};

-- 3. Historial de partners
SELECT pc.partner_id, p.name AS partner_name, pc.start_date, pc.stop_date
FROM odoo_partner_customers pc
LEFT JOIN odoo_partners p ON p.id = pc.partner_id
WHERE pc.customer_id = {{CUSTOMER_ID}}
ORDER BY pc.start_date DESC NULLS LAST;
```

**Gotchas:**

- `reference` puede ser nombre corto/comercial, no la razón social. Si el match no es claro, mostrar candidatos.
- `stop_date IS NULL` en el historial = relación activa actual. Si todas tienen stop_date → huérfano.
- `last_enrichment_date` nulo = nunca enriquecido. Útil para detectar cuentas a pasar por lead-prospector.

---

## SQL libre

Para preguntas no cubiertas. Restricciones del servidor MCP (no intentes esquivarlas — es solo lectura by design):

- Solo un `SELECT` / `WITH...SELECT` por llamada. Sin `;` intermedio, sin DDL/DML.
- LIMIT automático si falta; resultados capados (~1000 filas, `truncated: true` lo indica).

**Ejemplos:**

```sql
-- Distribución de clientes activos por industria en ES
SELECT c.industry, COUNT(*) AS n
FROM odoo_customers c
JOIN odoo_partner_customers pc ON pc.customer_id = c.id AND pc.stop_date IS NULL
WHERE c.country_of_origin = 'ES'
GROUP BY c.industry ORDER BY n DESC;

-- Partners ES con retention baja
SELECT name, customer_retention FROM odoo_partners
WHERE country = 'ES' AND customer_retention < 80
ORDER BY customer_retention ASC;

-- Versiones Odoo más comunes entre partners ES
SELECT v.version_name, COUNT(*) AS partners
FROM odoo_partner_versions pv
JOIN odoo_versions v ON v.id = pv.version_id
JOIN odoo_partners p ON p.id = pv.partner_id
WHERE p.country = 'ES' GROUP BY v.version_name ORDER BY partners DESC;

-- Industrias que más crecen en ES últimos 6 meses
WITH first_seen AS (
  SELECT customer_id, MIN(start_date) AS fs
  FROM odoo_partner_customers GROUP BY customer_id
)
SELECT c.industry, COUNT(*) AS new_n
FROM first_seen f
JOIN odoo_customers c ON c.id = f.customer_id
WHERE f.fs > CURRENT_DATE - INTERVAL '6 months'
  AND f.fs > '2025-07-01' AND c.country_of_origin = 'ES'
GROUP BY c.industry ORDER BY new_n DESC LIMIT 15;
```

---

## Catálogo de pistas (qué receta usar según la pregunta)

| Si el usuario pregunta… | Receta | Parámetros clave |
|---|---|---|
| "Nuevos clientes en X país" | `new-clients` | country, months=6 |
| "Cuentas mid-market nuevas" | `new-clients` | tier rank >= 3 (51-200) |
| "Cuentas que nadie de los nuestros ha tocado" | `new-clients` | variante never-with-partner 36 |
| "Quién se está llevando clientes" | `competitor-ranking` | country, months=6 (Indaws ya excluido) |
| "Clientes que han dejado a un partner" | `switchers` | months=12 (solo huérfanos por construcción) |
| "Cuentas de Indaws" | `partner-customers` | partner_id=36 |
| "Cuántos clientes tiene Adhoc" | resolver nombre → `partner-customers` | — |
| "Partners en mi país" | `partners-by-country` | country |
| "Solo partners Gold en ES" | `partners-by-country` | variante LATERAL nivel |
| "Ficha de partner X" | `partner-detail` | 6 queries |
| "Ficha del cliente X" | `customer-detail` | reference o id |
| "Agregación rara / pregunta única" | SQL libre | — |

---

## Casos de uso reales (recetas compuestas)

### Receta 1: Lista mensual de cuentas calientes (España, mid-market)

Objetivo: 50-100 cuentas a las que llamar este mes. Mix de nuevos puros + switchers huérfanos.

1. `new-clients` con country=ES, months=3, tier 11-50 (Indaws ya excluido).
2. `switchers` con country=ES, months=6, tier 11-50.
3. `new-clients` variante never-with-partner=36, months=6, tier 51-200 ("vírgenes").
4. Deduplicar por `customer_id`, exportar a XLSX en outputs, y ofrecer pasar a `marketing-skills:lead-prospector` para enriquecer contactos.

### Receta 2: Análisis competitivo — a quién pegamos primero

1. `competitor-ranking` country=ES, months=6, limit 15.
2. Para el top 3: `partner-detail` (6 queries) + `partner-customers` histórico completo.
3. `switchers` variante con `= {{TOP1_ID}}` para ver quién dejó concretamente al top 1.

### Receta 3: Health check trimestral del directorio

1. Partners ES con retention < 75 (SQL libre, ejemplo arriba).
2. Industrias que más crecen en ES últimos 6 meses (SQL libre, ejemplo arriba).
3. Versiones Odoo más usadas por partners ES (SQL libre, ejemplo arriba) — gap analysis vs lo que cubre Indaws.

Los gotchas transversales (country_of_origin, seed cutoff, una query por llamada, escape de comillas…) viven en la sección GOTCHAS y NEVER/ALWAYS de SKILL.md — no se repiten aquí.
