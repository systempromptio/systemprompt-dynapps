# project.update y nota de salud — Mapeo

`create_update.py` escribe dos cosas. El LLM no toca ninguna directamente:
vienen de `prepare_update.py` y del JSON de `project-health-monitor`.

## 1. project.update (solo campos estandar)

| Campo Odoo | Origen | Notas |
|---|---|---|
| `project_id` | `metadata.project_id` | ID numerico del proyecto |
| `name` | Generado segun idioma | `"Seguimiento Semanal {ws} - {we}"` (es) o `"Weekly Update {ws} - {we}"` (en) |
| `date` | `metadata.week_end` | Fecha del viernes de la semana reportada |
| `status` | Fijo `"on_track"` | Se mantiene siempre on_track; el cliente ve el progreso, no el estado interno |
| `description` | HTML del LLM | Cuerpo del update en el idioma solicitado |

**Campos eliminados**: los 16 `x_studio_*` de salud de `project.update`
(`x_studio_health_score`, `x_studio_health_status`, `x_studio_checklist_pct`,
`x_studio_timeline_pct`, `x_studio_hours_pct`, `x_studio_delta_ct`,
`x_studio_burn_rate_actual`, `x_studio_burn_rate_expected`, `x_studio_eac`,
`x_studio_acceleration_factor`, `x_studio_last_health_check`,
`x_studio_checklist_progress`, `x_studio_fecha_inicio`, `x_studio_horas`,
`x_studio_tickets_abiertos`, `x_studio_internal_report`) ya no existen en
portal.dynapps.es. Escribirlos da "Invalid field"; `create_update.py` aborta si
aparece cualquier `x_studio_*` en `odoo_values`.

## 2. Nota interna en el chatter de project.project

Generada con `health_note.py` (project-health-monitor) a partir del JSON de
metricas (`health_metrics_path` del JSON preparado):

- Encabezado `🩺 Health check · DD/MM/YYYY` (marca del historico).
- Health Score y estado, escenario, C% (items y peso), T% (dias), H% (horas
  consumidas de presupuesto), delta C-T, burn rate actual y esperado, EAC,
  factor de aceleracion, fecha de evaluacion, top 3 bottlenecks, avisos.
- Variacion respecto a la evaluacion anterior (snapshot local o ultima nota
  `PHM-DATA` del chatter; nunca campos de Odoo).
- "Seguimiento semanal asociado: <nombre> (project.update #id)".
- Informe interno HTML si se paso `--health-report` a `prepare_update.py`.
- Pie `PHM-DATA v1 | ...` parseable para comparar evaluaciones.

Publicacion: `message_post` con `subtype_xmlid='mail.mt_note'`,
`message_type='comment'`, `partner_ids=[]`, sin plantilla y sin anadir
seguidores. Interna: el cliente no la ve y no genera ningun correo.
