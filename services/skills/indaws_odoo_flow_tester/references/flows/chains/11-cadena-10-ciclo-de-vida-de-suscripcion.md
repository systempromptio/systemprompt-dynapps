## CADENA 10: Ciclo de Vida de Suscripcion
**ID:** CHAIN_SUBSCRIPTION_LIFECYCLE
**Apps:** Suscripciones + Contabilidad
**Descripcion:** Ciclo completo de una suscripcion: creacion, confirmacion, facturacion recurrente, renovacion y cierre.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| sale.order.action_confirm (recurring) | sale.order | account.move (recurrente) | `search_records.sh account.move '[["invoice_origin","=","<SO_name>"]]'` |
| Cierre suscripcion | sale.order | — | Cambia stage a Closed |

### Paso de Variables Entre Flujos
- `{chain.subscription_id}` — ID del sale.order (tipo suscripcion)
- `{chain.subscription_name}` — Nombre del pedido recurrente
- `{chain.template_id}` — ID del sale.order.template usado
- `{chain.invoice_id}` — ID de la factura recurrente generada
- `{chain.recurring_invoice_ids}` — IDs de todas las facturas generadas

### Precondiciones
- Modulo `sale_subscription` instalado
- Producto recurrente configurado (recurring_invoice=True en el producto)
- Plan de recurrencia configurado (sale.recurrence con intervalo y unidad)
- Partner existente

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | SUBSCRIPTION_CREATE | Crear sale.order con recurrence_id + lineas de producto recurrente | state=draft, is_subscription=True |
| 2 | SUBSCRIPTION_CONFIRM | `execute_method.sh sale.order action_confirm '[[<id>]]' '{}'` | state -> 'sale', stage en "In Progress" |
| 3 | SUBSCRIPTION_INVOICE | `execute_method.sh sale.order _create_invoices '[[<id>]]' '{}'` | Factura recurrente creada, state=draft |
| 4 | INVOICE_POST | `execute_method.sh account.move action_post '[[<invoice_id>]]' '{}'` | invoice.state -> 'posted' |
| 5 | SUBSCRIPTION_RENEW | `execute_method.sh sale.order prepare_renewal_order '[[<id>]]' '{}'` | Nuevo sale.order de renovacion creado |
| 6 | SUBSCRIPTION_CLOSE | `update_record.sh sale.order '[<id>]' '{"stage_id": <closed_stage_id>}'` + `execute_method.sh sale.order set_close '[[<id>]]' '{}'` | Suscripcion marcada como cerrada |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | SUBSCRIPTION_REOPEN | Cambiar stage a "In Progress" | `update_record.sh sale.order '[<id>]' '{"stage_id": <in_progress_stage_id>}'` |
| 5 | SUBSCRIPTION_RENEWAL_DELETE | Cancelar + eliminar el SO de renovacion | Solo si renewal en draft/cancel |
| 4 | INVOICE_DRAFT -> CANCEL -> DELETE | button_draft -> button_cancel -> delete | Verificar restricciones hash/audit |
| 3 | (eliminada en paso 4) | | |
| 2 | SALE_CANCEL | action_cancel con contexto disable_cancel_warning | Solo si no hay facturas posted pendientes |
| 1 | SALE_DELETE | delete_record | Solo desde state=cancel o draft |

### Puntos de No Retorno
- **Paso 4 (invoice posted)**: Si journal tiene hash -> no se puede volver a draft
- **Paso 5 (renovacion)**: La renovacion crea un nuevo SO independiente; cancelar la suscripcion original no afecta al renewal
- **Paso 6 (cierre)**: Reversible cambiando stage, pero las facturas generadas persisten

---

