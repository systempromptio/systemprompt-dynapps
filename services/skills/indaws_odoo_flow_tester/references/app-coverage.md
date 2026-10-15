# App Coverage — 28+ Apps, 190+ Flujos, 39 Cadenas

| App | Flujos | Cadenas | Notas |
|-----|--------|---------|-------|
| Ventas | 6 | CHAIN_SALE_TO_CASH, CHAIN_SALE_ADVANCE_PAYMENT, CHAIN_INTERCOMPANY | Anticipos, multi-empresa |
| Compras | 5 | CHAIN_PURCHASE_TO_PAY, CHAIN_DROPSHIP, CHAIN_INVENTORY_REPLENISHMENT | Dropship, reabastecimiento |
| Inventario | 4+2 ext | CHAIN_STOCK_INTERNAL | Desguace, ajustes, lotes, series, multi-almacén |
| Contabilidad | 6+3 ext | CHAIN_CREDIT_NOTE, CHAIN_BANK_RECONCILIATION | Extractos bancarios, conciliación, posición fiscal |
| Pagos | 5 | (integrado en cadenas) | Inbound, outbound, reconciliación, draft, delete |
| CRM | 5 | CHAIN_CRM_TO_SALE, CHAIN_CRM_TO_CASH | Pipeline completo, actividades, archivo |
| Proyecto | 4 | (standalone) | Tareas, cambio stage, archivo, delete |
| RRHH — Gastos | 9 | CHAIN_HR_EXPENSE, CHAIN_EXPENSE_TO_PAY | Gastos individuales + informes |
| RRHH — Ausencias | 4 | CHAIN_LEAVE_CYCLE | Allocation, solicitud, aprobación, rechazo |
| RRHH — Asistencia | 5 | (standalone) | Check-in/out, overtime, employee_id cascade |
| RRHH — Nóminas | 2+1 | CHAIN_PAYROLL | Enterprise — payslip + contratos |
| MRP / Fabricación | 7+1 | CHAIN_MRP_SIMPLE, CHAIN_MRP_FULL, CHAIN_MRP_COMPLETE, CHAIN_QUALITY_MRP | BOM, workorders, subcontratación, calidad |
| POS | 3 | CHAIN_POS_SESSION | Sesión completa, facturas POS |
| Suscripciones | 4 | CHAIN_SUBSCRIPTION_LIFECYCLE | Enterprise — usar `plan_id` (NO `recurrence_id`) |
| Email Marketing | 2 | (standalone) | Campañas, A/B test |
| Activos Fijos | 3 | CHAIN_ASSET_LIFECYCLE | Enterprise — depreciación, disposición |
| Reparaciones | 7 | CHAIN_REPAIR_COMPLETE, CHAIN_HELPDESK_TO_REPAIR | 5 flujos de estado + cancel_draft + delete |
| Reclutamiento | 5 | CHAIN_RECRUITMENT | hr.applicant + hr.candidate previo |
| Flota | 5 | (standalone) | vehicle + log.services. **fleet.vehicle.log.fuel NO existe en 18.0** |
| Mantenimiento | 7 | CHAIN_MAINTENANCE_CYCLE | equipment + request |
| Helpdesk | 4 | CHAIN_HELPDESK_TO_REPAIR | Enterprise — stage-based |
| Calidad | 7 | CHAIN_QUALITY_MRP | Enterprise — quality.check + quality.alert |
| Planning | 6 | (standalone) | Enterprise — planning.slot: draft/published |
| Aprobaciones | 8 | (standalone) | Enterprise — approval.request + approval.category |

## 39 Cadenas Predefinidas

Full chain definitions with rollback steps: `scripts/flows.py --chain <ID>`.

