# Diccionario de datos — registro `indaws_databases`

Esquema del registro maestro (solo lectura), verificado contra el master de prod (**68 registros**: 67 `connected` +
1 `pending`). Pide **solo** campos de estas listas: un campo inexistente hace fallar toda la llamada `search_read`
(`Internal server error`).

## Cómo se alimenta el registro (arquitectura)

El agente `indaws_databases_agent` instalado en cada cliente se registra, manda heartbeats y hace *poll* al master;
el master responde con **acciones firmadas** (`database.env.action`: `remote_code` corre en el cliente + `local_code`
escribe en el registro). Hay **9 acciones** en la única plantilla **"Full Audit"** (`database.env.action.template`),
en dos lotes (`batch_group`):

- **Lote 1:** *Installed modules* → `database.env.module(.version/.category)`; *Internal users* → `database.env.user`
  (modelos relacionales); *Activity probe* → escribe el campo `last_activity_date` **y** la clave `env_data['activity']`;
  *Module line counts* → escribe `loc_total` en `database.env.module.version` (corre después de *Installed modules*
  para que la fila del módulo ya exista).
- **Lote 2 — claves de `env_data`:** *Automation rules* → `automated_actions`; *Studio fields* → `studio_fields`;
  *Studio views* → `studio_views`; *Custom scheduled actions* → `custom_crons`; *Security Groups & Record Rules* →
  `security_audit`.

Por eso **módulos y usuarios se consultan en sus modelos relacionales** (no en `env_data`) y **la personalización en
`env_data`**. La fecha de cada bloque está en su `last_sync`.

## `database.env` — un registro por entorno cliente

### Campos legibles (verificados)

| Campo | Tipo | Significado |
|---|---|---|
| `name` | Char | Nombre del entorno (suele mostrar `[UUID8] <database>`) |
| `display_name` | Char | Hostname del entorno (derivado de `url`) — útil para nombrar |
| `state` | Selection | `draft` / `pending` / `connected` |
| `url` | Char | URL base del Odoo remoto (de aquí se deriva el hosting). **Es `url`, NO `database_url`** |
| `database` | Char | Nombre de la base remota |
| `database_uuid` | Char | UUID de la instancia remota |
| `odoo_version` | Char | p. ej. `18.0+e`, `19.0+e`, `19.0` |
| `odoo_edition` | Selection | `community` / `enterprise` |
| `enterprise_code` | Char | Código de contrato Enterprise (puede ser `false`) |
| `license_expiration_date` | Date | Caducidad de licencia (solo Enterprise) |
| `last_heartbeat` | Datetime | Último heartbeat del agente (liveness de servidor) |
| `last_sync_date` | Datetime | Última sincronización (poll/push/manual) |
| `last_sync_status` | Selection | `success` / `partial` / `error` |
| `last_activity_date` | Datetime | Última actividad **humana** = `max(last_presence, last_comment)`; la escribe la sonda *Activity probe*. `false` si el entorno aún no se auditó. **Base del churn** (ver `churn-signal.md`) |
| `module_count` | Integer | Nº de versiones de módulo instaladas |
| `user_count` | Integer | Nº de usuarios internos sincronizados |
| `partner_id` | Many2one→res.partner | Cliente asociado (con frecuencia `false`) |
| `env_data` | Json | Datos dinámicos por categoría (ver abajo) |
| `module_version_ids` | M2M→database.env.module.version | Módulos instalados |
| `user_ids` | O2M→database.env.user | Usuarios sincronizados |
| `snapshot_ids` | O2M→database.env.snapshot | Históricos |

### Campos a tratar con cuidado

| Campo | Estado |
|---|---|
| `database_url` | **No existe**: el campo correcto es `url`. |
| `expiration_date` | **No existe**: el campo correcto es `license_expiration_date`. |

No necesitas introspección en vivo del esquema (`describe_model`/`fields_get`): está completo en este documento.

### `env_data` (Json) — claves

`env_data` agrupa la personalización y la actividad observadas por entorno. Las claves de tipo lista usan el sobre
estándar `{ "data": [...], "count": int, "last_sync": ISO }`; `activity` y `security_audit` tienen su propia forma.
Lee `env_data` entero (una sola lectura por entorno) y navega las claves.

