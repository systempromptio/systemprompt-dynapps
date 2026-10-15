## Matriz de Reversibilidad Extendida

### Modelos Core

| Modelo | Estrategia | Pasos | Punto de no retorno? | Tier Maximo |
|--------|-----------|-------|---------------------|-------------|
| `sale.order` | cancel → delete | action_cancel → delete_record | No (siempre cancelable) | Tier 1 |
| `purchase.order` | cancel → delete | button_cancel → delete_record | No (si facturas canceladas primero) | Tier 1 |
| `account.move` | draft → cancel → delete | button_draft → button_cancel → delete_record | **Si**: `inalterable_hash`, `posted_before` + audit | Tier 3 |
| `account.payment` | cancel → draft → delete | action_cancel → action_draft → delete_record | **Si**: reconciliado con banco | Tier 2 |
| `stock.picking` | cancel/devolucion → delete | action_cancel o return picking | **Si**: `state=done` (requiere devolucion) | Tier 1 |
| `crm.lead` | delete directo | delete_record | No | Tier 1 |
| `project.task` | delete directo | delete_record (cascada subtareas) | No | Tier 1 |

### Modelos de Inventario/Fabricacion

| Modelo | Estrategia | Pasos | Punto de no retorno? | Tier Maximo |
|--------|-----------|-------|---------------------|-------------|
| `mrp.production` | cancel → draft → delete | action_cancel → button_draft → delete_record | **Si**: `state=done` con moves validados | Tier 3 |
| `stock.scrap` | delete o neutralizar | delete_record (solo draft) | **Si**: siempre (move done) | Tier 3 |
| `mrp.workorder` | cascada via produccion | Se revierte con su `mrp.production` padre | No (cascada) | Tier 1 |
| `mrp.bom` | delete directo | delete_record | No (si no hay producciones activas) | Tier 1 |
| `mrp.unbuild` | cancel → delete | Si `state != done`: cancel → delete | **Si**: `state=done` (moves validados) | Tier 3 |
| `stock.warehouse.orderpoint` | delete directo | delete_record | No | Tier 1 |

### Modelos RRHH

| Modelo | Estrategia | Pasos | Punto de no retorno? | Tier Maximo |
|--------|-----------|-------|---------------------|-------------|
| `hr.expense.sheet` | reset → delete | action_reset_expense_sheets → delete | **Si**: moves posted (cancelar primero) | Tier 2 |
| `hr.payslip` | cancel → draft → delete | action_payslip_cancel → action_payslip_draft → delete | **Si**: moves posted (cancelar primero) | Tier 2 |
| `hr.leave` | refuse → draft → delete | action_refuse → action_draft → delete | No (refuse restaura dias) | Tier 1 |
| `hr.contract` | cancel → delete | write state=cancel → delete | No (si no hay nominas activas) | Tier 1 |
| `hr.attendance` | delete directo | delete_record | No | Tier 1 |

### Modelos de Venta/Marketing/Suscripcion

| Modelo | Estrategia | Pasos | Punto de no retorno? | Tier Maximo |
|--------|-----------|-------|---------------------|-------------|
| `pos.order` | cancel (sesion abierta) / refund | action_pos_order_cancel o refund | **Si**: sesion cerrada con asiento publicado | Tier 3 |
| `pos.session` | reabrir (rescue) o ignorar | open_cashbox_pos para rescue | **Si**: `state=closed` con `move_id` publicado | Solo emergencia |
| `account.bank.statement.line` | desreconciliar → delete | Romper reconciliacion → delete_record | **Si**: reconciliado (desreconciliar primero) | Tier 2 |
| `account.asset` | draft → delete | set_to_draft → delete_record | **Si**: asientos amortizacion publicados | Tier 2 |
| `mailing.mailing` | cancel → draft → delete | action_cancel → action_set_as_draft → delete | **Si**: `state=done` (emails enviados) | Tier 1 |
| `sale.order` (suscripcion) | close → cancel → delete | set_close → action_cancel → delete | **Si**: periodos facturados con hash fiscal | Tier 3 |

### Modelos de Soporte/Calidad/Mantenimiento/Otros

| Modelo | Estrategia | Pasos | Punto de no retorno? | Tier Maximo |
|--------|-----------|-------|---------------------|-------------|
| `helpdesk.ticket` | delete directo | delete_record (borrar timesheets primero si hay) | No | Tier 1 |
| `quality.check` | delete directo | delete_record | No | Tier 1 |
| `maintenance.request` | delete directo | delete_record | No | Tier 1 |
| `documents.document` | delete directo | delete_record | No | Tier 1 |
| `planning.slot` | delete directo | delete_record | No | Tier 1 |
| `approval.request` | refuse → draft → delete | action_refuse → action_draft → delete | No | Tier 1 |
| `fleet.vehicle.log.services` | delete directo | delete_record | No | Tier 1 |
| `fleet.vehicle.log.fuel` | delete directo | delete_record | No | Tier 1 |

---

