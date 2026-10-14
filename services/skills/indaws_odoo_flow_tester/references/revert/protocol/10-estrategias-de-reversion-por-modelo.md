## Estrategias de Reversion por Modelo

### Modelos Core

#### account.payment
```
Estado: draft → delete
Estado: in_process/paid →
  1. action_cancel (estado → canceled, cancela moves draft via unlink, cancela moves posted via button_cancel)
  2. action_draft (estado → draft, llama move.button_draft() que desreconcilia automaticamente)
  3. delete_record (borra payment + moves)
```
**API:**
```bash
execute_method.sh account.payment action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh account.payment action_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh account.payment '[<id>]'
```

#### account.move (Facturas/Asientos)
```
Estado: draft → delete (con restricciones: force_delete si posted_before + audit)
Estado: posted →
  PASO 0 (PRE-CHECK): Leer inalterable_hash y company.check_account_audit_trail
  SI tiene inalterable_hash:
    → _reverse_moves(cancel=True) — crea reversal, reconcilia, saldo neto 0
    → NO intentar button_draft ni button_cancel
  SI posted_before + audit_trail y NO hash:
    → button_cancel (estado → cancel)
    → delete_record con contexto force_delete=True
  CASO ESTANDAR (sin hash, sin audit trail):
    1. button_draft (desreconcilia via remove_move_reconcile, limpia analitica)
       FALLA SI: inalterable_hash, exchange entry, cash basis, need_cancel_request
    2. button_cancel (cambia a cancel)
    3. delete_record con contexto force_delete=True
       FALLA SI: secuencia no-ultima sin group_account_manager
```
**API:**
```bash
# Pre-check obligatorio
search_records.sh account.move '[["id","=",<id>]]' '["state","inalterable_hash","posted_before"]' 1

# Caso estandar (sin hash)
execute_method.sh account.move button_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh account.move button_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh account.move '[<id>]' '{"context": {"force_delete": true}}'

# Caso con hash → reversal obligatorio
execute_method.sh account.move _reverse_moves '[[<id>], null, true]' '{"context": {"tracking_disable": true}}'
```
**Si button_draft FALLA (hash/audit):**
- Opcion A: Crear asiento inverso via `_reverse_moves(cancel=True)` → deja saldo neto 0
- Opcion B: Neutralizar: `update_record.sh account.move '[<id>]' '{"ref": "[NEUTRALIZED-TEST]"}'`
- Opcion C: Escalar a Tier 3 (SQL) → ver `flows.py --revert <modelo>`
- NUNCA forzar borrado de registros con hash sin aprobacion explicita

**Nota sobre `force_delete` contexto:**
El contexto `force_delete=True` SOLO funciona para `account.move`. No tiene efecto en otros modelos
como `sale.order`, `purchase.order` o `stock.picking`.

**Flujo interno de button_draft (Odoo 18.0):**
```
button_draft()
  → Verifica state in ('cancel', 'posted')
  → Verifica no need_cancel_request
  → _check_draftable()
    → Verifica NO es exchange diff (via account_full_reconcile.exchange_move_id y account_partial_reconcile.exchange_move_id)
    → Verifica NO es cash basis entry (tax_cash_basis_rec_id o tax_cash_basis_origin_move_id)
    → Verifica NO tiene inalterable_hash
  → Borra analytic_line_ids
  → Llama line_ids.remove_move_reconcile() que hace:
    → (matched_debit_ids + matched_credit_ids).unlink()
    → Esto borra account.partial.reconcile
    → Que a su vez borra account.full.reconcile huerfanos
    → Y revierte automaticamente exchange diff y CABA entries
  → state = 'draft'
```

**Flujo interno de _reverse_moves (Odoo 18.0):**
```
_reverse_moves(default_values_list=None, cancel=False)
  → Si cancel=True: primero remove_move_reconcile() en todas las lineas
  → Crea copia inversa de cada move con move_type invertido
  → Invierte balances de lineas
  → Si cancel=True: publica y reconcilia moves originales con sus inversos
  → Retorna recordset de reverse moves
```

#### stock.picking (Albaranes)

**IMPORTANTE (verificado en Odoo 18.0 source):**
- `stock.picking` NO tiene su propio metodo `_action_cancel`. Su `action_cancel()` llama `self.move_ids._action_cancel()`.
- `stock.move._action_cancel()` lanza `UserError` si `move.state == 'done' and not move.scrapped`.
- `stock.picking.unlink()` llama `self.move_ids._action_cancel()` seguido de `self.move_ids.unlink()`.
- `stock.move.unlink()` tiene constraint `_unlink_if_draft_or_cancel` que impide borrar moves con `move_orig_ids` o `move_dest_ids` si no estan en draft/cancel.
- `stock.picking._can_return()` solo devuelve True si `self.state == 'done'`.

