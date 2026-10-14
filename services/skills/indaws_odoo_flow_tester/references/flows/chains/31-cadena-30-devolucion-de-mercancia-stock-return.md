## CADENA 30: Devolucion de Mercancia (Stock Return)
**ID:** CHAIN_STOCK_RETURN
**Apps:** Ventas + Inventario
**Descripcion:** Flujo completo de venta con devolucion: crear pedido de venta, confirmar, entregar, crear devolucion via wizard, validar devolucion y verificar que el stock se ha restaurado.
**Tiempo estimado:** 2-4 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| sale.order.action_confirm | sale.order | stock.picking | `search_records.sh stock.picking '[["origin","=","<SO_name>"],["picking_type_code","=","outgoing"]]'` |
| stock.return.picking.action_create_returns | stock.return.picking (wizard) | stock.picking (return) | `search_records.sh stock.picking '[["origin","like","Return of <picking_name>"]]'` |

### Paso de Variables Entre Flujos
- `{chain.SO_id}` — ID del sale.order
- `{chain.SO_name}` — Nombre del pedido de venta
- `{chain.picking_out_id}` — ID del picking de salida original
- `{chain.picking_out_name}` — Nombre del albaran de salida
- `{chain.return_picking_id}` — ID del picking de devolucion
- `{chain.product_id}` — ID del producto
- `{chain.quant_qty_before}` — Cantidad en stock antes de la devolucion
- `{chain.quant_qty_after}` — Cantidad en stock despues de la devolucion

### Precondiciones
- Partner cliente existente
- Producto almacenable (type='product') con stock disponible
- Almacen con tipo de operacion de salida configurado

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | SALE_CREATE | Crear sale.order con partner_id + linea de producto | state=draft |
| 2 | SALE_CONFIRM | `execute_method.sh sale.order action_confirm '[[<SO_id>]]' '{}'` | state -> 'sale' |
| 3 | — detectar picking — | `search_records.sh stock.picking '[["origin","=","<SO_name>"],["picking_type_code","=","outgoing"]]'` | Picking de salida detectado |
| 4 | PICKING_VALIDATE | `execute_method.sh stock.picking button_validate '[[<picking_out_id>]]' '{"skip_backorder": true, "picking_ids_not_to_backorder": [<picking_out_id>]}'` | picking.state -> 'done' |
| 5 | STOCK_CHECK_BEFORE | `search_records.sh stock.quant '[["product_id","=",<product_id>],["location_id.usage","=","internal"]]' '["quantity"]'` | Registrar stock actual (reducido tras entrega) |
| 6 | RETURN_CREATE | Crear wizard stock.return.picking con picking_id del albaran original -> `execute_method.sh stock.return.picking action_create_returns '[[<wizard_id>]]' '{}'` | Picking de devolucion creado (state=assigned) |
| 7 | RETURN_VALIDATE | `execute_method.sh stock.picking button_validate '[[<return_picking_id>]]' '{"skip_backorder": true}'` | return_picking.state -> 'done' |
| 8 | STOCK_CHECK_AFTER | `search_records.sh stock.quant '[["product_id","=",<product_id>],["location_id.usage","=","internal"]]' '["quantity"]'` | Stock restaurado: qty_after == qty_before + qty_devuelta |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 8 | (verificacion) | | |
| 7 | RETURN_OF_RETURN | Crear otra devolucion del picking de devolucion | Doble devolucion = volver al estado post-entrega |
| 6 | (wizard ya ejecutado) | | |
| 5 | (verificacion) | | |
| 4 | PICKING_RETURN | El paso 6-7 ya hizo la devolucion del picking original | La devolucion YA es el rollback del paso 4 |
| 3 | (detectado) | | |
| 2 | SALE_CANCEL | `execute_method.sh sale.order action_cancel '[[<SO_id>]]' '{"disable_cancel_warning": true}'` | Solo si todos los pickings estan devueltos |
| 1 | SALE_DELETE | `delete_record.sh sale.order '[<SO_id>]'` | Solo desde cancel/draft |

### Puntos de No Retorno
- **Paso 4 (picking done)**: No se puede cancelar, solo devolver (que es exactamente lo que hace esta cadena)
- **Paso 7 (return done)**: El return picking done tampoco se puede cancelar, pero se puede hacer return del return
- **Nota**: El wizard stock.return.picking usa el metodo `_create_return()` que copia el picking original con cantidades invertidas

---

