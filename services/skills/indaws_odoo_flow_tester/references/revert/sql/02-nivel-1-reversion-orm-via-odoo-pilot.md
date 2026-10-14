## NIVEL 1 — Reversion ORM via odoo-pilot

### Secuencia estandar de reversion

Para la mayoria de modelos transaccionales, la secuencia es:

```
1. action_cancel / button_cancel  (estado → cancel)
2. button_draft                   (estado → draft)
3. unlink (delete_record.sh)      (eliminar registro)
```

### Contextos especiales para operaciones ORM

```bash
# Evitar generacion de mensajes en chatter durante reversion
execute_method.sh <model> <method> '[[<id>]]' '{"context": {"tracking_disable": true}}'

# Evitar creacion de logs de mail
execute_method.sh <model> <method> '[[<id>]]' '{"context": {"mail_create_nolog": true}}'

# Forzar borrado de account.move ignorando constraints de cadena y audit trail
# IMPORTANTE: force_delete SOLO funciona para account.move, NO para otros modelos
delete_record.sh account.move '[<id>]' '{"context": {"force_delete": true}}'

# Desactivar advertencia de cancelacion en ventas
execute_method.sh sale.order action_cancel '[[<id>]]' '{"context": {"disable_cancel_warning": true}}'

# Combinacion recomendada para reversion limpia
execute_method.sh <model> <method> '[[<id>]]' '{"context": {"tracking_disable": true, "mail_create_nolog": true, "mail_notrack": true}}'
```

### Cuando unlink() falla: Estrategia de Neutralizacion

Si `delete_record.sh` falla por constraint FK o regla de negocio:

```bash
# Paso 1: Desactivar el registro (no visible en busquedas normales)
update_record.sh <model> '[<id>]' '{"active": false}'

# Paso 2: Renombrar para identificar como neutralizado
update_record.sh <model> '[<id>]' '{"name": "[NEUTRALIZED-TEST-YYYYMMDD] <nombre_original>"}'

# Paso 3 (opcional): Mover a journal/warehouse de test si aplica
update_record.sh account.move '[<id>]' '{"journal_id": <test_journal_id>}'
update_record.sh stock.picking '[<id>]' '{"picking_type_id": <test_picking_type_id>}'
```

### Orden de dependencias FK (eliminar en este orden)

Basado en el esquema real de Odoo 18.0. Las FK mas criticas estan marcadas.

**Cadena critica contable (RESPETAR SIEMPRE):**
```
account.partial.reconcile → account.move.line → account.move → sale.order.line → sale.order
```

**NOTA sobre `force_delete` contexto:**
El contexto `force_delete=True` SOLO funciona para `account.move` (bypass de `_unlink_forbid_parts_of_chain`
y `_unlink_account_audit_trail_except_once_post`). NO tiene efecto en otros modelos como `sale.order`,
`purchase.order`, `stock.picking`, etc.

**NOTA sobre `inalterable_hash`:**
ANTES de intentar DELETE SQL de un `account.move`, verificar SIEMPRE si tiene `inalterable_hash`:
```sql
SELECT id, inalterable_hash, secure_sequence_number FROM account_move WHERE id IN (@move_ids);
```
Si `inalterable_hash IS NOT NULL`, borrar via SQL rompe la cadena criptografica del diario.
Solo proceder si se acepta perder validacion de integridad fiscal para ese diario.

```
 PASO  TABLA                              DEPENDENCIA FK CRITICA
 ─────────────────────────────────────────────────────────────────────────
  1    account.partial.reconcile          → account.move.line (debit/credit_move_id)
  2    account.full.reconcile             (auto-borrado al borrar todos los partials de un grupo)
  3    stock.valuation.layer              → stock.move, account.move (auto-ref primero)
       3a: UPDATE stock_valuation_layer SET stock_valuation_layer_id = NULL WHERE ...
       3b: DELETE stock_valuation_layer WHERE ...
  4    account.payment                    → account.move (move_id)
  5    account.move.line                  → account.move (move_id, cascade en ORM pero NO en SQL directo)
  6    account.move                       (facturas, asientos)
  7    stock.move.line                    → stock.move (move_id), stock.picking (picking_id)
  8    stock.move                         → stock.picking (picking_id)
       8a: DELETE FROM stock_move_move_rel WHERE ...  (M2M move_orig/dest_ids)
       8b: DELETE FROM stock_move WHERE ...
  9    stock.scrap                        → stock.picking, mrp.production
 10    stock.package_level                → stock.picking
 11    stock.picking                      → stock.picking (auto-ref: backorder_id)
       11a: UPDATE stock_picking SET backorder_id = NULL WHERE ...
       11b: DELETE stock_picking WHERE ...
 12    mrp.workcenter.productivity        → mrp.workorder
 13    mrp.workorder                      → mrp.production
 14    mrp.unbuild                        → mrp.production (mo_id)
 15    mrp.production                     (indirecta via stock.move)
 16    pos.payment                        → pos.order (cascade)
 17    pos.order.line                     → pos.order (cascade)
 18    pos.order                          → stock.picking, account.move
 19    sale.order.line                    → sale.order (cascade)
 20    sale.order
 21    purchase.order.line                → purchase.order (cascade)
 22    purchase.order
 23    hr.expense                         → hr.expense.sheet (sheet_id)
 24    hr.expense.sheet                   → account.move (account_move_ids)
 25    hr.payslip                         → account.move (move_id)
 26    hr.leave
 27    mailing.trace                      → mailing.mailing
 28    mailing.mailing
```

