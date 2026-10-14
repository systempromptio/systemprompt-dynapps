---
name: indaws-env-monitor
version: 1.1.0
effort: medium
model: opus
description: >-
  Estado del parque de entornos Odoo ya desplegados de clientes inDAWS (solo lectura, hechos ya
  observados): churn/inactividad, licencias Enterprise por caducar, módulos y cores por cliente y
  los más instalados, reparto v18/v19, servidores caídos y candidatos a migrar. Genera la ficha de
  un cliente e informes del parque (entornos, churn, renovaciones, migración, módulos/cores). Usar
  cuando pregunten por el estado del parque de clientes o entornos Odoo. NO para escribir o
  presupuestar código, migrar o instalar módulos, ni recomendar versión a un cliente nuevo.
---

# inDAWS Env Monitor

Inteligencia sobre el **parque de entornos Odoo de clientes**. El master `indaws_databases` mantiene un registro
(`database.env`, un registro por entorno cliente) que el agente de cada cliente alimenta con hechos observados:
versión/edición, módulos instalados, usuarios, personalización (Studio, automations, crons) y señales de actividad.
Esta skill **lee ese registro** y calcula al vuelo lo que el equipo necesita: churn, caducidades, inventarios y
candidatos a migración.

**Filosofía (clave):** el registro guarda solo **hechos crudos** que únicamente el agente puede observar. Las
analíticas derivadas (días de inactividad, estado de actividad, riesgo de churn, tipo de hosting, candidato a
migración, rankings, medias) **no se almacenan**: las calculas tú en cada consulta. No escribas nunca en el registro.

## Cuándo se activa

Preguntas sobre el parque de clientes/entornos: "¿quién está inactivo o en riesgo de churn?", "¿qué licencias
Enterprise caducan pronto?", "¿qué cores inDAWS tiene el cliente X?", "¿qué módulos son los más instalados?",
"¿cuántos clientes siguen en v18?", "¿qué entornos son candidatos a migración?", "¿qué servidores están caídos?",
"¿qué módulo custom es el más grande?", "¿cuánto código custom tiene el cliente X?".

## Informes disponibles

Informes predeterminados con formato fijo (en `templates/`). Si la petición encaja con uno, **lee su plantilla y
síguela**; si no, responde **libre** con el data-dictionary y las recetas. Reglas comunes en `references/report-rules.md`.

| Informe | Plantilla | Disparador típico |
|---|---|---|
| Entornos — resumen ejecutivo | `templates/entornos-resumen.md` | "informe de entornos", "estado general", "resumen del parque" |
| Churn / inactividad | `templates/churn-inactividad.md` | "quién está inactivo", "en riesgo de churn", "dormidos" |
| Renovaciones Enterprise | `templates/renovaciones.md` | "licencias que caducan", "renovaciones" |
| Candidatos a migración | `templates/migracion.md` | "candidatos a migrar", "viables de migrar a la vXX", "quién sigue en v18" |
| Ficha de entorno | `templates/ficha-entorno.md` | "dame todo del cliente X", "ficha de X" |
| Módulos y cores | `templates/modulos-cores.md` | "cores más instalados", "módulos más extendidos" |

Cualquier otra pregunta (filtrar, cruzar, profundizar) → **respuesta libre**. Tras un informe, recuerda al usuario que
puede seguir preguntando.

## Cómo leer el registro

Trabajas en **solo lectura** sobre `database.env` y sus modelos relacionados. La conexión a Odoo **no es asunto de
esta skill** — la resuelve el acceso estándar del ecosistema. Tu trabajo es saber **qué leer e interpretar**.

### Reglas al leer el modelo

1. **Un solo campo inexistente hace fallar TODA la llamada** (`Internal server error`). Pide únicamente campos
   confirmados en `references/data-dictionary.md`; si dudas de uno, pídelo **solo** (limit 1) antes de meterlo en una
   consulta grande.
2. No necesitas introspección en vivo del esquema: está completo en `references/data-dictionary.md`.
3. **El campo de URL es `url`** (no `database_url`); la caducidad es `license_expiration_date` (no `expiration_date`).
4. **`last_activity_date` (+ `env_data['activity']`) es la señal PRINCIPAL de churn:** `max(last_presence,
   last_comment)` = actividad humana real. Úsala directamente. Un entorno aún no auditado por la sonda puede traerla
   `false` → trátalo como "sin dato", no como dormido. Semántica completa en `references/churn-signal.md`.
5. `search_read` devuelve **80 registros por defecto** (el parque son 68 → limit 100 los cubre);
   `database.env.module.version` tiene miles → ordena+limita o pagina. `env_count` solo es ordenable en
   `database.env.module` (en `.version` es computado). Ver `references/shared/batch-limits.md`.
6. Los Many2one vuelven como `[id, "nombre"]`; `database_env_id` y el `display_name` de `database.env` muestran el
   **hostname** del entorno.

## Workflow

1. **Entiende la pregunta**: si encaja con un informe de "Informes disponibles", lee su plantilla y síguela; si no,
   mapéala a uno o más análisis de la tabla de abajo (respuesta libre).
