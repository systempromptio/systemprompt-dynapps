# Execution Guide — Flujos via API

## Estructura de un Flujo Atómico

```json
{
  "flow_id": "SALE_ORDER_CREATE",
  "model": "sale.order",
  "steps": [
    {
      "action": "create",
      "model": "sale.order",
      "values": {"partner_id": "{data.partner_id}", "client_order_ref": "{data.test_prefix}"},
      "save_as": "sale_order_id",
      "timeout": 30
    },
    {
      "action": "execute",
      "model": "sale.order",
      "method": "action_confirm",
      "ids": ["{result.sale_order_id}"],
      "verify": {"field": "state", "expected": "sale"},
      "timeout": 30
    }
  ],
  "revert": [
    {"action": "execute", "model": "sale.order", "method": "action_cancel", "ids": ["{result.sale_order_id}"]},
    {"action": "delete", "model": "sale.order", "ids": ["{result.sale_order_id}"]}
  ]
}
```

## Acciones Disponibles

| Acción | Script odoo-pilot | Descripción |
|--------|-------------------|-------------|
| search | search_records.sh | Buscar registros |
| create | create_record.sh | Crear registro |
| update | update_record.sh | Actualizar registro |
| delete | delete_record.sh | Eliminar registro |
| execute | execute_method.sh | Ejecutar método (workflow) |
| verify | search_records.sh | Verificar campo/estado |

## Variables del Sistema

- `{data.*}` — Datos del flujo (partner_id, product_id, test_prefix)
- `{result.*}` — IDs de registros creados durante ejecución
- `{env.*}` — Variables de entorno (ODOO_URL, ODOO_DB)
- `{pre.*}` — IDs de registros encontrados en verificación (precondiciones)
- `{today}` — Fecha actual en formato YYYY-MM-DD

## Campos para Prefijo de Test

| Modelo | Campo para prefijo |
|--------|-------------------|
| `sale.order` | `client_order_ref` |
| `purchase.order` | `partner_ref` |
| `account.move` | `ref` |
| `account.payment` | `ref` |
| `stock.picking` | `origin` |
| `mrp.production` | `origin` |
| `crm.lead` | `name` |
| `project.task` | `name` |
| `hr.expense` | `name` |
| `hr.expense.sheet` | `name` |
| `pos.order` | `pos_reference` |
| `account.bank.statement.line` | `payment_ref` |
| `account.asset` | `name` |
| `hr.payslip` | `name` |
| `hr.contract` | `name` |
| `mailing.mailing` | `subject` |
| `mrp.bom` | `code` |
| `stock.warehouse.orderpoint` | `name` |

## Ejecución con execute_flow.py

```bash
python3 execute_flow.py flow.json \
    --pilot-dir /path/to/odoo-pilot/scripts \
    [--dry-run] [--verbose] \
    [--output /tmp/flow_tracking.json] \
    [--env staging|production] \
    [--prefix "TEST-20260309-1430"]
```

## Reversión con revert_flow.py

```bash
# Dry-run
python3 revert_flow.py --tracking /tmp/flow_tracking.json --dry-run

# Con SQL fallback
python3 revert_flow.py --tracking /tmp/flow_tracking.json --sql-fallback

# Solo neutralizar (no eliminar)
python3 revert_flow.py --tracking /tmp/flow_tracking.json --neutralize-only

# Limpieza por prefijo
python3 revert_flow.py --cleanup-by-prefix "[TEST-20260309-1430]"

# Recuperar secuencias
python3 revert_flow.py --tracking /tmp/flow_tracking.json --recover-sequences
```

## Puntos de No Retorno

| Modelo | Condición | Por qué | Solución |
|--------|-----------|---------|----------|
| `account.move` | Hash fiscal + audit trail | Hash bloquea draft, audit trail bloquea unlink | Asiento inverso o SQL Tier 3 |
| `stock.picking` | `done` | No permite cancelar | Wizard `stock.return.picking` |
| `account.payment` | Reconciliado | Asientos conciliados | Desreconciliar primero |
| `pos.order` | `done` + sesión cerrada | Asientos consolidados | Reversión manual o SQL |
| `hr.payslip` | `done` | Asientos vinculados | `action_payslip_cancel` + revertir asientos |

Ver `scripts/flows.py --revert-tema reversibilidad` para la matriz completa.
