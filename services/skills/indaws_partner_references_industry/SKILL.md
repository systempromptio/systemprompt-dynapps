---
name: indaws-partner-references-industry
description: >-
  Rellena el campo 'Industria' vacío de las referencias del portal partner de odoo.com
  (/my/partner/references): investiga cada empresa en internet, elige la mejor opción del
  desplegable y la guarda fila a fila vía Claude in Chrome, verificando que persiste (desempate
  Construcción › Fabricación › Distribución). Usar cuando pidan 'rellena las industrias de las
  referencias', 'popular industrias en odoo.com' o cuando el scheduled task mensual lo dispare. NO
  para el directorio público (scout-odoo) ni el CRM interno (odoo-pilot).
version: 1.1.3
model: sonnet
effort: medium
allowed-tools: [Read, Write, Edit, Bash, Skill, WebFetch, WebSearch, Agent, mcp__Claude_in_Chrome__list_connected_browsers, mcp__Claude_in_Chrome__select_browser, mcp__Claude_in_Chrome__navigate, mcp__Claude_in_Chrome__get_page_text, mcp__Claude_in_Chrome__read_page, mcp__Claude_in_Chrome__javascript_tool, mcp__Claude_in_Chrome__read_network_requests, mcp__Claude_in_Chrome__find]
---

# Partner References — Industry Populator

Completa la **Industria** de las referencias de cliente del portal de partner de
Odoo (`https://www.odoo.com/es_ES/my/partner/references`), **solo las vacias**,
investigando el sector real de cada empresa.

## Importante (leer antes de actuar)

1. **[STOP] Pre-chequeo de navegador.** Es computer use sobre el Chrome local.
   Si no hay navegador conectado o la sesion de odoo.com no esta iniciada,
   **REPORTA y TERMINA** sin tocar nada (ver Paso 0). Nunca introduzcas
   credenciales en formularios — el login lo hace el usuario.
2. **Solo las vacias.** Nunca modifiques una referencia que ya tiene industria.
3. **No borres la web del cliente.** El guardado envia web+industria de la fila;
   cambiando solo la industria, la web ya rellenada se preserva. No la vacies.
4. **`change` por si solo NO persiste** — hay que clicar `button.save` (ver GOTCHAS).

## Credenciales

URL y usuario de odoo.com viven en `common-skills:odoo-com-sync` (no se hace login
por API aqui: se opera sobre la sesion del navegador). Si la sesion no esta
iniciada, pide al usuario que entre en odoo.com en su Chrome.

## Workflow

Tablas y snippets JS completos en [references/snippets.md](references/snippets.md).

- **Paso 0 — Conectar y validar sesion** `[sonnet]` `[STOP]`
  `list_connected_browsers` → `select_browser` → `navigate` a la URL →
  `get_page_text`. Si el titulo no es "Partnership Dashboard | Odoo" o redirige a
  login → abortar y avisar. NO continuar.
- **Paso 1 — Detectar vacias (todas las paginas)** `[sonnet]`
  Recorre `/page/N` con `fetch` same-origin (snippet "scan empties"). Vacia =
  `select[name="industry"].value===""`. Recoge `{page, dataId, name, web}`.
  No hardcodees 10 paginas: lee el ultimo enlace del paginador (snippet "page count").
- **Paso 2 — Investigar y clasificar** `[Agent×N: general-purpose, sonnet]`
  Reparte en lotes de ~6 y lanza agentes EN PARALELO (un mensaje, varias
  `Agent()`). Cada agente: WebSearch + WebFetch (web/eInforma/Iberinform → CNAE),
  mapea a UNA opcion. Pasa el mapa de valores y la regla de desempate. Devuelven
  JSON `[{name, activity, industry_text, industry_value, confidence, source}]`.
  **Antes de escribir**, re-lee las `<option>` reales (snippet) y reconstruye el
  mapa si los `value` difieren.
- **Paso 3 — Aplicar y verificar** `[sonnet]`
  Por cada pagina con vacias: navega → snippet "apply page" (fija select + click
  en `button.save` de la fila, ~1500 ms entre filas) → re-`fetch` y confirma
  `select.value`. El POST de guardado es `/my/partner/references/edit` (200).
