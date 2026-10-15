## 35. COMERCIO ELECTRÓNICO — Website Sale (sale.order vía API)

### ECOM_CART_CREATE — Crear Carrito de Compra (vía API)
**Modelo:** sale.order
**Descripción:** Simula la creación de un carrito de e-commerce vía API. En Website Sale, el carrito es un `sale.order` en estado `draft` vinculado a un website. El método `sale_get_order(force_create=True)` del modelo `website` crea el pedido, pero vía API se crea directamente como sale.order con campos de website.
**Campos mínimos:**
```json
{
  "partner_id": "<int>",
  "website_id": "<int>",
  "order_line": [[0, 0, {
    "product_id": "<int>",
    "product_uom_qty": "<float>"
  }]]
}
```
**Notas:**
- `website_id`: ID del website (website.website) — vincula el SO al e-commerce
- `partner_id`: cliente (puede ser portal user o public partner)
- El precio se calcula automáticamente desde la tarifa del website (`pricelist_id`)
- `pricelist_id`: se asigna automáticamente desde el website si no se indica
- `team_id`: se puede asignar desde `website.salesteam_id`
- En producción, los carritos se crean vía `website.sale_get_order()` que gestiona sesión, usuario, tarifa y posición fiscal
- Vía API no se usa sesión web — se crea el SO directamente
**Método/acción:** `create_record.sh sale.order '...'`
**Precondiciones:** Módulo `website_sale` instalado, al menos un website con productos publicados
**Resultado:** state = `draft` — carrito creado
**Verificación:** `state == 'draft'`, `website_id != False`, `amount_total > 0`
**Estrategia de reversión:** `delete_record.sh sale.order '[<id>]'` — solo en estado `draft`

### ECOM_CART_UPDATE — Añadir/Modificar Productos en Carrito
**Modelo:** sale.order
**Descripción:** Añade productos o modifica cantidades en el carrito. En website_sale se usa `_cart_update()`, pero vía API se manipulan las líneas directamente.
**Método vía sale.order (recomendado para API):**
```
execute_method.sh sale.order _cart_update '[[<so_id>]]' '{"product_id": <id>, "add_qty": 1}'
```
**Alternativa directa en líneas:**
```
update_record.sh sale.order.line '[<line_id>]' '{"product_uom_qty": <nueva_qty>}'
```
**Notas:**
- `_cart_update(product_id, add_qty=N)`: añade N unidades (N puede ser negativo para quitar)
- `_cart_update(product_id, set_qty=N)`: establece cantidad exacta a N
- Si `set_qty=0` o resultado ≤ 0, la línea se elimina
- El método `_cart_update` es privado (prefijo `_`) pero funciona vía XML-RPC
- También existe `_cart_update_order_line()` que gestiona la lógica interna de crear/actualizar línea
- El precio se recalcula automáticamente al cambiar la cantidad
**Precondiciones:** SO en estado `draft`, producto publicado y disponible para venta
**Resultado:** Línea actualizada/creada con cantidad indicada
**Verificación:** `order_line` con `product_id == <id>` y `product_uom_qty == <expected>`

### ECOM_CHECKOUT_CONFIRM — Simular Checkout y Confirmar Pedido
**Modelo:** sale.order
**Descripción:** Confirma el pedido de e-commerce. En el frontend se pasa por el flujo de checkout (dirección, envío, pago), pero vía API se confirma directamente.
**Pasos vía API:**
1. **Asignar dirección de envío** (si distinta): `update_record.sh sale.order '[<id>]' '{"partner_shipping_id": <addr_id>}'`
2. **Asignar método de envío** (opcional): `update_record.sh sale.order '[<id>]' '{"carrier_id": <carrier_id>}'`
3. **Confirmar:** `execute_method.sh sale.order action_confirm '[[<id>]]' '{}'`
**Notas:**
- En frontend, la confirmación la gestiona el controlador `/shop/payment/validate` tras el pago exitoso
- Vía API, se omite el paso de pago y se confirma directamente
- Tras confirmar, se generan picking(s) como en una venta normal
- El campo `website_order_line` (compute) filtra líneas que no son de descuento/envío
- `only_services`: campo compute que indica si todas las líneas son servicios (omite envío)
**Precondiciones:** state == 'draft', al menos una línea con producto, partner con dirección
**Resultado:** state → `sale`, picking generado (si hay productos físicos)
**Verificación:** `state == 'sale'`, `picking_ids` no vacío (para productos storable)
**Estrategia de reversión:** Igual que SALE_CANCEL — `action_cancel` con contexto `{'disable_cancel_warning': True}`

### ECOM_DELETE — Eliminar Carrito/Pedido
**Método:** `delete_record.sh sale.order '[<id>]'`
**Restricciones:** SOLO en estado 'draft' o 'cancel' (igual que sale.order estándar)
**Precondición:** state == 'draft' o 'cancel'

### Notas Generales sobre Website Sale vía API
- **Controllers relevantes (NO usados vía API, solo referencia):**
  - `/shop/cart/update` → `cart_update()` en `WebsiteSale` controller
  - `/shop/cart/update_json` → `cart_update_json()` — retorna JSON
  - `/shop/payment/validate` → confirma tras pago
  - `/shop/confirmation` → `shop_payment_confirmation()` — página de confirmación
- **Modelo clave vía API:** Todo se gestiona via `sale.order` y `sale.order.line`
- **Campo `website_id`**: Clave para identificar pedidos e-commerce vs pedidos manuales
- **Campo `cart_quantity`**: Total de unidades en el carrito (computed)
- **Posición fiscal**: Se calcula automáticamente desde partner + dirección de envío
