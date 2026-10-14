## 23. MANTENIMIENTO (maintenance.request)

### MAINTENANCE_CREATE — Crear Solicitud de Mantenimiento
**Modelo:** maintenance.request
**Descripción:** Crea una solicitud de mantenimiento para un equipo o instalación.
**Campos mínimos:**
```json
{
  "name": "Solicitud mantenimiento test",
  "maintenance_team_id": "<int>"
}
```
**Notas:**
- `name`: asunto de la solicitud (required)
- `maintenance_team_id`: equipo de mantenimiento (required, default: primer equipo)
- `equipment_id`: equipo asociado (maintenance.equipment, opcional)
- `maintenance_type`: "corrective" (Correctivo, default) o "preventive" (Preventivo)
- `user_id`: responsable (res.users)
- `owner_user_id`: solicitante (default: usuario actual)
- `stage_id`: etapa (maintenance.stage, default: primera etapa)
- `schedule_date`: fecha planificada
- `priority`: '0' (Normal), '1', '2', '3' (Muy urgente)
- `kanban_state`: 'normal' (En Progreso), 'blocked' (Bloqueado), 'done' (Listo)
- `company_id`: compañía (required, default)
**Método/acción:** `create_record.sh maintenance.request '...'`
**Precondiciones:** Al menos un equipo de mantenimiento (maintenance.team)
**Resultado:** Solicitud creada en primera etapa
**Verificación:** `name == <valor>`, `stage_id != False`
**Estrategia de reversión:** `delete_record.sh maintenance.request '[<id>]'`
**Registros cascada generados:** Ninguno

### MAINTENANCE_ASSIGN — Asignar Técnico
**Modelo:** maintenance.request
**Descripción:** Asigna un usuario responsable a la solicitud de mantenimiento.
**Método:** `update_record.sh maintenance.request '[<id>]' '{"user_id": <user_id>}'`
**Notas:**
- `user_id`: usuario técnico responsable
- Se puede combinar con cambio de `kanban_state` a 'done' (Ready for next stage)
**Resultado:** `user_id` asignado
**Verificación:** `user_id == <id>`

### MAINTENANCE_DONE — Completar Solicitud de Mantenimiento
**Modelo:** maintenance.request
**Descripción:** Mueve la solicitud a una etapa marcada como "done" (completada).
**Método:** `update_record.sh maintenance.request '[<id>]' '{"stage_id": <done_stage_id>}'`
**Notas:**
- La etapa destino debe tener `done == True` (campo related de maintenance.stage)
- Al mover a una etapa done, se establece `close_date` automáticamente
- Para encontrar la etapa done: `search_records.sh maintenance.stage '[["done","=",true]]' '["id","name"]'`
**Precondiciones:** Solicitud existente, etapa done configurada
**Resultado:** `stage_id` apunta a etapa done, `close_date` establecido
**Verificación:** `done == True`, `close_date != False`
**Estrategia de reversión:** Volver a etapa anterior: `update_record.sh maintenance.request '[<id>]' '{"stage_id": <previous_stage_id>}'`

---

