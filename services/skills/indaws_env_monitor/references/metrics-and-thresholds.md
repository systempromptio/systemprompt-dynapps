# Métricas y umbrales (cálculo al vuelo)

Ninguna de estas métricas se almacena en el registro: se calculan en cada consulta a partir de los campos crudos.
Usa **la fecha de hoy del contexto de sesión** como referencia temporal; no la inventes.

## Churn / inactividad

Señal ORO: `last_activity_date` (= `max(last_presence, last_comment)` — solo actividad humana real).
`inactivity_days = hoy − last_activity_date`.

| Estado | Rango | Lectura |
|---|---|---|
| Activo | ≤ 30 días | uso humano reciente |
| Idle / a vigilar | 31–60 días | empieza a enfriarse |
| En riesgo | 61–90 días | candidato a churn, avisar |
| Dormido | > 90 días | churn de facto |
| Sin dato | `last_activity_date` = `false` | entorno aún no auditado por la sonda; no concluyas churn |

> Las señales ORO (`last_presence`, `last_comment`) no avanzan sin un humano: un entorno movido solo por
> crons/API/integraciones cuenta como sin uso humano = churn. Detalle en `churn-signal.md`.

## Caducidad de licencias

Solo **Enterprise** (`odoo_edition = "enterprise"`), campo `license_expiration_date`. Son contratos con Odoo.

| Estado | Condición |
|---|---|
| Caducada | `license_expiration_date < hoy` |
| En riesgo | `hoy ≤ license_expiration_date ≤ hoy + 60 días` |
| OK | `> hoy + 60 días` |

Community no tiene caducidad: ignóralo en este análisis.

## Hosting (derivado de `url` / hostname)

| Tipo | Patrón en la URL |
|---|---|
| Indaws | contiene `indaws.cloud` o `indaws.es` |
| Odoo Online / SH | contiene `odoo.com` o `odoo.sh` |
| Dominio propio | cualquier otro dominio |

Nota: los hosts `*-master-NNN.dev.indaws.cloud` **son producción** con hosting Indaws (no staging). El staging real
ya viene filtrado del registro; no apliques filtros de staging adicionales.

## Retirada de Odoo.sh y recargo de renovación Enterprise (política externa)

Política de plataforma/comercial de Odoo. **Fuente de verdad:** `references/shared/odoo-lifecycle-licensing.md`
(no copies aquí las fechas ni la regla del 25%: léelas de allí). Dos señales derivadas por entorno:

**Retirada de Odoo.sh (solo hosting SH).** Para entornos cuyo hosting derivado es "Odoo Online / SH" **y** cuya URL es
`odoo.sh`, cruza la serie mayor con el calendario del SSOT (v14 → 31 oct 2026, v15 → 2027, … v19 → 2031).
`días_hasta_salida = fecha_retirada − hoy`. Es un deadline duro que obliga a migrar de versión o de hosting; refuerza
prioridad y es la palanca para traerlos a hosting Indaws. Entornos en hosting Indaws o dominio propio: no aplica.

**Recargo +25% en renovación (candidato).** Marca como *candidato a +25%* los Enterprise (`odoo_edition = "enterprise"`)
cuya **serie mayor** (la versión: `18.0+e` → 18) esté **fuera de las 3 más recientes** (regla del SSOT; hoy ≤ v16, y v17 entra ~2027). **No** puedes
confirmarlo solo con el registro: la regla depende de la fecha de renovación del contrato
(posterior a julio 2025) y de si es multianual — datos que el registro **no** contiene. Preséntalo como "posible +25%,
confirmar fecha de renovación con comercial" y remite al SSOT. Nunca afirmes el recargo como cerrado.

## Cores inDAWS instalados

"Core" = módulo cuyo `category_id` está en una de estas categorías de `database.env.module.category`:
inDAWS Core (1), inDAWS Core Enterprise (2), inDAWS Connectors (3), Inbuilt (4).
(El resto — Odoo Enterprise 5, Odoo Oficial 6, OCA 7, Terceros 8 — no son cores inDAWS.)

