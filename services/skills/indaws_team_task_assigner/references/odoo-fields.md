# Acciones de servidor y campos Odoo

## Acciones de Servidor

IDs varian entre entornos. `snapshot.py` los busca por nombre automaticamente.

| Accion | Modelo | Descripcion |
|--------|--------|-------------|
| Pasar siguiente semana | planning.slot | Copia slot al proximo lunes |
| Planificar (1a Semana) | project.task | Crea slot + stage=Desarrollo + is_planned=true |

Pre-requisitos de "Planificar (1a Semana)":
- `x_studio_plan_user_id` obligatorio
- `x_studio_plan_hours` > 0
- `x_studio_is_planned = false`

## Campos Odoo

| Campo | Modelo | Tipo | Notas |
|-------|--------|------|-------|
| `x_studio_review_priority` | project.task | selection | "3"=Urgente, "2"=Alta, "1"=Media, "0"=Baja |
| `x_studio_fecha_de_revision` | project.task | date | Fecha revision semanal |
| `x_studio_plan_hours` | project.task | float | Horas planificadas |
| `x_studio_plan_user_id` | project.task | m2o (res.users) | Clave para accion servidor |
| `x_studio_is_planned` | project.task | boolean | Skip si true |
| `project_id.xx_team_id` | project.task → project.project | m2o (crm.team) | Equipo. **La tarea NO tiene equipo propio**: `project.task.x_studio_team_id` se eliminó y no hay equivalente en la tarea. El equipo vive en el proyecto (`project.project.xx_team_id`) y se filtra atravesando `project_id`. Consecuencia: una tarea sin `project_id` no puede tener equipo y queda fuera de todo filtro por equipo. |
| `x_studio_done` | planning.slot | boolean | Completado |
| `task_id` | planning.slot | m2o (project.task) | Enlace slot-tarea |
