## NIVEL 2 — ORM + execute_method avanzado

### Metodos internos para bypass de restricciones

Usar `execute_method.sh` para llamar metodos internos que bypaseen restricciones estandar.

#### account.move — Revertir moves con hash fiscal

```bash
# _reverse_moves() crea un asiento inverso (nota de credito automatica)
# Deja el original intacto pero con saldo neto 0
# En Odoo 18.0, acepta default_values_list (opcional) y cancel (boolean)
execute_method.sh account.move _reverse_moves '[[<id>]]' '{"context": {"tracking_disable": true}}'

# Para revertir Y reconciliar automaticamente (cancel=True):
# Esto desreconcilia lineas primero, crea reversal, lo publica y reconcilia
execute_method.sh account.move _reverse_moves '[[<id>], null, true]' '{"context": {"tracking_disable": true}}'

# Para revertir multiples moves a la vez
execute_method.sh account.move _reverse_moves '[[<id1>, <id2>, <id3>]]' '{"context": {"tracking_disable": true}}'
```

#### account.move — Desreconciliacion programatica

```bash
# Metodo 1: Via remove_move_reconcile (recomendado)
# Buscar move lines del asiento
MOVE_LINE_IDS=$(search_records.sh account.move.line '[["move_id","=",<move_id>],["reconciled","=",true]]' '["id"]' 0)
# Desreconciliar (borra partials → borra fulls → revierte exchange diffs y CABA)
execute_method.sh account.move.line remove_move_reconcile "[[$MOVE_LINE_IDS]]" '{}'

# Metodo 2: Via borrado directo de partials
# Buscar reconciliaciones del move
PARTIAL_IDS=$(search_records.sh account.partial.reconcile '["|",["debit_move_id.move_id","=",<move_id>],["credit_move_id.move_id","=",<move_id>]]' '["id"]' 0)
# Borrar partials (dispara cascada completa: fulls, exchange moves, CABA entries)
delete_record.sh account.partial.reconcile '[<partial_ids>]'

# Metodo 3: Via payment.action_draft (si viene de un pago)
# action_draft → button_draft → remove_move_reconcile (todo automatico)
execute_method.sh account.payment action_draft '[[<payment_id>]]' '{"context": {"tracking_disable": true}}'
```

#### account.move — Forzar cancelacion con contextos especiales

```bash
# Forzar draft ignorando posted_before
execute_method.sh account.move button_draft '[[<id>]]' '{"context": {"tracking_disable": true, "force_delete": true}}'

# Cancelar con bypass de lock date check
execute_method.sh account.move button_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
```

#### account.move — _unlink_or_reverse (metodo inteligente)

En Odoo 18.0, `_unlink_or_reverse()` decide automaticamente la mejor estrategia:
```
1. Si _can_be_unlinked() (no hash, date > lock_date) → button_draft → unlink
2. Si _is_protected_by_audit_trail() (posted_before + audit_trail) → button_cancel (no borra)
3. Si tiene hash → intenta _reverse_moves (no borra)
```

```bash
# Dejar que Odoo decida la mejor estrategia
execute_method.sh account.move _unlink_or_reverse '[[<id>]]' '{"context": {"tracking_disable": true}}'
```

#### stock.picking — Flujo de devolucion via wizard

```bash
# Crear wizard (REQUIERE contexto con active_id y active_model)
WIZARD_ID=$(create_record.sh stock.return.picking '{"picking_id": <id>}' '{"context": {"active_id": <id>, "active_model": "stock.picking"}}')

# Devolucion completa (calcula cantidades no devueltas automaticamente)
execute_method.sh stock.return.picking action_create_returns_all "[[$WIZARD_ID]]" '{"context": {"active_id": <id>, "active_model": "stock.picking"}}'

# Buscar picking de devolucion (campo return_id, NO backorder_id)
RETURN_ID=$(search_records.sh stock.picking '[["return_id","=",<id>]]' '["id","name","state"]' 1)

# Validar devolucion
execute_method.sh stock.picking button_validate "[[$RETURN_ID]]" '{"context": {"tracking_disable": true, "skip_backorder": true}}'
```

#### mrp.production — Cancelar produccion finalizada

```bash
# _action_cancel con contexto especial
execute_method.sh mrp.production action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'

# Si tiene workorders finalizados, cancelarlos primero
WO_IDS=$(search_records.sh mrp.workorder '[["production_id","=",<id>]]' '["id"]' 0)
execute_method.sh mrp.workorder button_finish '[[<wo_ids>]]' '{"context": {"tracking_disable": true}}'
```

#### Contextos especiales ORM para bypass de restricciones (verificado)

```bash
# Tabla completa de contextos verificados contra Odoo 18.0:
#
# | Contexto                    | Modelo          | Efecto                                              | Referencia                    |
# |force_delete=True            | account.move    | Bypass _unlink_forbid_parts_of_chain + audit trail   | account_move.py L3500, L3510  |
# |disable_cancel_warning=True  | sale.order      | Salta wizard de cancelacion                          | sale_order.py L1322           |
# |tracking_disable=True        | Todos           | Evita tracking en chatter                            | ORM base                      |
# |mail_create_nolog=True       | Todos           | Evita log de mail                                    | mail module                   |
# |mail_notrack=True            | Todos           | Evita tracking de cambios                            | mail module                   |
# |skip_invoice_sync=True       | account.move    | Evita sync con facturas                              | account_move.py L3521         |
# |dynamic_unlink=True          | account.move    | Permite borrado dinamico                             | account_move.py L3521         |
# |leave_skip_date_check=True   | hr.leave        | Salta validacion de fechas en unlink                 | hr_leave.py L853              |
# |skip_activity=True           | mrp.production  | Salta log de actividad en cancel                     | mrp_production.py L1760       |
# |prefetch_fields=False        | Varios          | Evita prefetch en operaciones masivas                | Optimizacion rendimiento      |
```

#### Resetear secuencias via ir.sequence

```bash
# Obtener secuencia actual
search_records.sh ir.sequence '[["code","=","sale.order"],["company_id","=",1]]' '["id","number_next","prefix"]' 0

# Escribir nuevo valor de next_number
update_record.sh ir.sequence '[<seq_id>]' '{"number_next_actual": <valor_pre_test>}'

# Para secuencias con rangos de fecha (facturas)
search_records.sh ir.sequence.date_range '[["sequence_id","=",<seq_id>]]' '["id","date_from","date_to","number_next"]' 0
update_record.sh ir.sequence.date_range '[<range_id>]' '{"number_next_actual": <valor_pre_test>}'
```

**ADVERTENCIA sobre secuencias de facturas con hash:**
Si el diario tiene `restrict_mode_hash_table=True`, los numeros de secuencia forman parte del hash
encadenado (`name` es un campo de integridad). Retroceder la secuencia podria:
- Generar un numero duplicado → fallo en UNIQUE constraint
- Romper la cadena de hash (el siguiente hash no coincidira con el anterior)
- NUNCA restaurar secuencias de diarios hasheados

---

