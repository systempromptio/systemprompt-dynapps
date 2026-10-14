## CADENA 26: Ciclo de Mantenimiento
**ID:** CHAIN_MAINTENANCE_CYCLE
**Apps:** Mantenimiento + Inventario (opcional)
**Descripcion:** Ciclo completo de mantenimiento: solicitud de mantenimiento, asignacion a tecnico, uso de piezas del inventario, completar y registrar log.
**Tiempo estimado:** 1-2 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| maintenance.request (stage done) | maintenance.request | — | stage_id.done == True, close_date rellenada |

### Paso de Variables Entre Flujos
- `{chain.equipment_id}` — ID del equipo (maintenance.equipment)
- `{chain.category_id}` — ID de la categoria de equipo
- `{chain.maintenance_id}` — ID de la solicitud (maintenance.request)
- `{chain.user_id}` — ID del tecnico asignado
- `{chain.stage_new_id}` — ID de la etapa "Nuevo"
- `{chain.stage_progress_id}` — ID de la etapa "En progreso"
- `{chain.stage_done_id}` — ID de la etapa "Completado" (done=True)

### Precondiciones
- Modulo `maintenance` instalado
- Equipo (maintenance.equipment) existente
- Categoria de equipo configurada
- Etapas de mantenimiento (maintenance.stage) configuradas, incluyendo una con done=True
- Tecnico (res.users) asignado

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | EQUIPMENT_VERIFY | Verificar maintenance.equipment existente | equipment_id disponible |
| 2 | MAINTENANCE_CREATE | Crear maintenance.request con equipment_id, maintenance_type='corrective', user_id (tecnico) | Solicitud creada en stage inicial |
| 3 | MAINTENANCE_ASSIGN | `update_record.sh maintenance.request '[<id>]' '{"user_id": <tecnico_id>, "stage_id": <stage_progress_id>}'` | stage_id -> "En progreso", tecnico asignado |
| 4 | MAINTENANCE_LOG | `update_record.sh maintenance.request '[<id>]' '{"description": "Trabajo realizado: [detalle]"}'` | Descripcion actualizada con log de trabajo |
| 5 | MAINTENANCE_COMPLETE | `update_record.sh maintenance.request '[<id>]' '{"stage_id": <stage_done_id>}'` | stage_id.done == True, close_date automatica |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 5 | MAINTENANCE_REOPEN | `update_record.sh maintenance.request '[<id>]' '{"stage_id": <stage_new_id>}'` | Volver a etapa inicial, close_date se limpia |
| 4 | (log es solo texto) | Limpiar descripcion | |
| 3 | (assignment es solo update) | | |
| 2 | MAINTENANCE_DELETE | `delete_record.sh maintenance.request '[<id>]'` | Eliminar solicitud |
| 1 | (equipo existente) | No eliminar | |

### Puntos de No Retorno
- **Ninguno**: Las solicitudes de mantenimiento son completamente reversibles (no generan movimientos contables ni de stock por defecto)
- **Nota**: Si se integra con modulo de inventario para consumo de piezas, los movimientos de stock SI serian punto de no retorno

---

