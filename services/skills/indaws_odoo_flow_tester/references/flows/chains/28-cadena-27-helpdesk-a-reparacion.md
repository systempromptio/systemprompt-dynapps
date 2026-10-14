## CADENA 27: Helpdesk a Reparacion
**ID:** CHAIN_HELPDESK_TO_REPAIR
**Apps:** Helpdesk + Reparaciones + Inventario
**Descripcion:** Flujo desde la apertura de un ticket de soporte, diagnostico del problema, creacion de orden de reparacion vinculada, ejecucion de la reparacion y cierre del ticket.
**Tiempo estimado:** 3-5 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| helpdesk.ticket -> repair.order | helpdesk.ticket | repair.order | `search_records.sh repair.order '[["ticket_id","=",<ticket_id>]]'` o campo repair_ids en ticket |
| repair.order.action_repair_end | repair.order | stock.move | `search_records.sh stock.move '[["repair_id","=",<repair_id>]]'` |

### Paso de Variables Entre Flujos
- `{chain.ticket_id}` — ID del helpdesk.ticket
- `{chain.ticket_name}` — Referencia del ticket
- `{chain.team_id}` — ID del equipo de helpdesk
- `{chain.repair_id}` — ID de la repair.order vinculada
- `{chain.partner_id}` — ID del cliente
- `{chain.product_id}` — ID del producto a reparar
- `{chain.stage_new_id}` — ID de etapa "Nuevo"
- `{chain.stage_progress_id}` — ID de etapa "En progreso"
- `{chain.stage_done_id}` — ID de etapa "Resuelto"

### Precondiciones
- Modulo `helpdesk` + `helpdesk_repair` instalados (Enterprise)
- Equipo de helpdesk (helpdesk.team) con feature de reparaciones habilitada
- Etapas de helpdesk configuradas (helpdesk.stage)
- Producto reparable existente
- Piezas de repuesto con stock

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | TICKET_CREATE | Crear helpdesk.ticket con partner_id, team_id, name, description | Ticket creado en etapa inicial |
| 2 | TICKET_DIAGNOSE | `update_record.sh helpdesk.ticket '[<ticket_id>]' '{"stage_id": <stage_progress_id>, "description": "Diagnostico: [detalle]"}'` | Ticket en progreso con diagnostico |
| 3 | REPAIR_CREATE | Crear repair.order con product_id, partner_id, ticket_id (si campo existe) | Reparacion vinculada al ticket |
| 4 | REPAIR_CONFIRM | `execute_method.sh repair.order action_validate '[[<repair_id>]]' '{}'` | state -> 'confirmed' |
| 5 | REPAIR_START | `execute_method.sh repair.order action_repair_start '[[<repair_id>]]' '{}'` | state -> 'under_repair' |
| 6 | REPAIR_END | `execute_method.sh repair.order action_repair_end '[[<repair_id>]]' '{}'` | state -> 'done' |
| 7 | TICKET_CLOSE | `update_record.sh helpdesk.ticket '[<ticket_id>]' '{"stage_id": <stage_done_id>}'` | Ticket en etapa "Resuelto" |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 7 | TICKET_REOPEN | `update_record.sh helpdesk.ticket '[<ticket_id>]' '{"stage_id": <stage_new_id>}'` | Reabrir ticket |
| 6 | **PUNTO DE NO RETORNO PARCIAL** | Reparacion done — movimientos de stock realizados | Ajuste inventario si se necesita revertir stock |
| 5 | (incluido en done) | | |
| 4 | REPAIR_CANCEL | action_repair_cancel | Solo si state != 'done' |
| 3 | REPAIR_DELETE | delete_record repair.order | Solo desde cancel/draft |
| 2 | (update de ticket) | | |
| 1 | TICKET_DELETE | `delete_record.sh helpdesk.ticket '[<ticket_id>]'` | Eliminar ticket |

### Puntos de No Retorno
- **Paso 6 (repair done)**: Movimientos de stock de piezas consumidas son irreversibles sin ajuste manual
- **Nota**: El ticket de helpdesk es siempre reversible (solo cambios de stage). El punto critico es la reparacion vinculada

---