```
Estado: draft/waiting/confirmed/assigned →
  1. action_cancel (cancela moves via _action_cancel)
  2. delete_record (si no tiene links a otros moves)

Estado: done →
  NO SE PUEDE CANCELAR NI BORRAR
  Estrategia: Crear devolucion completa (return picking)
```

**Flujo de devolucion completo (Tier 1 para done pickings):**

```bash
# Paso 1: Crear wizard con contexto active_id (OBLIGATORIO — default_get lo requiere)
WIZARD_ID=$(create_record.sh stock.return.picking '{"picking_id": <id>}' '{"context": {"active_id": <id>, "active_model": "stock.picking"}}')

# Paso 2: Ejecutar devolucion completa (pone quantity = total no devuelto en cada linea)
execute_method.sh stock.return.picking action_create_returns_all "[[$WIZARD_ID]]" '{"context": {"active_id": <id>, "active_model": "stock.picking"}}'

# Paso 3: Buscar picking de devolucion
# El return picking tiene return_id = <id> (campo directo, no backorder_id)
# El origin sera "Return of <picking_name>"
RETURN_ID=$(search_records.sh stock.picking '[["return_id","=",<id>]]' '["id","name","state"]' 1)

# Paso 4: Validar devolucion (poner cantidades y confirmar)
# El return picking se crea en estado confirmed/assigned (action_confirm + action_assign se llaman automaticamente)
execute_method.sh stock.picking button_validate "[[$RETURN_ID]]" '{"context": {"tracking_disable": true, "skip_backorder": true}}'
```

**Flujo interno del wizard (verificado en source `stock_picking_return.py`):**
```
action_create_returns_all()
  → Para cada return_move, calcula quantity = move.quantity - sum(already_returned)
  → Llama action_create_returns()
    → Llama _create_return()
      → Unreserve moves_dest afectados
      → Copia picking con vals: state='draft', return_id=picking.id,
        origin="Return of <picking_name>", locations invertidas
        (location_id=picking.location_dest_id, location_dest_id basado en return_picking_type o picking.location_id)
      → Para cada linea: copia move con origin_returned_move_id, links move_orig/move_dest
      → action_confirm() + action_assign() en el nuevo picking
```

**NOTA:** El metodo del wizard es `action_create_returns` (NO `create_returns`). La busqueda del return picking debe usar `return_id` (no `backorder_id`).

**API:**
```bash
execute_method.sh stock.picking action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh stock.picking '[<id>]'
```

#### sale.order
```
Estado: draft →
  1. delete_record (permitido por @api.ondelete _unlink_except_draft_or_cancel)

Estado: sale/sent →
  1. Verificar locked: si order.locked → action_unlock primero
  2. action_cancel (con contexto disable_cancel_warning)
     - Cancela pickings no-done automaticamente
     - Cancela facturas draft automaticamente
     - NO cancela pickings done ni facturas posted
  3. delete_record (ahora en estado cancel, permitido por constraint)

Estado: cancel →
  1. delete_record directamente
```
**NOTA:** `force_delete` contexto NO funciona para sale.order.
**NOTA:** `action_draft()` solo funciona desde estados `cancel` o `sent`.
**API:**
```bash
# Verificar si esta bloqueado
search_records.sh sale.order '[["id","=",<id>]]' '["state","locked"]' 1
# Si locked=True:
execute_method.sh sale.order action_unlock '[[<id>]]' '{"context": {"tracking_disable": true}}'
# Cancelar
execute_method.sh sale.order action_cancel '[[<id>]]' '{"context": {"disable_cancel_warning": true, "tracking_disable": true}}'
delete_record.sh sale.order '[<id>]'
```

#### purchase.order
```
IMPORTANTE: Mas estricto que sale.order — @api.ondelete _unlink_if_cancelled()
solo permite borrar en estado 'cancel'. Ni siquiera 'draft' es borrable sin cancelar.

Estado: draft →
  1. button_cancel (OBLIGATORIO antes de borrar)
  2. delete_record (ahora en cancel)

Estado: purchase/done →
  1. Verificar facturas asociadas
     FALLA SI: hay facturas en estado != (cancel, draft)
     → Primero cancelar facturas asociadas
  2. button_cancel
  3. delete_record (solo desde cancel)
```
**NOTA:** `force_delete` contexto NO funciona para purchase.order.
**API:**
```bash
# Siempre cancelar primero (incluso en draft)
execute_method.sh purchase.order button_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh purchase.order '[<id>]'
```

#### crm.lead
```
Cualquier estado →
  1. delete_record (sin restricciones)
```

#### project.task
```
Cualquier estado →
  1. delete_record (elimina subtareas en cascada)
  FALLA SI: tiene timesheets → borrar timesheets primero
```

