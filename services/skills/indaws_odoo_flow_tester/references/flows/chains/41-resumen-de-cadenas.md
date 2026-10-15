## RESUMEN DE CADENAS

| ID | Nombre | Apps | Pasos | Rollback | [MIG] | Tiempo Est. |
|-----|--------|------|-------|----------|-------|-------------|
| CHAIN_SALE_TO_CASH | Venta Completa | Ventas+Inv+Cont | 8 | 8 | SI | 2-3 min |
| CHAIN_PURCHASE_TO_PAY | Compra Completa | Compras+Inv+Cont | 8 | 7 | SI | 2-3 min |
| CHAIN_SALE_ADVANCE_PAYMENT | Venta con Anticipo | Ventas+Cont | 8 | 8 | | 3-4 min |
| CHAIN_CRM_TO_SALE | CRM a Venta | CRM+Ventas | 5 | 4 | | 1-2 min |
| CHAIN_MRP_SIMPLE | Fabricacion | MRP+Inv | 5 | Parcial | SI | 1-2 min |
| CHAIN_STOCK_INTERNAL | Transferencia Interna | Inv | 3 | 1 | SI | 1 min |
| CHAIN_CREDIT_NOTE | Nota de Credito | Cont | 4 | 4 | SI | 1 min |
| CHAIN_HR_EXPENSE | Gasto Empleado | RRHH | 3 | 3 | | 1 min |
| CHAIN_POS_SESSION | Sesion POS Completa | POS+Cont | 6 | 6 | | 2-3 min |
| CHAIN_SUBSCRIPTION_LIFECYCLE | Ciclo de Vida Suscripcion | Suscripciones+Cont | 6 | 6 | | 2-3 min |
| CHAIN_EXPENSE_TO_PAYMENT | Gasto hasta Pago | RRHH+Cont | 6 | 6 | | 2-3 min |
| CHAIN_DROPSHIP | Dropshipping Completo | Ventas+Compras+Inv | 6 | 6 | | 2-3 min |
| CHAIN_MRP_FULL | Fabricacion Completa | MRP+Inv | 8 | Parcial | SI | 3-5 min |
| CHAIN_BANK_RECONCILIATION | Conciliacion Bancaria | Cont | 3 | 3 | SI | 1 min |
| CHAIN_ASSET_LIFECYCLE | Activo Fijo | Cont (Enterprise) | 6 | 6 | | 2-3 min |
| CHAIN_PAYROLL | Nomina Completa | RRHH (Enterprise) | 5 | 5 | | 2-3 min |
| CHAIN_INVENTORY_REPLENISHMENT | Reabastecimiento Auto | Inv+Compras | 6 | 6 | | 2-3 min |
| CHAIN_INTERCOMPANY | Flujo Inter-empresa | Ventas+Compras+Multi | 6 | 6 | | 3-4 min |
| CHAIN_MRP_COMPLETE | Fabricacion + Entrega | MRP+Inv+Ventas | 8 | Parcial | SI | 3-5 min |
| CHAIN_REPAIR_COMPLETE | Reparacion Completa | Reparaciones+Inv+Cont | 7 | Parcial | | 2-4 min |
| CHAIN_EXPENSE_TO_PAY | Gasto hasta Pago Extendido | RRHH+Cont | 7 | 7 | | 2-3 min |
| CHAIN_RECRUITMENT | Reclutamiento Completo | Reclutamiento+RRHH | 6 | 5 | | 2-3 min |
| CHAIN_LEAVE_CYCLE | Ciclo de Ausencias | Ausencias | 6 | 5 | | 1-2 min |
| CHAIN_INTERCOMPANY_FULL | Inter-empresa + Facturacion | Ventas+Compras+Inv+Cont+Multi | 10 | 10 | | 4-6 min |
| CHAIN_QUALITY_MRP | Calidad en Fabricacion | Calidad+MRP+Inv | 8 | Parcial | | 3-5 min |
| CHAIN_MAINTENANCE_CYCLE | Ciclo de Mantenimiento | Mantenimiento+Inv | 5 | 5 | | 1-2 min |
| CHAIN_HELPDESK_TO_REPAIR | Helpdesk a Reparacion | Helpdesk+Reparaciones+Inv | 7 | Parcial | | 3-5 min |
| CHAIN_CRM_TO_CASH | CRM a Cobro (End-to-End) | CRM+Ventas+Inv+Cont | 12 | 12 | | 4-6 min |
| CHAIN_BANK_STATEMENT_FULL | Extracto Bancario Completo | Cont | 6 | 3 | SI | 2-3 min |
| CHAIN_STOCK_RETURN | Devolucion de Mercancia | Ventas+Inv | 8 | 5 | SI | 2-4 min |
| CHAIN_REFUND_FLOW | Nota Credito + Reconciliacion | Cont | 7 | 6 | SI | 2-3 min |
| CHAIN_HR_CONTRACT_LIFECYCLE | Ciclo de Vida Contrato | RRHH | 6 | 6 | | 1-2 min |
| CHAIN_SALE_ADAPTIVE | Venta Adaptativa (multi-step) | Ventas+Inv+Cont | 5-9 | Variable | SI | 3-5 min |
| CHAIN_INTERCOMPANY_SALE | Venta Inter-empresa (plantilla) | Ventas+Compras+Multi | 8 | 8 | | 4-6 min |
| CHAIN_LEAVE_ALLOCATION_FULL | Asignacion y Ausencia Completa | Ausencias | 7 | 3 | | 1-2 min |
| CHAIN_BANK_RECONCILIATION_COMPLETE | Conciliacion Bancaria Completa | Cont | 8 | 3 | SI | 2-3 min |
| CHAIN_SCRAP_AND_ADJUST | Desguace y Ajuste Inventario | Inv | 5 | Parcial | | 1-2 min |
| CHAIN_ASSET_LIFECYCLE_FULL | Activo Fijo Completo con Reversion | Cont (Enterprise) | 5 | 5 | | 2-4 min |
| CHAIN_HELPDESK_TO_REPAIR_INVOICE | Helpdesk a Reparacion + Facturacion | Helpdesk+Rep+Inv+Cont | 9 | Parcial | | 3-5 min |
