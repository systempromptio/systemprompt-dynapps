# Mapa de campos Odoo usados por executive-projects-tracker

Fuente: instancia corporativa Indaws (`master` DB en `https://www.indaws.es`, Odoo 19).

## project.project

| Campo | Tipo | Uso |
|---|---|---|
| `id` | int | Clave primaria del JSON local |
| `name` | char | Nombre del proyecto |
| `stage_id` | M2o → `project.project.stage` | Filtrado por whitelist de etapas |
| `partner_id` | M2o → `res.partner` | Cliente |
| `user_id` | M2o → `res.users` | **Responsable** del proyecto (para email) |
| `date_start` | date | Inicio planificado del proyecto |
| `date` | date | **Fecha fin** planificada (en local: `date_end`) |
| `total_hours` | float | Horas presupuestadas (suma de tareas) |
| `total_hours_available` | float | Horas restantes (presupuesto − consumido) |
| `total_unbillable_hours` | float | Horas internas (no facturables) |
| `write_date` | datetime | Última modificación del proyecto |
| `x_studio_fecha_prevista_salida_produccin` | date Studio | **Salida a producción prevista** (sin "ó") |
| `active` | bool | Filtro `active=true` siempre |

> Nota: el campo de fecha fin en Odoo se llama **`date`** (no `date_end`). En los snapshots locales lo renombramos a `date_end` por claridad.

## project.project.stage

| Campo | Uso |
|---|---|
| `id` | Filtro M2o desde `project.project.stage_id` |
| `name` | Comparación literal con whitelist (ES): `Rescue Pack`, `Análisis`, `En proceso`, `Migraciones`, `Salida a producción` |

Las búsquedas usan `context: {"lang": "es_ES"}` para asegurar que `name` viene en español.

## res.users

Para obtener email + nombre del responsable:

| Campo | Uso |
|---|---|
| `id` | Filtrado por `project.user_id` |
| `name` | Mostrar en email |
| `email` | Destinatario (fallback `login` si vacío) |
| `login` | Fallback si `email` no está |

## project.task

Usado para detectar:
- **Tareas vencidas**: `date_deadline < hoy AND remaining_hours > 0`
- **Tareas sin fecha**: `date_deadline IS NULL`
- **Tareas grandes sin checklist**: `allocated_hours > 4` y sin entries en `x_project_indaws_check`
- IDs para localizar checklists asociados

| Campo | Tipo | Uso |
|---|---|---|
| `id` | int | PK |
| `name` | char | Nombre |
| `stage_id` | M2o → `project.task.type` | Etapa de la tarea (mostrada en tablas del email) |
| `planned_date_begin` | datetime | Inicio planificado (mostrado como "Inicio plan." en email) |
| `date_deadline` | datetime | Fin planificado / deadline |
| `allocated_hours` | float | Horas asignadas |
| `effective_hours` | float | Horas consumidas |
| `remaining_hours` | float | Horas restantes |
| `active` | bool | Filtro `active=true` siempre |

> Las tareas cuya `stage_id[1]` esté en `completed_task_stages` (config) se **excluyen** de las alertas. Por defecto: `Realizado`, `Hecho`, `Realizada`, `Done`, `Cancelado`.

## x_project_indaws_check (modelo Studio)

Checklists ponderados por tarea:

| Campo | Uso |
|---|---|
| `x_studio_task_id` | M2o → `project.task` (fields debe pedirlo explícitamente para el mapeo task→checklists) |
| `x_studio_peso` | Peso del item |
| `x_studio_completado` | Boolean: hecho / no hecho |
| `x_studio_hito` | Boolean: marca hito |

Cálculo C%: `Σ(peso si completado) / Σ(peso) × 100`.

Para el formato `89% (11/12)` en las tablas del email se calcula también el conteo de items: `count_done / count_total`.

## project.update

Reportes semanales que el responsable publica al cliente. Últimos 14 días:

| Campo | Tipo | Uso |
|---|---|---|
| `id` | int | PK (usado para buscar mail.message del chatter) |
| `name` | char | Título |
| `date` | date | Fecha del seguimiento |
| `create_date` | datetime | Cuándo se creó |
| `status` | selection | `on_track`, `at_risk`, etc. |
| `user_id` | M2o → `res.users` | Autor |

## mail.message (chatter de project.update)

Para detectar **envío al cliente**:

| Campo | Uso |
|---|---|
| `model` | filtrar `= 'project.update'` |
| `res_id` | filtrar `IN [update_ids]` |
| `subtype_id` | `== 1` (Discussions) — publicación pública |
| `partner_ids` | no vacío — al menos un destinatario externo |
| `date` | timestamp de envío |

**No cuentan** como envío al cliente: `subtype Note` (notas internas), `message_type=notification` (creación automática del registro).

## Campos NO usados (a propósito)

- **`sale.order`, `account.move`**: la facturación se eliminó del modelo (decidido por dirección — administración la lleva por separado).
- `x_studio_etapa` (Studio comercial): se usa la etapa estándar Odoo.
- `tag_ids`: no determinan etapa.
- **Modificaciones a Odoo**: el skill es **read-only**, nunca escribe.