| Clave | `data` | Metadata (en el dict de la clave) | Para qué |
|---|---|---|---|
| `activity` | **dict**: `last_presence`, `last_comment`, `last_login`, `last_email`, `last_backend_write`, `busiest_model` | `last_activity` (= la fecha ORO) | **Churn humano** — `max(last_presence, last_comment)`. Ver `churn-signal.md` |
| `automated_actions` | **lista** de reglas `base.automation`: `name, model, active, trigger, trigger_fields, filter_domain, filter_pre_domain, actions` | `count` | Automatización / complejidad |
| `studio_fields` | **lista** de campos `x_studio_*`: `name, model, type, store, required, readonly, related, relation, relation_field, compute, depends, domain, selection_ids, description, index, copied` | `count` | Footprint Studio |
| `studio_views` | **lista** de vistas `Odoo Studio:%`: `name, model, type, active, priority, inherit_id, xml_id` | `count` | Footprint Studio |
| `custom_crons` | **lista** de `ir.cron` sin xml-id estándar: `name, model, active, interval, nextcall, xml_id, code` (code[:200]) | `count` | Automatización custom |
| `security_audit` | **dict** `{ groups, rules }`. `rules[i]`: `name, model, model_name, domain, global, groups, source, perm_read/write/create/unlink`. `groups[i]`: `name, source, comment, user_count, users[{name,login}], implied_ids` | **sin `count`**; 7 contadores en el dict: `rules_count, global_rules, groups_count, manual_rules, studio_rules, manual_groups, studio_groups` | Seguridad / permisos (manual vs Studio) |

> En `activity`, solo `last_presence` y `last_comment` son **ORO** y definen `last_activity_date`; el resto
> (`last_login`, `last_email`, `last_backend_write`, `busiest_model`) es **contexto** y no mueve la fecha. Detalle en
> `churn-signal.md`.

## Modelos relacionados (legibles)

### `database.env.module` — catálogo de módulos (un registro por módulo)
`technical_name`, `name`, `author`, `category_id` (Many2one→categoría), `summary`, `website`, `env_count`
(**almacenado** — en cuántos entornos está; úsalo para rankear "más instalados"), `version_ids` (O2M→version).

### `database.env.module.version` — módulo+versión por entorno
`technical_name`, `version`, `name`, `author`, `category_id` (Many2one→categoría), `database_env_ids` (M2M→database.env),
`module_id`, `module_path`, `env_count` (**computado, NO almacenado** → no se puede usar en `order=`; para rankear usa
`database.env.module`), `loc_total` (Integer — líneas de código, ver abajo).

**`loc_total`:** líneas de código contadas con el propio `odoo.tools.cloc` de Odoo (AST para `.py`, sin comentarios en
`.xml`/`.js`/`.css`/`.scss`). Se calcula **una vez por `(technical_name, version)`**, no por entorno: si 40 clientes
comparten `indaws_base 19.0.2.1.0`, es la misma fila y el mismo número para los 40. Gotchas:
- **Solo cuenta código custom/OCA/inDAWS-core.** Cloc excluye a propósito todo lo que vive en la misma ruta que
  `base`/`web`/`web_enterprise`/`theme_common` → Odoo Community y Enterprise oficiales **siempre traen `loc_total = 0`
  o `false`**. No es un dato que falte: es la señal funcionando como se diseñó. No lo interpretes como "sin auditar".
  Para saber si de verdad falta el dato en un módulo custom, cruza con la categoría (1–4 y 7–8 son las que pueden
  tener LOC real; 5–6 nunca lo tendrán).
- **Se rellena una sola vez y no se recalcula.** Si el mismo `(technical_name, version)` ya tiene `loc_total` de un
  sync anterior (de este cliente o de otro), la acción no lo vuelve a tocar aunque el código real haya cambiado sin
  subir la versión del manifest (hotfix sin bump). Es la misma asunción que ya usa el resto del registro (versión =
  identidad del módulo).
- **La flota se rellena de forma gradual.** Es un campo nuevo (2026-08); un entorno puede tener módulos custom
  todavía sin `loc_total` si aún no le ha tocado la acción (heartbeat diario, o pendiente de un push masivo tras el
  despliegue). Trátalo como "sin dato todavía", no como "footprint cero".

### `database.env.module.category` — clasificación por patrón de ruta
`name`, `path_pattern`, `sequence`, `module_count`. Valores reales (id → nombre):
1 inDAWS Core · 2 inDAWS Core Enterprise · 3 inDAWS Connectors · 4 Inbuilt · 5 Odoo Enterprise · 6 Odoo Oficial ·
7 OCA · 8 Terceros.

### `database.env.user` — usuarios sincronizados
`name`, `login`, `login_date` (**suele venir vacío** → no fiable), `database_env_id` (Many2one, muestra hostname).

### `database.env.snapshot` — históricos punto-en-el-tiempo
`snapshot_date`, `snapshot_type` (`sync`/`manual`), `odoo_version`, `odoo_edition`, `enterprise_code`, `env_data`,
`module_list`, `user_list`, `module_count`, `user_count`, `changes_summary`, `has_changes`. Útil para tendencias
(p. ej. evolución de módulos/usuarios), no para el estado actual.

### `database.env.sync.log` — auditoría de sincronizaciones
`sync_type` (`poll`/`push`/`manual`), `status`, `summary`, `duration_seconds`. Se purga a los 90 días.