**Modelos adicionales** (sin FK criticas, borrar en cualquier orden despues):
```
 29    hr.contract                 ← Contratos RRHH
 30    account.asset               ← Activos fijos
 31    maintenance.request         ← Solicitudes de mantenimiento
 32    helpdesk.ticket             ← Tickets de soporte
 33    quality.check               ← Controles de calidad
 34    fleet.vehicle.log.services  ← Servicios de flota
 35    crm.lead                    ← Oportunidades
 36    project.task                ← Tiene auto-referencia (parent_id) y FK → account.analytic.line
```

---

### Reversion por Modelo — Nivel 1

#### sale.order

```bash
# Estado draft → borrar directamente
delete_record.sh sale.order '[<id>]'

# Estado sale/sent → cancelar primero
execute_method.sh sale.order action_cancel '[[<id>]]' '{"context": {"disable_cancel_warning": true, "tracking_disable": true}}'
delete_record.sh sale.order '[<id>]'

# Si unlink falla (pickings done, facturas posted):
# 1. Revertir pickings primero (ver stock.picking)
# 2. Revertir facturas primero (ver account.move)
# 3. Luego cancelar y borrar SO
```

#### purchase.order

```bash
# Estado draft → cancelar y borrar
execute_method.sh purchase.order button_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh purchase.order '[<id>]'

# Estado purchase/done → cancelar (requiere facturas canceladas primero)
# 1. Cancelar facturas asociadas
# 2. Cancelar recepciones pendientes
execute_method.sh purchase.order button_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh purchase.order '[<id>]'
```

#### account.move (Facturas / Asientos)

```bash
# Estado draft → borrar directamente (con force_delete si posted_before + audit trail)
delete_record.sh account.move '[<id>]' '{"context": {"force_delete": true}}'

# Estado posted → pasar a draft → cancelar → borrar
# PRE-CHECK: Verificar inalterable_hash y posted_before
search_records.sh account.move '[["id","=",<id>]]' '["state","inalterable_hash","posted_before"]' 1

# CASO ESTANDAR (sin hash):
execute_method.sh account.move button_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
# button_draft() L5525: desreconcilia via remove_move_reconcile, limpia analitica
# FALLA SI: inalterable_hash, exchange entry, cash basis, need_cancel_request
execute_method.sh account.move button_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh account.move '[<id>]' '{"context": {"force_delete": true}}'

# Si button_draft FALLA por inalterable_hash (L1791):
# Opcion A: Crear nota de credito (reversal) para saldo neto 0
execute_method.sh account.move _reverse_moves '[[<id>], null, true]' '{"context": {"tracking_disable": true}}'
# Opcion B: Neutralizar (no borrar)
update_record.sh account.move '[<id>]' '{"ref": "[NEUTRALIZED-TEST] original_ref"}'
# Opcion C: Escalar a Nivel 2 o Nivel 3

# NOTA: inalterable_hash solo bloquea button_draft, NO bloquea unlink.
# Si el move esta en draft, unlink funciona normalmente.
```

#### account.payment

```bash
# Estado draft → borrar directamente
delete_record.sh account.payment '[<id>]'

# Estado posted/in_process/paid → cancelar → draft → borrar
# action_cancel: estado→canceled, borra moves draft, cancela moves posted
execute_method.sh account.payment action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
# action_draft: estado→draft, llama move.button_draft() que desreconcilia automaticamente
execute_method.sh account.payment action_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh account.payment '[<id>]'

# Flujo simplificado (si reconciliado con factura):
# action_draft ya desreconcilia automaticamente via button_draft → remove_move_reconcile
execute_method.sh account.payment action_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh account.payment '[<id>]'
```

