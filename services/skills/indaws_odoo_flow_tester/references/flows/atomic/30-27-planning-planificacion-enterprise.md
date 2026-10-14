## 27. PLANNING — Planificación [Enterprise]

### PLANNING_CREATE — Crear Turno de Planificación
**Modelo:** planning.slot
**Descripción:** Crea un turno (slot) de planificación para un recurso/empleado.
**Campos mínimos:**
```json
{
  "start_datetime": "YYYY-MM-DD HH:MM:SS",
  "end_datetime": "YYYY-MM-DD HH:MM:SS",
  "resource_id": "<int>"
}
```
**Notas:**
- `start_datetime`: inicio del turno (required)
- `end_datetime`: fin del turno (required)
- `resource_id`: recurso asignado (resource.resource) — puede ser empleado o material
- `employee_id`: se computa desde resource_id (si el recurso está vinculado a un empleado)
- `role_id`: rol de planificación (planning.role, opcional)
- `company_id`: compañía (required, se computa)
- `state`: 'draft' (Draft) o 'published' (Published)
- `repeat`: si True, crea recurrencias
- `repeat_interval`, `repeat_unit`, `repeat_until`: configuración de recurrencia
- `allocated_hours`: horas asignadas (se computa desde las fechas)
- `template_id`: plantilla de turno (planning.slot.template)
**Método/acción:** `create_record.sh planning.slot '...'`
**Precondiciones:** Módulo `planning` instalado, recurso/empleado existente
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `start_datetime` y `end_datetime` correctos
**Estrategia de reversión:** `delete_record.sh planning.slot '[<id>]'`
**Registros cascada generados:** planning.slot (N, si repeat=True — crea recurrencias)

### PLANNING_PUBLISH — Publicar Planificación
**Modelo:** planning.slot
**Descripción:** Publica los turnos de planificación y notifica a los empleados.
**Método:** `execute_method.sh planning.slot action_send '[[<ids>]]' '{}'`
**Notas:**
- `action_send` envía la planificación por email a los empleados asignados
- Cambia state a `published`
- Se puede publicar un conjunto de slots a la vez
- Alternativa: usar planning.planning para publicar por periodo: crear planning.planning y ejecutar `action_send`
**Precondiciones:** state == 'draft', resource_id asignado
**Resultado:** state → `published`
**Efectos secundarios:**
- Envía email de notificación a empleados
- Establece `publication_warning = False`
**Verificación:** `state == 'published'`
**Estrategia de reversión:** Despublicar: `execute_method.sh planning.slot action_unpublish '[[<ids>]]' '{}'` — state → `draft`

---