2. **Lee el registro** con `search_read` usando solo campos del data dictionary y los dominios/recetas de
   `references/query-recipes.md`. Trae de una vez todos los entornos relevantes (no uno a uno).
3. **Calcula las métricas al vuelo** con los umbrales de `references/metrics-and-thresholds.md` (fecha de hoy del
   contexto de sesión; no la inventes).
4. **Responde** de forma accionable: agrega, rankea, agrupa; nombra los entornos por su hostname; cuantifica
   (cuántos, cuántos días, qué fecha). Señala las limitaciones de los datos cuando las haya (p. ej. entornos sin
   `last_activity_date` aún → "sin dato").

## Análisis que soporta

| Análisis | Señal principal | Cómo |
|---|---|---|
| Churn / inactividad | `last_activity_date` (ORO) | días desde la última actividad humana; umbrales 30/60/90d (ver churn-signal.md) |
| Caducidad de licencias | `license_expiration_date` (solo Enterprise) | caducadas + las que vencen ≤60d |
| Inventario de módulos | `database.env.module.version` | qué módulos/versión por entorno; ranking por `env_count` |
| Cores inDAWS instalados | `category_id` del módulo | módulos en categorías inDAWS Core / Core Enterprise / Connectors / Inbuilt |
| Versión y edición | `odoo_version`, `odoo_edition` | distribución v18/v19, community/enterprise |
| Hosting | `url` / hostname | Indaws / Odoo Online·SH / dominio propio (derivado de la URL) |
| Candidatos a migración | `odoo_version` (serie) + footprint | `serie < target` (target = el que pida el usuario o máx. del parque); urgencia por `lag`, complejidad por footprint |
| Retirada SH / recargo renovación | `url` (SH) + serie + `odoo_edition` | SH: fecha en que la versión sale de Odoo.sh; Enterprise en una versión fuera de las 3 más recientes (hoy ≤ v16): candidato a +25% (confirmar renovación con comercial). Política en el SSOT |
| Footprint / complejidad | `env_data`: studio_fields, studio_views, automated_actions, custom_crons | volumen de personalización por entorno (counts) |
| Footprint de código (LOC) | `database.env.module.version.loc_total` | líneas de código de módulos custom/OCA/inDAWS-core (nunca Odoo oficial/Enterprise); una vez por (módulo, versión), no por entorno |
| Seguridad / permisos | `env_data['security_audit']` | grupos y reglas de registro; distingue manual vs Studio (`manual_*`/`studio_*`) |
| Liveness / caídas | `last_heartbeat`, `last_sync_date`, `state` | servidores que dejaron de reportar |
| Tamaño | `user_count`, `module_count` | usuarios y módulos por entorno |

## Gotchas de interpretación

- **Liveness ≠ uso humano.** `last_heartbeat`/`last_sync_date` dicen que el servidor y el agente están vivos, no que
  alguien lo use. El churn se mide con `last_activity_date`. No los confundas en la respuesta.
- **`login_date` de los usuarios llega vacío** en la sincronización actual → no es fiable como señal de actividad.
- Los hosts `*-master-NNN.dev.indaws.cloud` **son producción** (hosting Indaws), no staging. No los descartes.
- El registro tiene un autoregistro del propio master (su `database` = `master` y hostname propio): no lo cuentes como
  cliente si la pregunta es sobre clientes.

## Agent Routing

Corre en **opus**: requiere razonamiento sobre datos heterogéneos, agregación y juicio (qué es churn, qué migrar).
Trabaja en solo lectura sobre el registro; **el acceso a Odoo lo gestiona el ecosistema** (delega en `odoo-pilot`),
no esta skill. Puede alimentar informes (p. ej. una presentación del parque) que deleguen en skills de documento.

## Delegates to

| Skill | Suite | Cuándo |
|---|---|---|
| odoo-pilot | common | Acceso de lectura a Odoo (esta skill no gestiona la conexión) |

## Consumed by

| Skill | Suite | Para qué |
|---|---|---|
| tl-assistant | common | Decisiones de TL sobre el parque (churn, migración, renovaciones) |
| _Skills de documento_ | varias | Bajo demanda, para montar presentaciones/informes del parque a partir de estos datos |

## REFERENCES

| Archivo | Contenido |
|---|---|
| `references/data-dictionary.md` | Arquitectura (acciones/Full Audit), campos de `database.env`, claves de `env_data`, modelos relacionados y campos a tratar con cuidado |
| `references/metrics-and-thresholds.md` | Umbrales de churn, renovaciones, derivación de hosting, categorías de cores, candidatos a migración |
| `references/query-recipes.md` | Llamadas `search_read` concretas para cada análisis |
| `references/churn-signal.md` | Semántica de la señal de actividad (ORO presence+comment) y cómo interpretarla |
| `references/report-rules.md` | Convenciones comunes de los informes (estados, "sin dato", cierre) |
| `templates/*.md` | Formato de salida de cada informe predeterminado |
| `references/shared/batch-limits.md` | Límites de lectura y paginación |
| `references/shared/field-mapping-syntax.md` | Tipos de campo y gotcha de Many2one |
| `references/shared/odoo-lifecycle-licensing.md` | **SSOT** del calendario de retirada de Odoo.sh y de la política del +25% en renovación Enterprise |
