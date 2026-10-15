## CADENA 34: Venta Adaptativa segun Configuracion de Almacen
**ID:** CHAIN_SALE_ADAPTIVE
**Apps:** Ventas + Inventario + Contabilidad
**Marcado:** [MIG]
**Descripcion:** Cadena de venta que se adapta automaticamente a la configuracion de pasos de entrega del almacen (ship_only, pick_ship, pick_pack_ship). Usa el sistema de **pasos condicionales** para ejecutar solo las validaciones de picking relevantes.
**Tiempo estimado:** 3-5 minutos

### Paso de Variables Entre Flujos
- `{chain.SO_id}` — ID del sale.order
- `{chain.SO_name}` — Nombre del pedido de venta
- `{chain.warehouse_id}` — ID del almacen
- `{chain.delivery_steps}` — Configuracion de pasos del almacen (ship_only|pick_ship|pick_pack_ship)
- `{chain.picking_id}` — ID del picking de entrega (ship_only)
- `{chain.pick_picking_id}` — ID del picking de recoleccion (pick_ship, pick_pack_ship)
- `{chain.pack_picking_id}` — ID del picking de empaquetado (pick_pack_ship)
- `{chain.out_picking_id}` — ID del picking de salida (pick_ship, pick_pack_ship)
- `{chain.invoice_id}` — ID de la factura
- `{chain.payment_id}` — ID del pago

### Precondiciones
- Partner cliente existente
- Producto almacenable con stock disponible
- Almacen con delivery_steps configurado (cualquier valor)
- Diarios de venta y banco configurados

### Flujo de Ejecucion (con pasos condicionales)

| # | Flujo Atomico | Condicion | Accion | Verificacion |
|---|--------------|-----------|--------|-------------|
| 1 | SALE_CREATE | — | Crear SO con partner + linea producto | state=draft |
| 2 | SALE_CONFIRM | — | action_confirm | state=sale |
| 3 | ENV_DETECT | — | Leer `stock.warehouse` para obtener `delivery_steps` y guardar en `{env.warehouse.delivery_steps}` | delivery_steps detectado |
| 3a | PICKING_VALIDATE (ship_only) | `{env.warehouse.delivery_steps}` == `ship_only` | Buscar picking de salida + button_validate | picking.state=done |
| 3b.1 | PICK_VALIDATE (pick_ship) | `{env.warehouse.delivery_steps}` == `pick_ship` | Buscar picking de recoleccion (pick) + button_validate | pick.state=done |
| 3b.2 | OUT_VALIDATE (pick_ship) | `{env.warehouse.delivery_steps}` == `pick_ship` | Buscar picking de salida (out) + button_validate | out.state=done |
| 3c.1 | PICK_VALIDATE (pick_pack_ship) | `{env.warehouse.delivery_steps}` == `pick_pack_ship` | Buscar picking de recoleccion (pick) + button_validate | pick.state=done |
| 3c.2 | PACK_VALIDATE (pick_pack_ship) | `{env.warehouse.delivery_steps}` == `pick_pack_ship` | Buscar picking de empaquetado (pack) + button_validate | pack.state=done |
| 3c.3 | OUT_VALIDATE (pick_pack_ship) | `{env.warehouse.delivery_steps}` == `pick_pack_ship` | Buscar picking de salida (out) + button_validate | out.state=done |
| 4 | INVOICE_CREATE | — | _create_invoices en SO | invoice creada |
| 5 | INVOICE_POST | — | action_post en invoice | invoice.state=posted |

### Ejemplo JSON con condiciones

```json
{
  "flow_id": "CHAIN_SALE_ADAPTIVE",
  "steps": [
    {"step": "1", "action": "create", "model": "sale.order", "save_as": "so", "values": {"partner_id": "{pre.partner_id}", "order_line": [[0, 0, {"product_id": "{pre.product_id}", "product_uom_qty": 1}]]}},
    {"step": "2", "action": "execute", "model": "sale.order", "method": "action_confirm", "ids": ["{result.so}"]},
    {"step": "3", "action": "search", "model": "stock.warehouse", "domain": [["id", "=", "{pre.warehouse_id}"]], "fields": ["id", "delivery_steps"], "save_as": "warehouse"},
    {
      "step": "3a", "action": "execute", "model": "stock.picking", "method": "button_validate",
      "condition": {"field": "{result.warehouse.delivery_steps}", "operator": "==", "value": "ship_only"},
      "ids": ["{result.out_picking}"],
      "comment": "Entrega directa: un solo picking"
    },
    {
      "step": "3b.1", "action": "execute", "model": "stock.picking", "method": "button_validate",
      "condition": {"field": "{result.warehouse.delivery_steps}", "operator": "==", "value": "pick_ship"},
      "ids": ["{result.pick_picking}"],
      "comment": "Dos pasos: validar picking de recoleccion"
    },
    {
      "step": "3b.2", "action": "execute", "model": "stock.picking", "method": "button_validate",
      "condition": {"field": "{result.warehouse.delivery_steps}", "operator": "==", "value": "pick_ship"},
      "ids": ["{result.out_picking}"],
      "comment": "Dos pasos: validar picking de salida"
    },
    {
      "step": "3c.1", "action": "execute", "model": "stock.picking", "method": "button_validate",
      "condition": {"field": "{result.warehouse.delivery_steps}", "operator": "==", "value": "pick_pack_ship"},
      "ids": ["{result.pick_picking}"],
      "comment": "Tres pasos: validar picking de recoleccion"
    },
    {
      "step": "3c.2", "action": "execute", "model": "stock.picking", "method": "button_validate",
      "condition": {"field": "{result.warehouse.delivery_steps}", "operator": "==", "value": "pick_pack_ship"},
      "ids": ["{result.pack_picking}"],
      "comment": "Tres pasos: validar picking de empaquetado"
    },
    {
      "step": "3c.3", "action": "execute", "model": "stock.picking", "method": "button_validate",
      "condition": {"field": "{result.warehouse.delivery_steps}", "operator": "==", "value": "pick_pack_ship"},
      "ids": ["{result.out_picking}"],
      "comment": "Tres pasos: validar picking de salida"
    },
    {"step": "4", "action": "execute", "model": "sale.order", "method": "_create_invoices", "ids": ["{result.so}"], "save_as": "invoice"},
    {"step": "5", "action": "execute", "model": "account.move", "method": "action_post", "ids": ["{result.invoice}"]}
  ]
}
```

### Plan de Rollback
Inverso segun los pasos ejecutados. Los pasos condicionales no ejecutados no necesitan rollback.

### Puntos de No Retorno
- **Pickings done**: Solo devolucion, no cancelacion (aplica a todos los pasos 3*)
- **Invoice posted**: Hash restrictions

---

