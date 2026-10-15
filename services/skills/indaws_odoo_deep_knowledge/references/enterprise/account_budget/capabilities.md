<!-- verified: v18.0 @ 2026-06 (account_budget EE; bloqueo-al-exceder negativo; anclas verificadas vs fuente) -->
# Account Budget (`account_budget`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (ABG*).

**ABCAP1. ¿Presupuestos por cuenta analítica?** — **NATIVO** · baja · `budget.analytic` + `budget.line` (hereda `analytic.plan.fields.mixin` → columnas de plan analítico).

**ABCAP2. ¿Líneas por periodo?** — **NATIVO** · baja · `budget.line.date_from`/`date_to` (related store, `_check_date_range`).

**ABCAP3. ¿Presupuestado vs real conseguido?** — **NATIVO** · media · `budget.line.achieved_amount` (= facturado, vía `budget.report`). **Ojo**: el campo es `achieved_amount`, no `practical_amount`.

**ABCAP4. ¿% de cumplimiento?** — **NATIVO** · baja · `budget.line.achieved_percentage` (+ `theoritical_percentage`, `committed_percentage`).

**ABCAP5. ¿Presupuestar por varios planes/ejes analíticos?** — **NATIVO** · media · cada plan genera su columna (`account_id` / `x_plan{id}_id`, `AccountAnalyticPlan._get_all_plans`).

**ABCAP6. ¿Comprometido (committed) vs real?** — **NATIVO** · media · `budget.line.committed_amount` (= facturado + pedidos de compra confirmados); `budget.report` separa committed/achieved.

**ABCAP7. ¿Presupuesto de compras?** — **NATIVO (integrado)** · media · vive **dentro** de `account_budget` (depends `purchase`); **no** existe `account_budget_purchase` aparte en EE18 (`purchase_order_line.is_above_budget`).

**ABCAP8. ¿Estados del presupuesto?** — **NATIVO** · baja · `budget.analytic.state` = `draft`/`confirmed`(Open)/`revised`/`done`/`canceled` (no solo draft/confirmed/done).

**ABCAP9. ¿Generar líneas por mes/trimestre/año?** — **NATIVO** · baja · wizard `budget.split.wizard.action_budget_split` (`period` year/quarter/month).

**ABCAP10. ¿Alerta o bloqueo al superar el presupuesto?** — **NO (solo señalización)** · baja · `budget.line.is_above_budget` / `purchase_order_line.is_above_budget` colorean en rojo (`decoration-danger`); **no** hay `raise`/UserError ni alerta automática al exceder.
