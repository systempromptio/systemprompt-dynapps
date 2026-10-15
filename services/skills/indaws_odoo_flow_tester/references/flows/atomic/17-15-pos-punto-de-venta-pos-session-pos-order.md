## 15. POS — Punto de Venta (pos.session, pos.order)

### POS_SESSION_OPEN — Abrir Sesión POS
**Modelo:** pos.session
**Descripción:** Crea y abre una sesión de punto de venta.
**Campos mínimos:**
```json
{
  "config_id": "<int>"
}
```
**Notas:**
- `config_id`: configuración del punto de venta
- `user_id`: default el usuario actual
- `name`: se asigna automáticamente ("Session/<sequence>")
**Pasos:**
1. Crear sesión: `create_record.sh pos.session '{"config_id": <id>}'`
2. Abrir: `execute_method.sh pos.session action_pos_session_open '[[<id>]]' '{}'`
**Precondiciones:** config_id sin sesión abierta activa
**Restricciones:**
- Si `cash_control == True`, establece saldo inicial desde la última sesión cerrada
- Solo sesiones en estado `opening_control` se abren
**Resultado:** state → `opened`
**Verificación:** `state == 'opened'`
**Estrategia de reversión:** Cerrar sesión (ver POS_SESSION_CLOSE)
**Registros cascada generados:** Ninguno directamente

### POS_ORDER_CREATE — Crear Pedido POS
**Modelo:** pos.order
**Descripción:** Crea un pedido de punto de venta. Normalmente se crean desde el frontend POS, pero se pueden crear vía API.
**Campos mínimos:**
```json
{
  "session_id": "<int>",
  "partner_id": "<int>",
  "lines": [[0, 0, {
    "product_id": "<int>",
    "qty": "<float>",
    "price_unit": "<float>",
    "price_subtotal": "<float>",
    "price_subtotal_incl": "<float>"
  }]]
}
```
**Notas:**
- `session_id`: sesión POS abierta
- `partner_id`: opcional
- `lines`: líneas del pedido con producto, cantidad y precio
- `amount_total`, `amount_tax`, `amount_paid`: se calculan
- Los pedidos POS normalmente se crean y pagan desde el frontend vía `_process_order`
- state: 'draft' → 'paid' → 'done' → 'invoiced'
**Método/acción:** `create_record.sh pos.order '...'`
**Precondiciones:** Sesión POS en estado `opened`
**Resultado:** state = `draft`
**Verificación:** `state == 'draft'`, `session_id == <id>`
**Estrategia de reversión:** Cancelar y eliminar (solo en draft): `update_record.sh pos.order '[<id>]' '{"state": "cancel"}'` luego `delete_record.sh pos.order '[<id>]'`
**Registros cascada generados:** pos.order.line (N)

### POS_SESSION_CLOSE — Cerrar Sesión POS
**Modelo:** pos.session
**Descripción:** Cierra una sesión POS, validando los pagos y generando los asientos contables.
**Método:** `execute_method.sh pos.session action_pos_session_closing_control '[[<id>]]' '{}'`
**Precondiciones:** state == 'opened', sin pedidos en estado 'draft'
**Restricciones:**
- UserError si hay pedidos en estado 'draft' sin procesar
- UserError si la sesión ya está cerrada
- Si `cash_control == True`: pasa a estado `closing_control` y requiere `action_pos_session_validate` adicional
- Si `cash_control == False`: cierra directamente (ejecuta `action_pos_session_close` internamente)
**Resultado:** state → `closed`
**Efectos secundarios:**
- Genera asientos contables (account.move) para todos los pagos
- Reconcilia pagos con las cuentas correspondientes
- Establece `stop_at` (fecha/hora de cierre)
**Verificación:** `state == 'closed'`, `stop_at != False`
**Estrategia de reversión:** No hay reversión. Las sesiones cerradas no se pueden reabrir.
**⚠️ PUNTO DE NO RETORNO:** Una vez cerrada, genera asientos contables irreversibles.
**Registros cascada generados:** account.move (N), account.bank.statement.line (M)

---