#### hr.expense / hr.expense.sheet
```
Sheet estado: draft/submit →
  1. delete expenses
  2. delete sheet

Sheet estado: approve (con moves draft) →
  1. action_reset_expense_sheets (vuelve a draft)
  2. delete expenses + sheet

Sheet estado: approve (con moves posted) →
  No se puede rechazar/resetear
  → Requiere cancelar moves primero
```

### Modelos Extendidos

#### mrp.production
```
Estado: draft/confirmed →
  1. action_cancel
  2. delete_record

Estado: progress →
  1. action_cancel (workorders en cascada)
  2. button_draft
  3. delete_record

Estado: done →
  NO SE PUEDE CANCELAR FACILMENTE
  Opcion A: Crear mrp.unbuild (reversion de stock)
  Opcion B: Neutralizar (rename origin + active=False)
```
**API:**
```bash
execute_method.sh mrp.production action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh mrp.production button_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh mrp.production '[<id>]'
```

#### stock.scrap
```
Estado: draft → delete_record
Estado: done → NO reversible directo
  Opcion A: Crear unbuild o devolucion manual
  Opcion B: Neutralizar (rename + active=False si existe campo)
```

#### hr.leave (Ausencias)
```
Estado: draft/confirm →
  1. action_refuse
  2. action_draft
  3. delete_record

Estado: validate →
  1. action_refuse (restaura dias de vacaciones automaticamente)
  2. action_draft
  3. delete_record

Estado: refuse →
  1. action_draft
  2. delete_record
```
**API:**
```bash
execute_method.sh hr.leave action_refuse '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh hr.leave action_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh hr.leave '[<id>]'
```

#### hr.payslip (Nominas)
```
Estado: draft → delete_record
Estado: done →
  1. action_payslip_cancel (cancela move asociado)
  2. action_payslip_draft (vuelve a draft)
  3. delete_record
  FALLA SI: move posted → cancelar move primero
```
**API:**
```bash
execute_method.sh hr.payslip action_payslip_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh hr.payslip action_payslip_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh hr.payslip '[<id>]'
```

#### hr.contract (Contratos)
```
Cualquier estado →
  1. write state=cancel
  2. delete_record (verificar que no hay nominas activas)
```
**API:**
```bash
update_record.sh hr.contract '[<id>]' '{"state": "cancel"}'
delete_record.sh hr.contract '[<id>]'
```

#### pos.order
```bash
# Solo si sesion abierta y order en draft/paid (no posted)
execute_method.sh pos.order action_pos_order_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
# Si sesion cerrada → NO se puede cancelar via ORM
# Alternativa: crear refund
execute_method.sh pos.order refund '[[<id>]]' '{}'
```

#### account.asset
```bash
# Pasar a borrador
execute_method.sh account.asset set_to_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
# Eliminar (solo en draft)
delete_record.sh account.asset '[<id>]'
# Si tiene asientos de amortizacion publicados:
# 1. Revertir asientos con button_draft/button_cancel
# 2. Luego set_to_draft del asset
# 3. Luego delete
```

#### mailing.mailing
```bash
# Cancelar mailing (solo si no esta done)
execute_method.sh mailing.mailing action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
# Pasar a draft
execute_method.sh mailing.mailing action_set_as_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
# Eliminar (borrar trazas primero si falla)
delete_record.sh mailing.mailing '[<id>]'
# NOTA: Si state=done, los emails ya fueron enviados — irrecuperables
```

#### helpdesk.ticket
```bash
# Borrar timesheets si existen
TIMESHEET_IDS=$(search_records.sh account.analytic.line '[["helpdesk_ticket_id","=",<id>]]' '["id"]' 0)
delete_record.sh account.analytic.line '[<timesheet_ids>]'
# Borrar ticket
delete_record.sh helpdesk.ticket '[<id>]'
```

#### quality.check
```bash
# Borrar directamente
delete_record.sh quality.check '[<id>]'
```

#### maintenance.request
```bash
# Borrar directamente (actividades/followers se borran via aux cleanup)
delete_record.sh maintenance.request '[<id>]'
```

#### documents.document
```bash
# Borrar directamente
delete_record.sh documents.document '[<id>]'
```

#### planning.slot
```bash
# Borrar directamente
delete_record.sh planning.slot '[<id>]'
```

#### approval.request
```bash
# Si estado != draft, rechazar primero
execute_method.sh approval.request action_refuse '[[<id>]]' '{"context": {"tracking_disable": true}}'
# Volver a draft si es posible
execute_method.sh approval.request action_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
# Borrar
delete_record.sh approval.request '[<id>]'
```

#### fleet.vehicle.log.services / fleet.vehicle.log.fuel
```bash
# Borrar directamente
delete_record.sh fleet.vehicle.log.services '[<id>]'
delete_record.sh fleet.vehicle.log.fuel '[<id>]'
```

#### stock.warehouse.orderpoint
```bash
# Eliminar directamente (no tiene workflow de estados)
delete_record.sh stock.warehouse.orderpoint '[<id>]'
```

---

