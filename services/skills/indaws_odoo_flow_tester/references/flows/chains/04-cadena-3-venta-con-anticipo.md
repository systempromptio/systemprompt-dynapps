## CADENA 3: Venta con Anticipo
**ID:** CHAIN_SALE_ADVANCE_PAYMENT
**Apps:** Ventas + Contabilidad

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | SALE_CREATE | Crear SO | state=draft |
| 2 | SALE_CONFIRM | action_confirm | state=sale |
| 3 | INVOICE_CREATE (anticipo) | Crear factura con % anticipo via sale.advance.payment.wizard | invoice.state=draft |
| 4 | INVOICE_POST | action_post en factura anticipo | state=posted |
| 5 | PAYMENT_CREATE + POST | Pago del anticipo | state=paid |
| 6 | PICKING_VALIDATE | Validar albaran | state=done |
| 7 | INVOICE_CREATE (final) | _create_invoices con final=True | Factura final con descuento anticipo |
| 8 | INVOICE_POST | action_post factura final | state=posted |

### Plan de Rollback
Inverso: 8->1, misma logica que CHAIN_SALE_TO_CASH pero con 2 facturas

### Puntos de No Retorno
- **Paso 4 (invoice anticipo posted)**: Si el diario tiene hash -> no se puede volver a draft. Alternativa: nota de credito
- **Paso 6 (picking done)**: No se puede cancelar, solo devolver
- **Paso 8 (invoice final posted)**: Misma restriccion de hash que paso 4

---