- **Paso 4 — Barrido final + informe** `[sonnet]`
  Confirma `stillEmptyCount === 0`. Entrega: nº rellenadas, tabla empresa →
  industria → confianza, lista de BAJA confianza (pedir CIF/web), decisiones de
  desempate, y cualquier fila que no se pudo guardar.

## Regla de desempate (obligatoria)

Ante la duda entre dos industrias: **Construcción y Renovación (6) > Fabricación /
Mantenimiento (3) > Venta al por mayor/menor (7, distribución)**.
- Fabrica **y** distribuye → Fabricación (3). Solo comercializa → 7.
  Mantenimiento/reparación industrial → 3.
- Nombres engañosos ("Seguridad", "Mantenimiento") no determinan el sector: usa
  el CNAE/actividad real.

## NEVER / ALWAYS

**NEVER**
1. Introducir credenciales en formularios de odoo.com — el login lo hace el usuario. *Riesgo: política de credenciales + acción irreversible.*
2. Tocar una referencia que ya tiene industria, o vaciar la web de una fila. *El objetivo es solo completar huecos.*
3. Confiar en que `change` guarda — sin click en `button.save` no persiste (GOTCHA 3).
4. Abortar el lote por un fallo individual — log y continúa (idempotente y resiliente).
5. Hardcodear el nº de páginas ni los `value` del desplegable — léelos en vivo.

**ALWAYS**
1. Pre-chequear navegador/sesión en Paso 0 y abortar limpio si falta.
2. Verificar cada página re-leyéndola tras guardar; barrido final a 0 vacías.
3. Marcar como baja confianza las empresas no identificadas y pedir CIF/web.
4. Investigar en paralelo (Agent×N) — no en serie — para no encarecer la ejecución.

## GOTCHAS

1. **`change` no persiste.** Fijar el `select` y disparar `change` NO guarda. Hay
   que clicar `button.save` (`#save<dataId>`) → `POST /my/partner/references/edit`.
2. **`javascript_tool` no soporta `await` top-level suelto.** Envuelve el código
   async en `(async () => { ... })()` y deja que la herramienta resuelva la promesa.
3. **El visor trunca/bloquea salidas grandes.** JSON largo se corta y strings que
   "parecen cookies" se bloquean. Guarda en `window.__x` y devuelve en trozos / texto corto.
4. **`read_network_requests` empieza a trazar en la 1ª llamada.** Si quieres ver el
   POST de guardado, llámala ANTES del click; lo previo no queda registrado.
5. **Paginador variable.** El nº de páginas crece con el tiempo; no lo hardcodees.
6. **Sesión caducada = redirect silencioso a login.** El Paso 0 lo detecta por el título.

## Agent Routing

| Fase | Modelo | Ejecuta como |
|---|---|---|
| Pasos 0,1,3,4 (navegación, detección, guardado, informe) | sonnet | Principal |
| Paso 2 (investigación + clasificación) | sonnet | Subagentes `general-purpose` en paralelo (~6 empresas/agente) |

## Delegates to

| Skill | Para qué |
|---|---|
| `common-skills:odoo-com-sync` | Fuente de URL/usuario de odoo.com (credenciales del portal) |

## Consumed by

| Consumidor | Para qué |
|---|---|
| scheduled task `monthly-partner-references-industry` | Ejecución mensual desatendida (cron `0 9 1 * *`) |

## Cuándo usar

- "Rellena las industrias de las referencias del partner portal"
- "Popular / completar industria en odoo.com"
- "Partner references industry", "industrias de mis referencias Odoo"
- Disparo del scheduled task mensual.

## Cuándo NO usar

- **Consultar el directorio público de partners/clientes de Odoo** (datos, no escritura) → `sales-skills:scout-odoo`.
- **CRM / ventas / proyectos internos de Indaws** → `common-skills:odoo-pilot` (otro Odoo).
- **Enriquecer leads con contactos/decisores** → `marketing-skills:lead-prospector`.

## REFERENCES

- [references/snippets.md](references/snippets.md) — mapa de valores del desplegable + snippets JS (scan, apply, verify, barrido final, save mechanism, page count).

> Nota: este skill **no** usa `common:references/shared/` (patrones de Odoo API
> ORM/conexión/batch) porque opera por **computer use** sobre el portal web, no
> por la API. La única "conexión" es la sesión del navegador (Paso 0).