Para "qué cores tiene el cliente X": filtra `database.env.module.version` por ese entorno y por esas categorías.
Para "cores más extendidos": agrupa por `technical_name`/`module_id` y ordena por `env_count`.

## Footprint de código (LOC)

`loc_total` de `database.env.module.version` — líneas de código de cada módulo custom/OCA/inDAWS-core, contadas una
vez por `(technical_name, version)` y compartidas por todos los entornos que la tengan instalada (no es un contador
por cliente). Nunca hay LOC para Community/Enterprise oficial: Cloc los excluye por diseño, no es un dato ausente.

Úsalo para:
- **"¿Qué módulos pesan más en el parque?"** — ranking por `loc_total` (receta "Módulos custom más grandes").
- **Footprint de código de un cliente** — suma de `loc_total` de sus módulos custom (receta correspondiente).
- **Refinar la complejidad de migración** (ver más abajo): es una señal más objetiva que contar campos/vistas Studio,
  porque mide volumen de código real, no número de elementos.

Sin umbrales fijos de LOC todavía (es una señal nueva, 2026-08): repórtalo como cifra absoluta o compáralo entre
entornos, no lo clasifiques en 🔴/🟡/✅ sin que te lo pidan explícitamente.

## Candidatos a migración (agnóstico de versión)

Parsea la **serie mayor** de `odoo_version` = el entero antes del primer punto (`"18.0+e"→18`, `"19.0"→19`,
`"saas~20.1"→20`); omite el entorno si no parsea.

**Versión objetivo (`target`):**
- Si el usuario la indica en la pregunta ("…migrar a la versión 20"), úsala tal cual — aunque ningún entorno esté aún
  en ella.
- Si no la indica, `target = max(serie mayor observada en el parque)` (data-driven; se actualiza solo cuando aparece
  una serie nueva).

**Candidato:** entorno con `serie < target`. `lag = target − serie`. La propuesta es siempre **serie → target**; el
camino (saltos 18→19→20 o directo) lo deciden `migration-assistant` / `odoo-database-analyzer`.

**Urgencia** — ventana de soporte de Odoo = **3** mayores (vigente + 2 anteriores):

| lag | Estado |
|---|---|
| ≥ 3 | 🔴 fuera de soporte / EOL — prioritario (sin parches de seguridad) |
| 2 | 🟠 dos versiones por detrás |
| 1 | 🟡 una por detrás (saldrá de soporte con la siguiente) |
| 0 | ✅ al día — no es candidato |

**Complejidad** (independiente de la versión, orientativa, NO horas): footprint de personalización
`studio_fields + studio_views + automated_actions + custom_crons` → ✅ Baja ≤10 · 🟡 Media 11–50 · 🔴 Alta >50.
Las horas reales las estima `odoo-effort-estimator`. Enterprise con licencia próxima a caducar refuerza la prioridad.

Si el entorno ya tiene `loc_total` calculado en sus módulos custom (footprint de código, ver arriba), añádelo como
**segunda señal de complejidad** junto al conteo de Studio/Automations/Crons — mide volumen de código real, no
número de elementos, y es justo el umbral que ya usáis a ojo en las estimaciones de migración ("más de 500 LOC ⇒
complejo"). Si no está disponible todavía para ese entorno (flota en proceso de rellenarse), usa solo el conteo de
personalización como hasta ahora; no bloquees el informe por su ausencia.

## Liveness (servidor/agente vivo) — distinto de uso humano

- `last_heartbeat` muy antiguo (p. ej. > 7 días) con `state = connected` → servidor probablemente caído.
- `last_sync_status = error` repetido → agente con problemas.
- **No** presentes liveness como "actividad" del cliente: mide que la máquina responde, no que alguien la use.
