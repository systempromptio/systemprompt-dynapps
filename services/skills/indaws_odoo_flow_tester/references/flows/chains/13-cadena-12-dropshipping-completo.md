## CADENA 12: Dropshipping Completo
**ID:** CHAIN_DROPSHIP
**Apps:** Ventas + Compras + Inventario
**Descripcion:** Flujo de dropshipping donde una venta genera automaticamente una orden de compra al proveedor, que envia directamente al cliente sin pasar por el almacen propio.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| sale.order.action_confirm (dropship) | sale.order | purchase.order | `search_records.sh purchase.order '[["origin","=","<SO_name>"]]'` |
| sale.order.action_confirm (dropship) | sale.order | stock.picking (tipo dropship) | `search_records.sh stock.picking '[["origin","=","<SO_name>"],["picking_type_code","=","incoming"]]'` |
| purchase.order.button_confirm | purchase.order | stock.picking | `search_records.sh stock.picking '[["origin","=","<PO_name>"]]'` |

### Paso de Variables Entre Flujos
- `{chain.SO_id}` — ID del sale.order
- `{chain.SO_name}` — Nombre del pedido de venta
- `{chain.PO_id}` — ID del purchase.order generado automaticamente
- `{chain.PO_name}` — Nombre de la orden de compra
- `{chain.picking_id}` — ID del picking dropship (supplier -> customer)

### Precondiciones
- Ruta **dropship** configurada: `stock.route` con nombre "Dropship" activa
- Producto con ruta dropship asignada (route_ids incluye la ruta dropship)
- Proveedor configurado en el producto (seller_ids con partner_id del proveedor)
- Partner cliente existente
- Tipo de operacion dropship habilitado en el almacen

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | SALE_CREATE | Crear SO con partner + linea de producto con ruta dropship | state=draft, amount_total>0 |
| 2 | SALE_CONFIRM | `execute_method.sh sale.order action_confirm '[[<id>]]' '{}'` | state -> 'sale' |
| 3 | — detectar PO — | `search_records.sh purchase.order '[["origin","=","<SO_name>"]]'` | PO generado automaticamente para el proveedor del producto |
| 4 | PURCHASE_CONFIRM | `execute_method.sh purchase.order button_confirm '[[<PO_id>]]' '{}'` | PO.state -> 'purchase' |
| 5 | — detectar picking — | `search_records.sh stock.picking '[["origin","=","<PO_name>"]]'` | Picking dropship detectado (supplier -> customer) |
| 6 | PICKING_VALIDATE | `execute_method.sh stock.picking button_validate '[[<picking_id>]]' '{}'` | picking.state -> 'done' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | PICKING_RETURN | Wizard stock.return.picking -> validate return | Crea picking inverso (customer -> supplier) |
| 5 | (limpieza) | Verificar que return picking este done | |
| 4 | PURCHASE_CANCEL | `execute_method.sh purchase.order button_cancel '[[<PO_id>]]' '{}'` | Solo si no hay invoices asociadas |
| 3 | PURCHASE_DELETE | `delete_record.sh purchase.order '[<PO_id>]'` | Solo desde state=cancel |
| 2 | SALE_CANCEL | action_cancel con contexto `{'disable_cancel_warning': True}` | Pickings done requieren devolucion previa |
| 1 | SALE_DELETE | `delete_record.sh sale.order '[<SO_id>]'` | Solo desde state=cancel o draft |

### Puntos de No Retorno
- **Paso 2 (confirm SO)**: La PO se genera automaticamente — cancelar la SO no elimina la PO
- **Paso 6 (picking done)**: No se puede cancelar, solo devolver
- **Nota**: El picking dropship va directamente de proveedor a cliente, sin stock intermedio

---

