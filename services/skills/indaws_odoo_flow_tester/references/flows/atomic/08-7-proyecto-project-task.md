## 7. PROYECTO (project.task)

### TASK_CREATE — Crear Tarea
**Modelo:** project.task
**Campos mínimos:**
```json
{
  "name": "Tarea test",
  "project_id": "<int>"
}
```
**Notas:** `project_id` es opcional pero recomendado.
**Resultado:** state = '01_in_progress', stage_id auto-asignado
**Estados disponibles en project.task (Odoo 18.0):**
- `01_in_progress` — In Progress (default)
- `02_changes_requested` — Changes Requested
- `03_approved` — Approved
- `1_done` — Done (closed)
- `1_canceled` — Cancelled (closed)
- `04_waiting_normal` — Waiting

**⚠️ IMPORTANTE:** `state` y `stage_id` son campos **DIFERENTES**:
- `stage_id` = etapa kanban (configurable por proyecto, visual)
- `state` = estado interno con valores fijos (computed con inverse)
- El campo `state` es **computed** (`_compute_state`): si la tarea tiene dependencias abiertas, se pone automáticamente en `04_waiting_normal`. Al hacer `write({'state': '...'})` funciona vía inverse, pero puede ser sobrescrito por el compute.

### TASK_CHANGE_STAGE — Cambiar Etapa Kanban
**Método:** `update_record.sh project.task '[<id>]' '{"stage_id": <stage_id>}'`
**Restricciones:** stage debe pertenecer al proyecto (domain)

### TASK_CHANGE_STATE — Cambiar Estado Interno
**Método:** `update_record.sh project.task '[<id>]' '{"state": "<state_value>"}'`
**Valores válidos:** `01_in_progress`, `02_changes_requested`, `03_approved`, `1_done`, `1_canceled`, `04_waiting_normal`
**Nota:** Funciona vía el `inverse` del campo computed. Si la tarea tiene dependencias abiertas, el compute puede sobrescribir el estado.

### TASK_DONE — Cerrar Tarea
**Método:** `update_record.sh project.task '[<id>]' '{"state": "1_done"}'`
**Resultado:** state → `1_done` (closed)

### TASK_CANCEL — Cancelar Tarea
**Método:** `update_record.sh project.task '[<id>]' '{"state": "1_canceled"}'`
**Resultado:** state → `1_canceled` (closed)

### TASK_REOPEN — Reabrir Tarea
**Método:** `update_record.sh project.task '[<id>]' '{"state": "01_in_progress"}'`
**Resultado:** state → `01_in_progress`

### TASK_ARCHIVE — Archivar
**Método:** `execute_method.sh project.task toggle_active '[[<id>]]' '{}'`

### TASK_DELETE — Eliminar
**Método:** `delete_record.sh project.task '[<id>]'`
**Efectos:** Incluye todas las subtareas (`_get_all_subtasks()`) en la eliminación. Si la tarea es la última de su recurrencia, elimina también la recurrencia.
**Restricciones:** Ninguna por estado
**Nota:** `project.project.unlink()` elimina TODAS las tareas del proyecto automáticamente.

---

