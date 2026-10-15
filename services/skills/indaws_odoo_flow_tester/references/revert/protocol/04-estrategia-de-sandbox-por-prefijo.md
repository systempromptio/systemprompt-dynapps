## Estrategia de Sandbox por Prefijo

### Principio
TODOS los registros de test DEBEN usar un prefijo identificable: `[TEST-YYYYMMDD-HHMM]` en el campo
de nombre/referencia del modelo. Esto permite localizar y limpiar registros de test incluso si el
tracking JSON falla o se pierde.

### Campo de prefijo por modelo

| Modelo | Campo para prefijo | Ejemplo |
|--------|-------------------|---------|
| `sale.order` | `client_order_ref` | `[TEST-20260309-1430]` |
| `purchase.order` | `partner_ref` | `[TEST-20260309-1430]` |
| `account.move` | `ref` | `[TEST-20260309-1430]` |
| `account.payment` | `ref` | `[TEST-20260309-1430]` |
| `stock.picking` | `origin` | `[TEST-20260309-1430]` |
| `mrp.production` | `origin` | `[TEST-20260309-1430]` |
| `crm.lead` | `name` | `[TEST-20260309-1430] Oportunidad de prueba` |
| `project.task` | `name` | `[TEST-20260309-1430] Tarea de prueba` |
| `hr.expense` | `name` | `[TEST-20260309-1430] Gasto de prueba` |
| `hr.expense.sheet` | `name` | `[TEST-20260309-1430] Informe gastos` |
| `pos.order` | `pos_reference` | `[TEST-20260309-1430]` |
| `pos.session` | — | No se prefija (se identifica via sus orders) |
| `account.bank.statement.line` | `payment_ref` | `[TEST-20260309-1430]` |
| `account.asset` | `name` | `[TEST-20260309-1430] Activo de prueba` |
| `stock.scrap` | `name` | `[TEST-20260309-1430] Desecho de prueba` |
| `mrp.unbuild` | — | Se identifica via `mo_id` (orden de fabricacion) |
| `mrp.workorder` | — | Se identifica via `production_id` (cascada) |
| `mrp.bom` | `code` | `[TEST-20260309-1430]` |
| `hr.payslip` | `name` | `[TEST-20260309-1430] Nomina test` |
| `hr.contract` | `name` | `[TEST-20260309-1430] Contrato test` |
| `hr.attendance` | — | Se identifica via `employee_id` + rango de fecha |
| `hr.leave` | `name` | `[TEST-20260309-1430] Ausencia test` |
| `mailing.mailing` | `subject` | `[TEST-20260309-1430] Mailing test` |
| `stock.warehouse.orderpoint` | `name` | `[TEST-20260309-1430] Regla abastecimiento` |
| `sale.order` (suscripcion) | `client_order_ref` | `[TEST-20260309-1430]` |
| `helpdesk.ticket` | `name` | `[TEST-20260309-1430] Ticket test` |
| `quality.check` | `title` | `[TEST-20260309-1430] Control calidad` |
| `maintenance.request` | `name` | `[TEST-20260309-1430] Solicitud mantenimiento` |
| `documents.document` | `name` | `[TEST-20260309-1430] Documento test` |
| `planning.slot` | `name` | `[TEST-20260309-1430] Planificacion test` |
| `approval.request` | `name` | `[TEST-20260309-1430] Aprobacion test` |
| `fleet.vehicle.log.services` | `description` | `[TEST-20260309-1430] Servicio flota` |
| `fleet.vehicle.log.fuel` | `description` | `[TEST-20260309-1430] Combustible flota` |

### Uso en la creacion de registros

```bash
# El prefijo se genera automaticamente con fecha/hora del test
PREFIX="[TEST-$(date +%Y%m%d-%H%M)]"

# Ejemplo: crear pedido de venta con prefijo
create_record.sh sale.order '{
  "partner_id": 1,
  "client_order_ref": "[TEST-20260309-1430]",
  "order_line": [[0, 0, {"product_id": 5, "product_uom_qty": 1}]]
}' '{"context": {"tracking_disable": true}}'
```

### Limpieza de emergencia por prefijo

Si el tracking JSON se pierde o falla, usar el modo `--cleanup-by-prefix` de `revert_flow.py`:

```bash
# Buscar y listar todos los registros con el prefijo del test
python3 revert_flow.py --cleanup-by-prefix "[TEST-20260309-1430]" --dry-run

# Ejecutar limpieza real
python3 revert_flow.py --cleanup-by-prefix "[TEST-20260309-1430]"
```

El script busca en todos los modelos conocidos usando los campos de la tabla anterior.

---

