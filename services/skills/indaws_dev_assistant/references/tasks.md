# Información de Tareas

Documentación técnica para consultas de detalles de tareas.

## Modelos Odoo

### project.task (Tarea)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | Char | Nombre de la tarea |
| `description` | Html | Descripción detallada |
| `project_id` | Many2one | Proyecto |
| `stage_id` | Many2one | Etapa actual |
| `priority` | Selection | Prioridad (0=Normal, 1=Alta) |
| `date_deadline` | Date | Fecha límite |
| `planned_hours` | Float | Horas planificadas |
| `effective_hours` | Float | Horas reales |
| `remaining_hours` | Float | Horas restantes |
| `user_ids` | Many2many | Usuarios asignados |

### Campos Studio

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `x_studio_review_user_id` | Many2one | Usuario que asignó/revisará la tarea |

### mail.message (Chatter)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `model` | Char | Modelo relacionado |
| `res_id` | Integer | ID del registro |
| `body` | Html | Contenido del mensaje |
| `author_id` | Many2one | Autor |
| `date` | Datetime | Fecha |
| `message_type` | Selection | Tipo de mensaje |

---

## Consultas

**Script principal**: `scripts/get_task_details.sh`
**Template**: `templates/task-info.md`

### Detalles de la tarea

```bash
./scripts/get_task_details.sh <task_id>
```

### Detalles con chatter

```bash
./scripts/get_task_details.sh <task_id> true
```

### Tarea actual (desde planificación)

Primero obtener slot actual:
```bash
./scripts/get_planning_slots.sh <employee_id> <week_start> true
```

Extraer `task_id` de `x_studio_task_id[0]` y consultar detalles.

---

## Procesamiento

### Limpiar HTML de descripción

El campo `description` viene en HTML. Extraer solo el texto:
- Eliminar tags HTML
- Convertir `<br>` y `</p>` en saltos de línea
- Eliminar espacios extra

### Filtrar mensajes del chatter

**Incluir:**
- `message_type = 'comment'` → Comentarios de usuarios
- `message_type = 'email'` → Emails recibidos

**Ignorar (mensajes automáticos):**
- Notificaciones de cambio de etapa
- Asignaciones automáticas
- Mensajes del sistema

---

## Notas

- `x_studio_review_user_id` es quien puede dar contexto si falta información
- Si `planned_hours = 0` y `effective_hours > 0`, la tarea no fue estimada
- `remaining_hours` negativo indica que se han excedido las horas planificadas
