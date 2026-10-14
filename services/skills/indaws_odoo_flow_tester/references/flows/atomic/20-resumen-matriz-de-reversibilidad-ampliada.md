## RESUMEN — Matriz de Reversibilidad (Ampliada)

| Modelo | Cancel | Draft | Delete | Return | Punto de No Retorno |
|--------|--------|-------|--------|--------|---------------------|
| sale.order | ✅ (si no locked) | ✅ (desde cancel/sent) | ✅ (draft/cancel) | N/A | Picking done |
| purchase.order | ✅ (si no hay invoices) | ✅ | ✅ (solo cancel) | N/A | Invoice posted |
| stock.picking | ✅ (si no done) | N/A | ✅ (si moves no done) | ✅ (solo done) | Move done sin return |
| account.move | ✅ | ✅ (si no hash/exchange/caba) | ⚠️ (restricciones — hash NO bloquea unlink) | ✅ (reversal) | inalterable_hash (solo para draft), audit trail |
| account.move.line | N/A | N/A | N/A | ✅ (remove_move_reconcile) | Ninguno |
| account.payment | ✅ | ✅ | ✅ | N/A | Move reconciliado |
| account.payment.register | N/A | N/A | N/A | N/A | Wizard transitorio |
| crm.lead | N/A | N/A | ✅ | N/A | Ninguno |
| project.task | ✅ (1_canceled) | ✅ (01_in_progress) | ✅ (+ subtareas) | N/A | Ninguno |
| hr.expense | ✅ (refuse) | ✅ (reset) | ✅ | N/A | Move posted |
| account.bank.statement | N/A | N/A | ✅ | N/A | Ninguno (solo agrupa) |
| account.bank.statement.line | N/A | N/A | ✅ (audit trail) | ✅ (undo_reconciliation) | Audit trail activo |
| account.asset | N/A | N/A | ✅ (solo draft) | N/A | state == 'close' |
| stock.scrap | N/A | N/A | ⚠️ (solo draft) | N/A | state == 'done' |
| stock.quant (ajuste) | N/A | N/A | N/A | N/A | Ninguno (re-ajustar) |
| stock.lot | N/A | N/A | ✅ (sin quants) | N/A | Quants asociados |
| stock.warehouse.orderpoint | N/A | N/A | ✅ | N/A | Ninguno |
| mrp.unbuild | N/A | N/A | ⚠️ (solo draft) | N/A | state == 'done' |
| mrp.bom | N/A | N/A | ✅ | N/A | Ninguno |
| mrp.workorder | N/A | N/A | N/A | N/A | state == 'done' |
| hr.payslip | ✅ (solo admin en done) | ✅ | ✅ (solo draft) | N/A | Move posted |
| hr.contract | ✅ (cancel) | ✅ (manual) | ⚠️ (work entries) | N/A | state='open' auto-genera work entries validados |
| hr.attendance | N/A | N/A | ✅ | N/A | Ninguno |
| pos.session | N/A | N/A | N/A | N/A | state == 'closed' |
| pos.order | ✅ | N/A | ✅ (solo draft/cancel) | N/A | state == 'done' |
| sale.order (suscripción) | ✅ | ✅ (desde cancel) | ✅ (draft/cancel) | N/A | subscription_state == '6_churn' |
| mailing.mailing | ✅ (si in_queue) | N/A | ✅ | N/A | state == 'done' (enviado) |
| mrp.production | ✅ (si no done) | N/A | ✅ (cualquier estado excepto done) | N/A | button_mark_done |
| hr.leave | ✅ (refuse) | ✅ (action_reset_confirm desde cancel/refuse) | ⚠️ (según rol y estado) | N/A | validate |
| hr.expense.sheet | ✅ (refuse) | ✅ (action_reset_expense_sheets) | ✅ | N/A | Move posted |
| hr.applicant | N/A | N/A | ✅ | N/A | Empleado creado |
| fleet.vehicle | N/A | N/A | ✅ | N/A | Ninguno |
| fleet.vehicle.log.services | N/A | N/A | ✅ | N/A | Ninguno |
| maintenance.request | N/A | N/A | ✅ | N/A | Ninguno |
| repair.order | ✅ | ⚠️ (cancel_draft) | ⚠️ (cancel) | N/A | done |
| helpdesk.ticket | N/A | N/A | ✅ | N/A | Ninguno |
| quality.check | N/A | N/A | ✅ | N/A | Ninguno |
| planning.slot | N/A | N/A | ✅ | N/A | Ninguno |
| approval.request | ✅ | ✅ (draft) | ✅ | N/A | approved |

---