#### stock.picking (Albaranes)

```bash
# Estado draft/waiting/confirmed/assigned → cancelar y borrar
execute_method.sh stock.picking action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh stock.picking '[<id>]'

# Estado done → NO se puede cancelar, crear devolucion completa
# Paso 1: Crear wizard con contexto obligatorio
WIZARD_ID=$(create_record.sh stock.return.picking '{"picking_id": <id>}' '{"context": {"active_id": <id>, "active_model": "stock.picking"}}')
# Paso 2: Ejecutar devolucion completa (calcula cantidades automaticamente)
execute_method.sh stock.return.picking action_create_returns_all "[[$WIZARD_ID]]" '{"context": {"active_id": <id>, "active_model": "stock.picking"}}'
# Paso 3: Buscar picking de devolucion (usa campo return_id, NO backorder_id)
RETURN_ID=$(search_records.sh stock.picking '[["return_id","=",<id>]]' '["id","name","state"]' 1)
# Paso 4: Validar devolucion
execute_method.sh stock.picking button_validate "[[$RETURN_ID]]" '{"context": {"tracking_disable": true, "skip_backorder": true}}'
```

#### crm.lead

```bash
# Cualquier estado → borrar directamente (sin restricciones)
delete_record.sh crm.lead '[<id>]'
```

#### project.task

```bash
# Cualquier estado → borrar directamente (subtareas en cascada)
delete_record.sh project.task '[<id>]'

# Si tiene timesheets que impiden borrado:
# 1. Borrar timesheets primero
TIMESHEET_IDS=$(search_records.sh account.analytic.line '[["task_id","=",<id>]]' '["id"]' 0)
delete_record.sh account.analytic.line '[<timesheet_ids>]'
# 2. Luego borrar tarea
delete_record.sh project.task '[<id>]'
```

#### hr.expense / hr.expense.sheet

```bash
# Sheet en draft/submit → borrar expenses y sheet
delete_record.sh hr.expense '[<expense_ids>]'
delete_record.sh hr.expense.sheet '[<sheet_id>]'

# Sheet en approve (moves draft) → resetear → borrar
execute_method.sh hr.expense.sheet action_reset_expense_sheets '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh hr.expense '[<expense_ids>]'
delete_record.sh hr.expense.sheet '[<sheet_id>]'

# Sheet en approve (moves posted) → cancelar moves primero
# 1. Buscar moves del sheet
MOVE_IDS=$(search_records.sh account.move '[["expense_sheet_id","=",<sheet_id>]]' '["id"]' 0)
# 2. Cancelar moves (ver account.move)
# 3. Luego resetear y borrar sheet
```

#### pos.order

```bash
# Solo si sesion abierta y order en draft/paid
execute_method.sh pos.order action_pos_order_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh pos.order '[<id>]'

# Si sesion cerrada → crear refund
execute_method.sh pos.order refund '[[<id>]]' '{}'

# Si no se puede borrar → neutralizar
update_record.sh pos.order '[<id>]' '{"note": "[NEUTRALIZED-TEST]"}'
```

#### account.asset

```bash
# Estado draft → borrar directamente
delete_record.sh account.asset '[<id>]'

# Estado open → pasar a draft → borrar
execute_method.sh account.asset set_to_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh account.asset '[<id>]'

# Si tiene asientos de amortizacion publicados:
# 1. Buscar depreciation moves
DEPRECIATION_MOVE_IDS=$(search_records.sh account.move '[["asset_id","=",<id>],["state","=","posted"]]' '["id"]' 0)
# 2. Revertir asientos (ver account.move)
# 3. Luego set_to_draft → delete
```

#### mailing.mailing

```bash
# Estado draft/in_queue → cancelar y borrar
execute_method.sh mailing.mailing action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh mailing.mailing action_set_as_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh mailing.mailing '[<id>]'

# Estado done → emails ya enviados, solo borrar registro
# Los emails enviados son irrecuperables
delete_record.sh mailing.mailing '[<id>]'

# Si falla el borrado: buscar y borrar trazas primero
TRACE_IDS=$(search_records.sh mailing.trace '[["mass_mailing_id","=",<id>]]' '["id"]' 0)
delete_record.sh mailing.trace '[<trace_ids>]'
delete_record.sh mailing.mailing '[<id>]'
```

#### mrp.production

