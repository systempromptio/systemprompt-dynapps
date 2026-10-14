# Template: Información de Tarea

Formato de salida para los detalles de una tarea.

## Información Completa

```
📋 Tarea: [nombre]
**Proyecto:** [proyecto]
**Etapa:** [etapa]
**Prioridad:** Alta/Normal
**Deadline:** [DD/MM/YYYY] o "Sin fecha"

📝 **Descripción:**
[descripción resumida, máx 500 caracteres]

💬 **Contexto del chatter:**
- [autor] ([fecha]): [resumen del mensaje]
- [autor] ([fecha]): [resumen del mensaje]
- ...

⏱️ **Horas:** [HH:MM] planificadas | [HH:MM] realizadas | [HH:MM] restantes
```

### Campos mostrados

| Campo | Origen | Notas |
|-------|--------|-------|
| Nombre | `project.task.name` | |
| Proyecto | `project.task.project_id[1]` | Nombre del proyecto |
| Etapa | `project.task.stage_id[1]` | Nombre de la etapa |
| Prioridad | `project.task.priority` | 0=Normal, 1=Alta |
| Deadline | `project.task.date_deadline` | Formato DD/MM/YYYY |
| Descripción | `project.task.description` | Limpiar HTML, máx 500 chars |
| Horas planificadas | `project.task.planned_hours` | Formato HH:MM |
| Horas realizadas | `project.task.effective_hours` | Formato HH:MM |
| Horas restantes | `project.task.remaining_hours` | Formato HH:MM |

### Procesamiento del Chatter

Origen: `mail.message` con `model = 'project.task'` y `res_id = TASK_ID`

**Incluir:**
- `message_type = 'comment'` → Comentarios de usuarios
- `message_type = 'email'` → Emails recibidos

**Excluir (mensajes automáticos):**
- Notificaciones de cambio de etapa
- Asignaciones automáticas
- Mensajes del sistema

## Información Insuficiente

Cuando la descripción está vacía o es genérica y no hay mensajes relevantes:

```
📋 Tarea: [nombre]
**Proyecto:** [proyecto]

⚠️ Información insuficiente para abordar esta tarea.
- Descripción: [vacía/genérica]
- Chatter: [sin mensajes relevantes]

💡 Contacta a **[x_studio_review_user_id]** para obtener más detalles sobre esta tarea.
```

## Notas

- `x_studio_review_user_id` es quien puede dar contexto si falta información
- Si `planned_hours = 0` y `effective_hours > 0`, la tarea no fue estimada
- `remaining_hours` negativo indica que se han excedido las horas planificadas
