## 16. SUSCRIPCIONES (sale.order con sale_subscription) [Enterprise]

### SUBSCRIPTION_CREATE — Crear Suscripción
**Modelo:** sale.order (extendido por `sale_subscription`)
**Descripción:** Crea un pedido de venta recurrente (suscripción). En Odoo 18, las suscripciones son `sale.order` con `is_subscription == True`.
**Campos mínimos:**
```json
{
  "partner_id": "<int>",
  "plan_id": "<int>",
  "order_line": [[0, 0, {
    "product_id": "<int>",
    "product_uom_qty": "<float>",
    "price_unit": "<float>"
  }]],
  "start_date": "YYYY-MM-DD",
  "next_invoice_date": "YYYY-MM-DD"
}
```
**Notas:**
- `plan_id`: plan de suscripción (sale.subscription.plan) — define la recurrencia
- `product_id` en las líneas debe tener `recurring_invoice == True`
- `start_date`: inicio del período de suscripción
- `next_invoice_date`: fecha de la próxima factura (>= start_date)
- `is_subscription` se computa automáticamente cuando hay `plan_id` + productos recurrentes
- `subscription_state`: '1_draft' (Quotation) al crear
**Método/acción:** `create_record.sh sale.order '...'`
**Precondiciones:** Módulo `sale_subscription` instalado, plan de suscripción configurado, producto con `recurring_invoice == True`
**Resultado:** state = `draft`, subscription_state = `1_draft`, is_subscription = True
**Verificación:** `is_subscription == True`, `subscription_state == '1_draft'`
**Estrategia de reversión:** `delete_record.sh sale.order '[<id>]'` — solo en estado draft
**Registros cascada generados:** sale.order.line (N)

### SUBSCRIPTION_CONFIRM — Confirmar Suscripción
**Modelo:** sale.order
**Descripción:** Confirma la suscripción, activándola para facturación recurrente.
**Método:** `execute_method.sh sale.order action_confirm '[[<id>]]' '{}'`
**Precondiciones:** state == 'draft', subscription_state == '1_draft', plan_id asignado, al menos una línea recurrente
**Restricciones:**
- Constraint: SO confirmado no puede tener subscription_state '1_draft'
- Debe tener `plan_id` si hay productos recurrentes
- Debe tener productos recurrentes si hay `plan_id`
**Resultado:** state → `sale`, subscription_state → `3_progress` (In Progress)
**Efectos secundarios:**
- Crea sale.order.log para tracking de MRR
- Activa la facturación recurrente automática
**Verificación:** `state == 'sale'`, `subscription_state == '3_progress'`
**Estrategia de reversión:** Cerrar suscripción (ver SUBSCRIPTION_CLOSE)
**Registros cascada generados:** sale.order.log (1)

### SUBSCRIPTION_RENEW — Renovar Suscripción
**Modelo:** sale.order
**Descripción:** Crea un pedido de renovación vinculado a la suscripción actual.
**Método:** `execute_method.sh sale.order prepare_renewal_order '[[<id>]]' '{}'`
**Precondiciones:** subscription_state in ('3_progress', '4_paused', '5_renewed')
**Resultado:** Crea nuevo sale.order con subscription_state = `2_renewal`, vinculado al original vía `subscription_id`
**Efectos secundarios:**
- El nuevo pedido hereda líneas del contrato original
- Al confirmar la renovación, el contrato original pasa a `5_renewed`
**Verificación:** Nuevo SO creado con `subscription_state == '2_renewal'`, `subscription_id == <original_id>`
**Estrategia de reversión:** Cancelar el pedido de renovación: `execute_method.sh sale.order action_cancel '[[<renewal_id>]]' '{}'`
**Registros cascada generados:** sale.order (1 renovación), sale.order.line (N), sale.order.log (1)

### SUBSCRIPTION_CLOSE — Cerrar Suscripción
**Modelo:** sale.order
**Descripción:** Cierra (churn) una suscripción activa.
**Método:** `execute_method.sh sale.order set_close '[[<id>]]' '{"close_reason_id": <int>}'`
**Precondiciones:** subscription_state in ('3_progress', '4_paused')
**Notas:**
- `close_reason_id`: ID de sale.order.close.reason (motivo de cierre)
- Si no se indica close_reason_id, se usan razones por defecto del módulo
**Resultado:** subscription_state → `6_churn` (Churned)
**Efectos secundarios:**
- Registra motivo de cierre en `close_reason_id`
- Genera sale.order.log con el evento de cierre
- Detiene la facturación recurrente
**Verificación:** `subscription_state == '6_churn'`
**Estrategia de reversión:** Reabrir: `execute_method.sh sale.order reopen_order '[[<id>]]' '{}'` (si el método existe)
**Registros cascada generados:** sale.order.log (1)

---

