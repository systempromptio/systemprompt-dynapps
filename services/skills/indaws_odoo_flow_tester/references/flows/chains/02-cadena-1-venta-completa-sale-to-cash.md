## CADENA 1: Venta Completa (Sale-to-Cash)
**ID:** CHAIN_SALE_TO_CASH
**Apps:** Ventas + Inventario + Contabilidad
**Marcado:** [MIG] — Obligatorio en migraciones

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | SALE_CREATE | Crear SO con partner + linea producto | state=draft, amount_total>0 |
| 2 | SALE_CONFIRM | action_confirm | state=sale |
| 3 | — detectar — | Buscar picking: origin=SO.name | picking encontrado |
| 4 | PICKING_VALIDATE | button_validate en picking | picking.state=done |
| 5 | SALE_CREATE_INVOICE | _create_invoices en SO | invoice creada, state=draft |
| 6 | INVOICE_POST | action_post en invoice | invoice.state=posted |
| 7 | PAYMENT_CREATE | Crear pago inbound para partner | payment.state=draft |
| 8 | PAYMENT_POST | action_post en payment | payment.state=in_process/paid |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 8 | PAYMENT_CANCEL | action_cancel | Cancela y borra move |
| 7 | PAYMENT_DELETE | delete_record | Elimina pago |
| 6 | INVOICE_DRAFT | button_draft | Desreconcilia automaticamente |
| 5 | INVOICE_CANCEL + DELETE | button_cancel -> delete_record | Verificar que no tenga hash |
| 4 | PICKING_RETURN | Wizard stock.return.picking -> validate return | Crea picking inverso |
| 3 | — limpiar return — | Verificar que return picking este done | |
| 2 | SALE_CANCEL | action_cancel con contexto disable_cancel_warning | Cancela SO |
| 1 | SALE_DELETE | delete_record | Elimina SO |

### Puntos de No Retorno
- **Paso 4 (picking done)**: No se puede cancelar, solo devolver
- **Paso 6 (invoice posted)**: Si journal tiene hash -> no se puede volver a draft
- **Paso 8 (pago reconciliado)**: Debe desreconciliar antes de cancelar

---

