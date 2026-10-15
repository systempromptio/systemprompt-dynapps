## CADENA 28: Venta Completa desde CRM (CRM to Cash)
**ID:** CHAIN_CRM_TO_CASH
**Apps:** CRM + Ventas + Inventario + Contabilidad
**Descripcion:** Flujo completo end-to-end desde la oportunidad CRM hasta el cobro: oportunidad, presupuesto, confirmacion, entrega, facturacion y pago.
**Tiempo estimado:** 4-6 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| crm.lead.action_sale_quotations_new | crm.lead | sale.order | `search_records.sh sale.order '[["opportunity_id","=",<lead_id>]]'` |
| sale.order.action_confirm | sale.order | stock.picking | `search_records.sh stock.picking '[["origin","=","<SO_name>"]]'` |
| sale.order._create_invoices | sale.order | account.move | `search_records.sh account.move '[["invoice_origin","=","<SO_name>"]]'` |

### Paso de Variables Entre Flujos
- `{chain.lead_id}` — ID de la oportunidad
- `{chain.SO_id}` — ID del sale.order
- `{chain.SO_name}` — Nombre del pedido
- `{chain.picking_id}` — ID del picking
- `{chain.invoice_id}` — ID de la factura
- `{chain.payment_id}` — ID del pago

### Precondiciones
- Pipeline de CRM con stages configuradas
- Producto vendible con stock
- Diarios de venta y banco configurados
- Partner existente

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | CRM_CREATE | Crear crm.lead con partner_id, name, expected_revenue | type=opportunity |
| 2 | CRM_ADVANCE | Mover por stages del pipeline | stage_id actualizado |
| 3 | CRM_QUOTATION | Crear SO desde lead (action_sale_quotations_new o crear SO con opportunity_id) | SO creado vinculado al lead |
| 4 | SO_ADD_LINES | Anadir lineas al SO | amount_total > 0 |
| 5 | SO_CONFIRM | action_confirm | state -> 'sale' |
| 6 | — detectar picking — | Buscar picking via origin | picking encontrado |
| 7 | PICKING_VALIDATE | button_validate | picking.state -> 'done' |
| 8 | INVOICE_CREATE | _create_invoices | invoice creada |
| 9 | INVOICE_POST | action_post | invoice.state -> 'posted' |
| 10 | PAYMENT_CREATE | Crear pago inbound | payment.state -> 'draft' |
| 11 | PAYMENT_POST | action_post | payment.state -> 'in_process'/'paid' |
| 12 | CRM_WON | Marcar lead como ganado | probability -> 100 |

### Plan de Rollback (orden inverso)

| # | Accion | Notas |
|---|--------|-------|
| 12 | Reactivar lead si archivado | |
| 11 | PAYMENT_CANCEL | |
| 10 | PAYMENT_DELETE | |
| 9 | INVOICE_DRAFT | Verificar hash |
| 8 | INVOICE_CANCEL + DELETE | |
| 7 | PICKING_RETURN + VALIDATE RETURN | |
| 6 | (detectado) | |
| 5 | SALE_CANCEL | |
| 4 | (incluido en SO) | |
| 3 | SALE_DELETE | |
| 2 | (stage updates) | |
| 1 | CRM_DELETE | |

### Puntos de No Retorno
- **Paso 7 (picking done)**: Solo devolucion
- **Paso 9 (invoice posted)**: Hash restrictions
- **Paso 11 (pago)**: Desreconciliar antes de cancelar

---

