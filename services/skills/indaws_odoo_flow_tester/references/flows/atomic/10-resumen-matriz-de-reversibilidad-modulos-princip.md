## RESUMEN — Matriz de Reversibilidad (Módulos Principales)

| Modelo | Cancel | Draft | Delete | Return | Punto de No Retorno |
|--------|--------|-------|--------|--------|---------------------|
| sale.order | ✅ (si no locked) | ✅ (desde cancel/sent) | ✅ (draft/cancel) | N/A | Picking done |
| purchase.order | ✅ (si no hay invoices) | ✅ | ✅ (solo cancel) | N/A | Invoice posted |
| stock.picking | ✅ (si no done) | N/A | ✅ (si moves no done) | ✅ (solo done) | Move done sin return |
| account.move | ✅ | ✅ (si no hash/exchange/caba) | ⚠️ (restricciones — hash NO bloquea unlink) | ✅ (reversal) | inalterable_hash (solo para draft), audit trail |
| account.payment | ✅ | ✅ | ✅ | N/A | Move reconciliado |
| crm.lead | N/A | N/A | ✅ | N/A | Ninguno |
| project.task | ✅ (1_canceled) | ✅ (01_in_progress) | ✅ (+ subtareas) | N/A | Ninguno |
| hr.expense | ✅ (refuse) | ✅ (reset) | ✅ | N/A | Move posted |
| mrp.production | ✅ (si no done) | N/A | ✅ (cualquier estado excepto done) | N/A | button_mark_done |
| hr.leave | ✅ (refuse) | ✅ (action_reset_confirm desde cancel/refuse) | ⚠️ (según rol y estado) | N/A | validate |
| hr.expense.sheet | ✅ (refuse) | ✅ (action_reset_expense_sheets) | ✅ | N/A | Move posted |
| hr.applicant | N/A | N/A | ✅ | N/A | Empleado creado |
| fleet.vehicle | N/A | N/A | ✅ | N/A | Ninguno |
| maintenance.request | N/A | N/A | ✅ | N/A | Ninguno |
| repair.order | ✅ | ⚠️ (cancel_draft) | ⚠️ (cancel) | N/A | done |
| helpdesk.ticket | N/A | N/A | ✅ | N/A | Ninguno |
| quality.check | N/A | N/A | ✅ | N/A | Ninguno |
| planning.slot | N/A | N/A | ✅ | N/A | Ninguno |
| approval.request | ✅ | ✅ (draft) | ✅ | N/A | approved |

*(Ver Matriz Ampliada al final del documento para todos los módulos)*

Leyenda:
- ✅ = Soportado sin restricciones mayores
- ⚠️ = Soportado con restricciones (ver flujo específico)
- N/A = No aplica para este modelo

---

