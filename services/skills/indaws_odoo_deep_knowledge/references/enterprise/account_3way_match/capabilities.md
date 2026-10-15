<!-- verified: v18.0 @ 2026-06 (account_3way_match EE; no bloquea pago; anclas verificadas vs fuente) -->
# 3-Way Match (`account_3way_match`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (A3M*).

**A3MCAP1. ¿Conciliación a 3 bandas (PO↔recepción↔factura)?** — **NATIVO** · media · `account.move.line._can_be_paid` compara `qty_invoiced`/`qty_received`/`product_qty` de la PO line y agrega `can_be_paid` en `release_to_pay`.

**A3MCAP2. ¿Estado "Should Be Paid"?** — **NATIVO** · baja · `account.move.release_to_pay_manual` (selection `yes`/`no`/`exception`).

**A3MCAP3. ¿Override manual del estado?** — **NATIVO** · baja · `account.move.force_release_to_pay` ("Force Status") activa el valor manual.

**A3MCAP4. ¿Considera PO y recepción?** — **NATIVO** · media · `_can_be_paid` (`:114`): sin `purchase_line_id` → `exception`; con él, evalúa según `purchase_method` (ordered vs received).

**A3MCAP5. ¿Aviso visual en la factura?** — **NATIVO (parcial)** · baja · muestra "Should Be Paid"/"Force Status" + filtros (`bills_to_pay`, `exception`); **no** hay decoración de color/alerta.

**A3MCAP6. ¿Requiere purchase?** — **NATIVO** · trivial · `__manifest__.py` depende de `purchase`.

**A3MCAP7. ¿Bloquea el pago o solo informa?** — **NO (solo informa)** · baja · no hay `raise`/bloqueo en el flujo de pago; calcula el estado y ofrece filtros, **no** impide registrar el pago.

**A3MCAP8. ¿Recepción parcial?** — **NATIVO** · media · `_can_be_paid_received_qty`/`_ordered_qty` comparan con `float_compare`; recepción parcial → `exception`.

**A3MCAP9. ¿Se recalcula al validar recepción/factura?** — **NATIVO** · media · `@api.depends('purchase_line_id.qty_received','qty_invoiced',…)` + `payment_state` (store).

**A3MCAP10. ¿Configurable por compañía o por factura?** — **POR FACTURA** · baja · el override (`force_release_to_pay`/`release_to_pay_manual`) es por factura; **no** hay ajuste por compañía.
