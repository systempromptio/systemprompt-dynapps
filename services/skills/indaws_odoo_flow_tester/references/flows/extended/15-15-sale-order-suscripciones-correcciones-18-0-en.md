## 15. sale.order (Suscripciones) — Correcciones 18.0 (Enterprise)

### Correcciones validadas en tests en vivo (2026-03-10)

#### Campo `recurrence_id` NO existe en 18.0
- El campo `recurrence_id` y el modelo `sale.temporal.recurrence` **NO EXISTEN** en Odoo 18.0.
- En su lugar, usar `plan_id` (Many2one a `sale.subscription.plan`), que define la recurrencia.
- Los productos de suscripcion necesitan `recurring_invoice=True`.
- `is_subscription` se computa automaticamente cuando hay `plan_id` + productos recurrentes.

**Campos correctos para crear suscripcion en 18.0:**
```json
{
  "partner_id": "<int>",
  "plan_id": "<int>",
  "order_line": [[0, 0, {
    "product_id": "<int (recurring_invoice=True)>",
    "product_uom_qty": "<float>",
    "price_unit": "<float>"
  }]],
  "start_date": "YYYY-MM-DD",
  "next_invoice_date": "YYYY-MM-DD"
}
```

---

