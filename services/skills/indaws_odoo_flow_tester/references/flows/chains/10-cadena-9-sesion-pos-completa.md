## CADENA 9: Sesion POS Completa
**ID:** CHAIN_POS_SESSION
**Apps:** POS + Contabilidad
**Descripcion:** Ciclo completo de una sesion de punto de venta: apertura de sesion, creacion de pedido POS con pago, y cierre de sesion que genera asientos contables automaticos.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| pos.session.action_pos_session_open | pos.session | — | Sesion en estado opened |
| pos.order create + payment | pos.order | account.bank.statement.line | Lineas de pago en la sesion |
| pos.session.action_pos_session_closing_control | pos.session | account.move | `search_records.sh account.move '[["ref","like","<session_name>"]]'` |

### Paso de Variables Entre Flujos
- `{chain.session_id}` — ID de la pos.session creada
- `{chain.session_name}` — Nombre de la sesion POS
- `{chain.config_id}` — ID del pos.config usado
- `{chain.pos_order_id}` — ID del pos.order creado
- `{chain.pos_order_name}` — Referencia del pedido POS
- `{chain.payment_method_id}` — ID del metodo de pago POS
- `{chain.move_ids}` — IDs de los asientos generados al cerrar

### Precondiciones
- pos.config existente con al menos un metodo de pago configurado
- Producto disponible para venta POS (available_in_pos=True)
- Usuario con acceso al punto de venta

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | POS_SESSION_OPEN | `execute_method.sh pos.session open_ui '[[]]' '{"config_id": <config_id>}'` — buscar sesion abierta para el config_id | session.state == 'opened' |
| 2 | POS_ORDER_CREATE | Crear pos.order con session_id + lineas (product_id, qty, price_unit) | pos.order creado, amount_total > 0 |
| 3 | POS_PAYMENT_CREATE | Crear pos.payment con pos_order_id, payment_method_id, amount | Pago registrado en el pedido |
| 4 | POS_ORDER_VALIDATE | `execute_method.sh pos.order action_pos_order_paid '[[<id>]]' '{}'` | state -> 'paid' |
| 5 | POS_SESSION_CLOSE | `execute_method.sh pos.session action_pos_session_closing_control '[[<session_id>]]' '{}'` | state -> 'closed' |
| 6 | — detectar — | Buscar asientos: `search_records.sh account.move '[["ref","like","<session_name>"]]'` | Asientos contables generados y posted |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | INVOICE_DRAFT -> CANCEL -> DELETE | button_draft -> button_cancel -> delete en asientos generados | Solo si no tienen inalterable_hash |
| 5 | (sesion ya cerrada) | No se puede reabrir una sesion cerrada | **PUNTO DE NO RETORNO** |
| 4 | POS_ORDER_REFUND | Crear pedido de devolucion POS | Genera nuevo pedido con cantidades negativas |
| 3 | (incluido en refund) | El refund genera su propio pago inverso | |
| 2 | POS_ORDER_DELETE | `delete_record.sh pos.order '[<id>]'` | Solo si state == 'draft' (antes de pagar) |
| 1 | POS_SESSION_DELETE | `delete_record.sh pos.session '[<id>]'` | Solo si state == 'new' (nunca abierta) |

### Puntos de No Retorno
- **Paso 4 (order paid)**: Una vez pagado, solo se puede revertir con refund
- **Paso 5 (session closed)**: La sesion cerrada NO se puede reabrir; los asientos contables se generan automaticamente
- **Paso 6 (asientos posted)**: Si journal tiene hash -> no se puede volver a draft

---