```bash
# Estado draft/confirmed → cancelar y borrar
execute_method.sh mrp.production action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh mrp.production '[<id>]'

# Estado progress → cancelar (workorders se cancelan en cascada)
execute_method.sh mrp.production action_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh mrp.production button_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh mrp.production '[<id>]'

# Estado done → NO se puede cancelar facilmente
# Opcion A: Crear mrp.unbuild
UNBUILD_ID=$(create_record.sh mrp.unbuild '{"mo_id": <id>, "product_id": <product_id>, "product_qty": <qty>, "product_uom_id": <uom_id>, "location_id": <location_id>, "location_dest_id": <location_dest_id>}')
execute_method.sh mrp.unbuild action_unbuild "[[$UNBUILD_ID]]" '{"context": {"tracking_disable": true}}'
# Opcion B: Neutralizar
update_record.sh mrp.production '[<id>]' '{"origin": "[NEUTRALIZED-TEST] original_origin"}'
```

#### stock.scrap

```bash
# Estado draft → borrar directamente
delete_record.sh stock.scrap '[<id>]'

# Estado done → NO reversible directo
# Opcion A: Crear unbuild o devolucion manual
# Opcion B: Neutralizar
update_record.sh stock.scrap '[<id>]' '{"name": "[NEUTRALIZED-TEST] original_name"}'
```

#### hr.leave (Ausencias)

```bash
# Estado draft/confirm → rechazar y borrar
execute_method.sh hr.leave action_refuse '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh hr.leave action_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh hr.leave '[<id>]'

# Estado validate → rechazar primero (restaura dias de vacaciones)
execute_method.sh hr.leave action_refuse '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh hr.leave action_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh hr.leave '[<id>]'
```

#### hr.payslip

```bash
# Estado draft → borrar directamente
delete_record.sh hr.payslip '[<id>]'

# Estado done → cancelar → draft → borrar
execute_method.sh hr.payslip action_payslip_cancel '[[<id>]]' '{"context": {"tracking_disable": true}}'
execute_method.sh hr.payslip action_payslip_draft '[[<id>]]' '{"context": {"tracking_disable": true}}'
delete_record.sh hr.payslip '[<id>]'

# Si tiene moves posted → cancelar moves primero (ver account.move)
```

#### fleet.vehicle.log.services

```bash
# Cualquier estado → borrar directamente
delete_record.sh fleet.vehicle.log.services '[<id>]'
```

#### maintenance.request

```bash
# Cualquier estado → borrar directamente
delete_record.sh maintenance.request '[<id>]'
```

#### helpdesk.ticket

```bash
# Cualquier estado → borrar directamente
delete_record.sh helpdesk.ticket '[<id>]'

# Si tiene timesheets:
TIMESHEET_IDS=$(search_records.sh account.analytic.line '[["helpdesk_ticket_id","=",<id>]]' '["id"]' 0)
delete_record.sh account.analytic.line '[<timesheet_ids>]'
delete_record.sh helpdesk.ticket '[<id>]'
```

#### quality.check

```bash
# Cualquier estado → borrar directamente
delete_record.sh quality.check '[<id>]'
```

#### subscription (sale.order con recurrence)

```bash
# Cerrar suscripcion primero
execute_method.sh sale.order set_close '[[<id>]]' '{"context": {"tracking_disable": true}}'
# Cancelar
execute_method.sh sale.order action_cancel '[[<id>]]' '{"context": {"disable_cancel_warning": true, "tracking_disable": true}}'
# Borrar
delete_record.sh sale.order '[<id>]'

# Si tiene facturas con hash fiscal → no se pueden borrar
# Neutralizar suscripcion en su lugar
update_record.sh sale.order '[<id>]' '{"client_order_ref": "[NEUTRALIZED-TEST] original_ref"}'
```

---

### Limpieza de registros auxiliares (Nivel 1)

Para cualquier modelo, limpiar registros auxiliares ANTES de borrar el registro principal:

```bash
# Mensajes del chatter
MSG_IDS=$(search_records.sh mail.message '[["model","=","<model>"],["res_id","=",<id>]]' '["id"]' 0)
delete_record.sh mail.message '[<msg_ids>]'

# Seguidores
FOLLOWER_IDS=$(search_records.sh mail.followers '[["res_model","=","<model>"],["res_id","=",<id>]]' '["id"]' 0)
delete_record.sh mail.followers '[<follower_ids>]'

# Actividades programadas
ACTIVITY_IDS=$(search_records.sh mail.activity '[["res_model","=","<model>"],["res_id","=",<id>]]' '["id"]' 0)
delete_record.sh mail.activity '[<activity_ids>]'

# Adjuntos
ATTACH_IDS=$(search_records.sh ir.attachment '[["res_model","=","<model>"],["res_id","=",<id>]]' '["id"]' 0)
delete_record.sh ir.attachment '[<attach_ids>]'
```

---

