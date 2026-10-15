## CADENA 39: Helpdesk a Reparacion con Facturacion
**ID:** CHAIN_HELPDESK_TO_REPAIR_INVOICE
**Apps:** Helpdesk + Reparaciones + Inventario + Contabilidad (Enterprise)
**Descripcion:** Flujo completo desde ticket de soporte hasta facturacion: crear ticket, crear reparacion vinculada desde ticket, confirmar reparacion, completar, generar SO/factura desde reparacion, y cerrar ticket. Incluye reversion completa de la cadena.
**Tiempo estimado:** 3-5 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| create repair con ticket_id | helpdesk.ticket | repair.order | `search_records.sh repair.order '[["ticket_id","=",<ticket_id>]]'` |
| repair.action_repair_end | repair.order | stock.move | `search_records.sh stock.move '[["repair_id","=",<repair_id>]]'` |
| repair.action_create_sale_order | repair.order | sale.order | Campo sale_order_id en repair |
| sale.order._create_invoices | sale.order | account.move | `search_records.sh account.move '[["invoice_origin","=",<SO_name>]]'` |

### Paso de Variables Entre Flujos
- `{chain.ticket_id}` — ID del helpdesk.ticket
- `{chain.team_id}` — ID del equipo de helpdesk
- `{chain.repair_id}` — ID de la repair.order vinculada
- `{chain.partner_id}` — ID del cliente
- `{chain.product_id}` — ID del producto a reparar
- `{chain.so_id}` — ID del sale.order generado desde la reparacion
- `{chain.invoice_id}` — ID de la factura generada
- `{chain.stage_done_id}` — ID de etapa de cierre del equipo

### Precondiciones
- Modulos `helpdesk`, `helpdesk_repair` instalados (Enterprise)
- Equipo de helpdesk con `use_product_repairs == True`
- Etapas de helpdesk configuradas (al menos una con fold=True para cierre)
- Producto reparable existente con stock
- Piezas de repuesto disponibles
- Tipo de operacion de reparacion configurado

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | TICKET_CREATE | Crear helpdesk.ticket con partner_id, team_id, name, product_id | Ticket creado, stage_id = etapa inicial |
| 2 | REPAIR_CREATE | Crear repair.order con product_id, partner_id, ticket_id, picking_type_id, move_ids (piezas) | state='draft', ticket_id vinculado |
| 3 | REPAIR_CONFIRM | `execute_method.sh repair.order action_validate '[[<repair_id>]]' '{}'` | state -> 'confirmed' |
| 4 | REPAIR_START | `execute_method.sh repair.order action_repair_start '[[<repair_id>]]' '{}'` | state -> 'under_repair' |
| 5 | REPAIR_END | `execute_method.sh repair.order action_repair_end '[[<repair_id>]]' '{}'` | state -> 'done', moves validados |
| 6 | REPAIR_CREATE_SO | `execute_method.sh repair.order action_create_sale_order '[[<repair_id>]]' '{}'` | sale_order_id creado |
| 7 | SO_CONFIRM | `execute_method.sh sale.order action_confirm '[[<so_id>]]' '{}'` | SO state -> 'sale' |
| 8 | INVOICE_CREATE | `execute_method.sh sale.order _create_invoices '[[<so_id>]]' '{}'` | Factura creada state='draft' |
| 9 | TICKET_CLOSE | `update_record.sh helpdesk.ticket '[<ticket_id>]' '{"stage_id": <stage_done_id>}'` | close_date != False |

### Plan de Rollback (orden inverso)

**Escenario A: Si reparacion NO completada (state != 'done') — rollback completo**

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 9 | TICKET_REOPEN | `update_record.sh helpdesk.ticket '[<ticket_id>]' '{"stage_id": <stage_initial_id>}'` | Reabrir ticket |
| 8 | INVOICE_DELETE | `delete_record.sh account.move '[<invoice_id>]'` | Solo si state='draft' |
| 7 | SO_CANCEL | `execute_method.sh sale.order action_cancel '[[<so_id>]]' '{}' --context '{"disable_cancel_warning": true}'` | |
| 6 | SO_DELETE | `delete_record.sh sale.order '[<so_id>]'` | Solo desde cancel |
| 4 | REPAIR_CANCEL | `execute_method.sh repair.order action_repair_cancel '[[<repair_id>]]' '{}'` | Solo si state != 'done' |
| 3 | REPAIR_DELETE | `delete_record.sh repair.order '[<repair_id>]'` | Solo desde cancel/draft |
| 2 | (incluido en repair delete) | | |
| 1 | TICKET_DELETE | `delete_record.sh helpdesk.ticket '[<ticket_id>]'` | Eliminar ticket |

**Escenario B: Si reparacion completada (state == 'done') — PUNTO DE NO RETORNO en paso 5**

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 9 | TICKET_REOPEN | `update_record.sh helpdesk.ticket '[<ticket_id>]' '{"stage_id": <stage_initial_id>}'` | Reabrir ticket |
| 8 | INVOICE_DELETE | `delete_record.sh account.move '[<invoice_id>]'` | Solo si state='draft' |
| 7 | SO_CANCEL | `execute_method.sh sale.order action_cancel '[[<so_id>]]' '{}' --context '{"disable_cancel_warning": true}'` | |
| 6 | SO_DELETE | `delete_record.sh sale.order '[<so_id>]'` | Solo desde cancel |
| 5 | **PUNTO DE NO RETORNO** | Reparacion done — stock moves irreversibles | Rollback automatizado se detiene aqui |
| — | AJUSTE MANUAL | Ajuste de inventario manual para revertir stock de piezas consumidas | No automatizable via API estandar |
| 1 | TICKET_DELETE | `delete_record.sh helpdesk.ticket '[<ticket_id>]'` | Eliminar ticket (la reparacion done queda en el sistema) |

### Puntos de No Retorno
- **Paso 5 (repair done)**: Movimientos de stock irreversibles sin ajuste manual
- **Paso 8 (invoice posted)**: Si se publica la factura y el diario tiene hash -> irreversible
- **Nota**: La reparacion done con SO y factura vinculados crea dependencias multiples — revertir requiere orden estricto

---

