## MATRIZ DE REVERSIBILIDAD

### Modelos Core

| Modelo | Nivel 1 (ORM) | Nivel 2 (Methods) | Nivel 3 (SQL) | Punto de No Retorno | Alternativa de Neutralizacion |
|--------|---------------|-------------------|---------------|---------------------|-------------------------------|
| `sale.order` | cancel → delete | N/A | DELETE directo | No (siempre cancelable) | `active=False`, renombrar ref |
| `purchase.order` | cancel → delete | N/A | DELETE directo | No (si facturas canceladas) | `active=False`, renombrar ref |
| `account.move` | draft → cancel → delete | `_reverse_moves()`, `_unlink_or_reverse()` | DELETE con limpieza FK + reconciliaciones | **Si**: `inalterable_hash`, `posted_before` + audit | Nota de credito, neutralizar ref |
| `account.payment` | cancel → draft → delete | `action_draft()` (desreconcilia auto) | DELETE con reconciliaciones | **Si**: reconciliado con banco | Desreconciliar primero |
| `stock.picking` | cancel → delete / devolucion | `action_create_returns_all` (wizard) | DELETE con stock.move + quant fix | **Si**: `state=done` (requiere devolucion) | Devolucion inversa |
| `crm.lead` | delete directo | N/A | DELETE directo | No | N/A |
| `project.task` | delete directo | N/A | DELETE con timesheets | No | `active=False` |

### Modelos de Inventario/Fabricacion

| Modelo | Nivel 1 (ORM) | Nivel 2 (Methods) | Nivel 3 (SQL) | Punto de No Retorno | Alternativa de Neutralizacion |
|--------|---------------|-------------------|---------------|---------------------|-------------------------------|
| `mrp.production` | cancel → draft → delete | `action_cancel` + force | DELETE con workorders/moves | **Si**: `state=done` con moves validados | Crear unbuild, neutralizar origin |
| `stock.scrap` | delete (solo draft) | N/A | DELETE con stock.move | **Si**: siempre (move done) | Crear unbuild/devolucion manual |
| `stock.valuation.layer` | N/A (cascada via move) | N/A | DELETE rompiendo auto-ref | **Si**: si tiene account.move con hash | Neutralizar via move reversal |

### Modelos RRHH

| Modelo | Nivel 1 (ORM) | Nivel 2 (Methods) | Nivel 3 (SQL) | Punto de No Retorno | Alternativa de Neutralizacion |
|--------|---------------|-------------------|---------------|---------------------|-------------------------------|
| `hr.expense.sheet` | reset → delete | N/A | DELETE con expenses | **Si**: moves posted (cancelar primero) | Neutralizar name |
| `hr.payslip` | cancel → draft → delete | `action_payslip_cancel` | DELETE con account.move | **Si**: moves posted (cancelar primero) | Neutralizar name |
| `hr.leave` | refuse → draft → delete | `action_refuse` + `action_draft` | DELETE directo | No (refuse restaura dias) | Neutralizar name |
| `hr.contract` | write state=cancel → delete | N/A | DELETE directo | No (si no hay nominas activas) | `state=cancel` |

### Modelos de Venta/Marketing

| Modelo | Nivel 1 (ORM) | Nivel 2 (Methods) | Nivel 3 (SQL) | Punto de No Retorno | Alternativa de Neutralizacion |
|--------|---------------|-------------------|---------------|---------------------|-------------------------------|
| `pos.order` | cancel (sesion abierta) / refund | N/A | DELETE con pagos/lineas | **Si**: sesion cerrada con asiento publicado | Refund, neutralizar note |
| `account.asset` | draft → delete | `set_to_draft` | DELETE con depreciation moves | **Si**: asientos amortizacion publicados | Revertir asientos primero |
| `mailing.mailing` | cancel → draft → delete | `action_cancel` + `action_set_as_draft` | DELETE con trazas | **Si**: `state=done` (emails enviados) | Borrar registro (emails irrecuperables) |
| `sale.order` (suscripcion) | close → cancel → delete | `set_close` | DELETE directo | **Si**: periodos facturados con hash | Neutralizar ref |

### Modelos de Soporte/Calidad/Mantenimiento

| Modelo | Nivel 1 (ORM) | Nivel 2 (Methods) | Nivel 3 (SQL) | Punto de No Retorno | Alternativa de Neutralizacion |
|--------|---------------|-------------------|---------------|---------------------|-------------------------------|
| `helpdesk.ticket` | delete directo | N/A | DELETE con timesheets | No | `active=False` |
| `quality.check` | delete directo | N/A | DELETE directo | No | N/A |
| `maintenance.request` | delete directo | N/A | DELETE directo | No | `active=False` |
| `fleet.vehicle.log.services` | delete directo | N/A | DELETE directo | No | N/A |

---

