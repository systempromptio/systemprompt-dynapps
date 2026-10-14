## CADENA 2: Compra Completa (Purchase-to-Pay)
**ID:** CHAIN_PURCHASE_TO_PAY
**Apps:** Compras + Inventario + Contabilidad
**Marcado:** [MIG]

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | PURCHASE_CREATE | Crear PO con partner + linea producto | state=draft |
| 2 | PURCHASE_CONFIRM | button_confirm | state=purchase |
| 3 | — detectar — | Buscar picking: origin=PO.name | picking encontrado (recepcion) |
| 4 | PICKING_VALIDATE | button_validate en picking recepcion | picking.state=done |
| 5 | INVOICE_CREATE | Crear factura proveedor (in_invoice) con ref=PO.name | invoice.state=draft |
| 6 | INVOICE_POST | action_post | invoice.state=posted |
| 7 | PAYMENT_CREATE | Crear pago outbound para supplier | payment.state=draft |
| 8 | PAYMENT_POST | action_post | payment.state=in_process |

### Plan de Rollback

| # | Accion | Notas |
|---|--------|-------|
| 8 | PAYMENT_CANCEL -> DELETE | |
| 7 | (ya eliminado en 8) | |
| 6a | DESRECONCILIAR PAGO | Si el pago se reconcilio con la factura, ejecutar `remove_move_reconcile` en las account.move.line reconciliadas |
| 6b | INVOICE_DRAFT | `execute_method.sh account.move button_draft '[[<invoice_id>]]' '{}'` — Solo si no tiene inalterable_hash |
| 6c | INVOICE_CANCEL + DELETE | `execute_method.sh account.move button_cancel '[[<invoice_id>]]' '{}'` -> `delete_record.sh account.move '[<invoice_id>]'` |
| 5 | (ya eliminada en 6c) | |
| 4 | PICKING_RETURN -> VALIDATE RETURN | Devolucion a proveedor |
| 3 | (limpieza) | |
| 2 | PURCHASE_CANCEL | Solo si invoices ya canceladas |
| 1 | PURCHASE_DELETE | Solo desde state=cancel |

### Puntos de No Retorno
- **Paso 4 (picking done)**: No se puede cancelar, solo devolver con picking inverso
- **Paso 6 (invoice posted)**: Si el diario tiene hash (inalterable_hash) -> no se puede volver a draft. Alternativa: nota de credito
- **Paso 8 (pago reconciliado)**: Debe desreconciliar antes de cancelar el pago

---