| ID | Nombre | Apps | Pasos |
|-----|--------|------|-------|
| CHAIN_SALE_TO_CASH | Venta Completa | Ventas+Inv+Cont | 8 |
| CHAIN_PURCHASE_TO_PAY | Compra Completa | Compras+Inv+Cont | 8 |
| CHAIN_SALE_ADVANCE_PAYMENT | Venta con Anticipo | Ventas+Cont | 8 |
| CHAIN_CRM_TO_SALE | CRM a Venta | CRM+Ventas | 5 |
| CHAIN_CRM_TO_CASH | CRM a Cobro (E2E) | CRM+Ventas+Inv+Cont | 12 |
| CHAIN_MRP_SIMPLE | Fabricación Simple | MRP+Inv | 5 |
| CHAIN_MRP_FULL | Fabricación Completa | MRP+Inv | 8 |
| CHAIN_MRP_COMPLETE | Fabricación + Entrega | MRP+Inv+Ventas | 8 |
| CHAIN_STOCK_INTERNAL | Transferencia Interna | Inv | 3 |
| CHAIN_STOCK_RETURN | Devolución Mercancía | Ventas+Inv | 8 |
| CHAIN_CREDIT_NOTE | Nota de Crédito | Cont | 4 |
| CHAIN_REFUND_FLOW | NC con Reconciliación | Cont | 7 |
| CHAIN_BANK_RECONCILIATION | Conciliación Bancaria | Cont | 3 |
| CHAIN_BANK_RECONCILIATION_COMPLETE | Conciliación con Verificación | Cont | 8 |
| CHAIN_BANK_STATEMENT_FULL | Extracto Completo | Cont | 6 |
| CHAIN_HR_EXPENSE | Gasto Empleado | RRHH | 3 |
| CHAIN_EXPENSE_TO_PAY | Gasto hasta Pago | RRHH+Cont | 6 |
| CHAIN_EXPENSE_TO_PAYMENT | Gasto Extendido | RRHH+Cont | 7 |
| CHAIN_LEAVE_CYCLE | Ciclo Ausencias | Ausencias | 6 |
| CHAIN_LEAVE_ALLOCATION_FULL | Asignación + Ausencia | Ausencias | 7 |
| CHAIN_POS_SESSION | Sesión POS | POS+Cont | 6 |
| CHAIN_SUBSCRIPTION_LIFECYCLE | Suscripción | Suscripciones+Cont | 6 |
| CHAIN_DROPSHIP | Dropshipping | Ventas+Compras+Inv | 6 |
| CHAIN_ASSET_LIFECYCLE | Activo Fijo | Cont (Enterprise) | 6 |
| CHAIN_ASSET_LIFECYCLE_FULL | Activo con Reversión | Cont (Enterprise) | 5 |
| CHAIN_PAYROLL | Nómina | RRHH (Enterprise) | 5 |
| CHAIN_INVENTORY_REPLENISHMENT | Reabastecimiento | Inv+Compras | 6 |
| CHAIN_INTERCOMPANY | Inter-empresa | Ventas+Compras+Multi | 6 |
| CHAIN_INTERCOMPANY_FULL | Inter-empresa + Fact | Multi+Cont | 10 |
| CHAIN_INTERCOMPANY_SALE | Venta Inter-empresa | Multi | 8 |
| CHAIN_REPAIR_COMPLETE | Reparación | Reparaciones+Inv+Cont | 7 |
| CHAIN_RECRUITMENT | Reclutamiento | Reclutamiento+RRHH | 6 |
| CHAIN_QUALITY_MRP | Calidad en MRP | Calidad+MRP+Inv | 8 |
| CHAIN_MAINTENANCE_CYCLE | Mantenimiento | Mantenimiento+Inv | 5 |
| CHAIN_HELPDESK_TO_REPAIR | Helpdesk→Reparación | Helpdesk+Reparaciones | 7 |
| CHAIN_HELPDESK_TO_REPAIR_INVOICE | Helpdesk→Reparación+Fact | Enterprise | 9 |
| CHAIN_SALE_ADAPTIVE | Venta Adaptativa | Ventas+Inv+Cont | 11 |
| CHAIN_SCRAP_AND_ADJUST | Desguace + Ajuste | Inv | 5 |
| CHAIN_HR_CONTRACT_LIFECYCLE | Contrato RRHH | RRHH (Contratos) | 6 |
