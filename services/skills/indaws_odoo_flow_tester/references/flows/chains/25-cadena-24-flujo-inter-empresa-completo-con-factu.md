## CADENA 24: Flujo Inter-empresa Completo con Facturacion
**ID:** CHAIN_INTERCOMPANY_FULL
**Apps:** Ventas + Compras + Inventario + Contabilidad + Multi-empresa
**Descripcion:** Flujo inter-empresa extendido con facturacion cruzada: venta en Cia A genera PO en Cia B, recepcion en Cia B, facturacion en ambas empresas.
**Tiempo estimado:** 4-6 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| sale.order.action_confirm (intercompany) | sale.order (Cia A) | purchase.order (Cia B) | `search_records.sh purchase.order '[["origin","=","<SO_name>"],["company_id","=",<company_b_id>]]'` |
| purchase.order.button_confirm (Cia B) | purchase.order | stock.picking | `search_records.sh stock.picking '[["origin","=","<PO_name>"],["company_id","=",<company_b_id>]]'` |
| Factura proveedor auto | sale.order (Cia A) | account.move (Cia B) | `search_records.sh account.move '[["ref","=","<SO_name>"],["company_id","=",<company_b_id>]]'` |

### Paso de Variables Entre Flujos
- `{chain.company_a_id}` — ID de la empresa vendedora
- `{chain.company_b_id}` — ID de la empresa compradora
- `{chain.SO_id}` — ID del sale.order en Empresa A
- `{chain.SO_name}` — Nombre del pedido de venta
- `{chain.PO_id}` — ID del purchase.order en Empresa B
- `{chain.picking_id}` — ID del picking de recepcion en Empresa B
- `{chain.invoice_a_id}` — ID de la factura de venta en Empresa A
- `{chain.invoice_b_id}` — ID de la factura de compra en Empresa B

### Precondiciones
- Configuracion inter-company activa en ambas empresas (rule_type='sale_purchase')
- Partners de cada empresa mapeados como inter-company partner
- Producto disponible en ambas empresas
- Diarios de venta (Cia A) y compra (Cia B) configurados
- Impuestos mapeados entre empresas

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | SALE_CREATE | Crear SO en Empresa A con partner_id de Empresa B | state=draft |
| 2 | SALE_CONFIRM | action_confirm en SO (contexto Cia A) | state -> 'sale', PO generada en Cia B |
| 3 | — detectar PO — | Buscar PO en Cia B con origin=SO.name | PO en Cia B |
| 4 | PURCHASE_CONFIRM | button_confirm en PO (contexto Cia B) | PO.state -> 'purchase' |
| 5 | — detectar picking — | Buscar picking en Cia B | Picking recepcion |
| 6 | PICKING_VALIDATE | button_validate (contexto Cia B) | picking.state -> 'done' |
| 7 | INVOICE_CREATE_A | _create_invoices en SO (contexto Cia A) | Factura de venta en Cia A |
| 8 | INVOICE_POST_A | action_post en factura Cia A | invoice.state -> 'posted' |
| 9 | — detectar factura Cia B — | Buscar account.move tipo in_invoice en Cia B | Factura proveedor auto-generada |
| 10 | INVOICE_POST_B | action_post en factura Cia B | invoice.state -> 'posted' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 10 | INVOICE_B_REVERT | button_draft -> button_cancel -> delete (Cia B) | Verificar hash |
| 9 | (detectada) | | |
| 8 | INVOICE_A_REVERT | button_draft -> button_cancel -> delete (Cia A) | Verificar hash |
| 7 | (eliminada en 8) | | |
| 6 | PICKING_RETURN | return picking + validate return (Cia B) | |
| 5 | (detectado) | | |
| 4 | PURCHASE_CANCEL | button_cancel PO (Cia B) | Solo si no hay invoices |
| 3 | PURCHASE_DELETE | delete PO (Cia B) | Solo desde cancel |
| 2 | SALE_CANCEL | action_cancel SO (Cia A) | Cancelar PO de Cia B PRIMERO |
| 1 | SALE_DELETE | delete SO (Cia A) | Solo desde cancel/draft |

### Puntos de No Retorno
- **Paso 2 (SO confirm)**: PO auto-generada en Cia B — cancelar SO no cancela PO
- **Paso 6 (picking done)**: Solo devolucion
- **Paso 8/10 (invoices posted)**: Hash restrictions aplican por empresa independientemente
- **IMPORTANTE**: Rollback en orden estricto: Cia B primero, Cia A despues

---

