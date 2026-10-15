## Orden de Procesamiento

```
1.  Pagos (account.payment) ← Bloquean facturas por reconciliacion
2.  Facturas/Asientos (account.move) ← Bloquean pedidos/albaranes
3.  Albaranes (stock.picking) ← Bloqueados si state=done
4.  Ordenes de fabricacion (mrp.production)
5.  Desechos (stock.scrap) ← Vinculados a produccion o picking
6.  Pedidos de venta (sale.order)
7.  Pedidos de compra (purchase.order)
8.  Oportunidades CRM (crm.lead)
9.  Tareas de proyecto (project.task)
10. Gastos (hr.expense + hr.expense.sheet)
11. Pedidos POS (pos.order) ← Requiere sesion abierta
12. Nominas (hr.payslip) ← Requiere moves cancelados
13. Ausencias (hr.leave) ← Requiere refuse antes de borrar
14. Contratos RRHH (hr.contract)
15. Activos contables (account.asset)
16. Mailings (mailing.mailing)
17. Tickets de soporte (helpdesk.ticket)
18. Controles de calidad (quality.check)
19. Solicitudes de mantenimiento (maintenance.request)
20. Documentos (documents.document)
21. Planificacion (planning.slot)
22. Aprobaciones (approval.request)
23. Registros de flota (fleet.vehicle.log.services / fleet.vehicle.log.fuel)
24. Reglas de reabastecimiento (stock.warehouse.orderpoint)
25. Suscripciones (sale.order con recurrence)
```

---

