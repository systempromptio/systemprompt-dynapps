## Constraints Reales de Odoo 18.0 por Modelo (del codigo fuente)

### account.move (Facturas/Asientos)

**Constraints de borrado (`@api.ondelete`):**

1. **`_unlink_forbid_parts_of_chain()`** (L3488-3506): No se puede borrar un asiento con numero de secuencia
   a menos que sea el ultimo de la cadena. Bypass: pertenecer al grupo `account.group_account_manager`,
   o que la empresa tenga `quick_edit_mode` activo, o pasar `force_delete=True` en el contexto.
   Error: `"You cannot delete this entry, as it has already consumed a sequence number and is not the last one in the chain."`

2. **`_unlink_account_audit_trail_except_once_post()`** (L3508-3517): No se puede borrar si
   `posted_before=True AND company.check_account_audit_trail=True`. Bypass: `force_delete=True` en contexto.
   Error: `"To keep the audit trail, you can not delete journal entries once they have been posted."`

3. **`unlink()`** (L3519-3527): Borra `line_ids` primero, luego `super().unlink()`.

4. **Entries con `inalterable_hash`**: Solo bloquean `button_draft()` (L1791), NO bloquean `unlink()`.
   Si el move esta en draft, se puede borrar. Si esta posted, la opcion ORM es `_reverse_moves(cancel=True)`.

**Prerequisitos de `button_draft()`:**
- `state in ('cancel', 'posted')`
- No `need_cancel_request`
- No exchange diff entries (via `account_full_reconcile.exchange_move_id` y `account_partial_reconcile.exchange_move_id`)
- No tax cash basis entries (`tax_cash_basis_rec_id` o `tax_cash_basis_origin_move_id`)
- No `inalterable_hash`

**Metodo `_unlink_or_reverse()` — Estrategia de 3 niveles:**
1. Si `_can_be_unlinked()` (draft, no hash, date > lock_date) → `button_draft()` → `unlink()`
2. Si `_is_protected_by_audit_trail()` (`posted_before` + audit trail) → solo `button_cancel()` (no borra)
3. Si tiene hash → `_reverse_moves()` (no borra, crea reversal)

**SQL fallback:** DELETE directo respetando orden FK:
`account_partial_reconcile` → `account_move_line` → `account_move`.
Rompe cadena de hash — solo con aprobacion explicita.

**Puntos de no retorno:**
- `inalterable_hash` + periodo fiscal cerrado
- Factura enviada a EDI (SII/TicketBAI)

### sale.order

**Constraint de borrado (`@api.ondelete` L1004-1010):**
- `_unlink_except_draft_or_cancel()`: Solo permite borrar en estados `draft` o `cancel`.
  Error: `"You can not delete a sent quotation or a confirmed sales order. You must first cancel it."`

**Cancelacion:**
- `action_cancel()` (L1273) muestra wizard si no draft; `disable_cancel_warning` ctx lo salta.
  `_action_cancel()` (L1311) cancela facturas draft asociadas.
  NO puede cancelar pedidos con `order.locked == True` (L1281).
  Bypass: primero `action_unlock()` o `write({'locked': False})`.
- NO tiene `button_draft` nativo.
- **⚠️ NOTA RPC:** `_action_cancel` es privado y NO funciona correctamente via JSON-RPC. Usar siempre `action_cancel` con contexto `{'disable_cancel_warning': True}` para bypass del wizard.

**Registros hijos cascade:** `sale_order_line.order_id` ondelete=cascade (sale_order_line.py L41).

**SQL fallback:** DELETE de `sale_order_line` → `sale_order`.

**Puntos de no retorno:** Ninguno (siempre cancelable si se desbloquea y pickings/facturas revertidos primero).

### purchase.order

**Constraint de borrado (`@api.ondelete` L295-299):**
- `_unlink_if_cancelled()`: SOLO permite borrar en estado `cancel` (MAS ESTRICTO que sale.order).
  A diferencia de SO, ni siquiera el estado `draft` permite borrar sin cancelar primero.
  Error: `"In order to delete a purchase order, you must cancel it first."`

**Cancelacion:**
- `button_cancel()` (L553) falla si hay facturas asociadas en estado != (cancel, draft) (L554-556).
  → Primero cancelar facturas asociadas.
- `button_draft()` (L534) sin restricciones adicionales.

**Registros hijos cascade:** `purchase_order_line.order_id` ondelete=cascade (purchase_order_line.py L47).

**SQL fallback:** `UPDATE account_move_line SET purchase_line_id = NULL` → DELETE de `purchase_order_line` → `purchase_order`.

**Puntos de no retorno:** Ninguno (siempre cancelable si facturas estan canceladas).

### stock.move

**Constraint de borrado (`@api.ondelete` L2130-2133):**
- `_unlink_if_draft_or_cancel()`: No se puede borrar si tiene `move_orig_ids` o `move_dest_ids`
  en estado no-draft/cancel.
  Error: `"You can not delete moves linked to another operation"`

**`unlink()` (L2135-2138):** Borra `move_line_ids` primero, luego `super()`.

**Cancelacion:**
- `_action_cancel()` (L1997-1999) lanza `UserError` en moves con `state='done' and not scrapped`.
  Error: `"You cannot cancel a stock move that has been set to 'Done'. Create a return in order to reverse the moves which took place."`
  NUNCA intentar `_action_cancel()` en moves done → usar `stock.return.picking` wizard.
- La cancelacion se propaga downstream si `propagate_cancel=True`.

**SQL fallback:** DELETE de `stock_move_move_rel` → `stock_move_line` → `stock_move`.

**Puntos de no retorno:** `state=done` sin scrap (requiere devolucion obligatoria).

### stock.picking (Albaranes)

**Mecanismo interno (verificado en fuente):**
- `unlink()` (L1152-1155) auto-llama `move_ids._action_cancel()` y luego `move_ids.unlink()`.
  **Sin `@api.ondelete`** — no valida estado directamente, pero los moves done fallan en `_action_cancel`.
- `action_cancel()` (L1198) delega en `move_ids._action_cancel()`.
- Ambos fallan si algun move esta en `state=done and not scrapped`.

**Done pickings:** OBLIGATORIO crear devolucion via `stock.return.picking` wizard.
No existe otra opcion ORM para revertir un picking validado.

**Registros hijos:** `stock_move.picking_id` ondelete=set null (NO cascade — hay que borrar manualmente).
`stock_move_line.picking_id` ondelete=set null (NO cascade).

**SQL fallback:** DELETE de `stock_move_line` → `stock_move` → `stock_picking`.
Requiere recalculo posterior de `stock.quant`.

**Puntos de no retorno:** `state=done` (requiere devolucion, no cancelacion).

### mrp.production

**Constraints de borrado:**
- `_unlink_if_not_done()` (`@api.ondelete at_uninstall=True` L1062-1065): ERROR si `state == 'done'`.
  Error: `"You cannot delete a manufacturing order that is already done."`
- `_unlink_except_done()` (`@api.ondelete at_uninstall=False` L1365-1370): ERROR si `state == 'done'`.
  Error: `"Cannot delete a manufacturing order in done state."`
- `unlink()` (L1055-1060): Llama `action_cancel()` primero, luego borra workorders no done.

**Cancelacion:**
- `action_cancel()` (L1740) ERROR si `state == 'done'` (L1743-1744).

**Puntos de no retorno:** `state=done` = no se puede cancelar NI borrar. Alternativa: `mrp.unbuild`.

### pos.order

**Constraint de borrado (`@api.ondelete` L501-504):**
- `_unlink_except_draft_or_cancel()`: Solo permite borrar en `draft` o `cancel`.
  Error: `"In order to delete a sale, it must be new or cancelled."`

**`pos.order.line` (`@api.ondelete` L1536-1539):**
- `_unlink_except_order_state()`: Solo si `order_id.state in ["draft", "cancel"]`.
  Error: `"You can only unlink PoS order lines that are related to orders in new or cancelled state."`

**Registros hijos cascade:** `pos_order_line.order_id` ondelete=cascade (L1401). `pos_payment.pos_order_id` ondelete=cascade (pos_payment.py L20).

### hr.expense / hr.expense.sheet

**hr.expense (`@api.ondelete` L609-613):**
- `_unlink_except_posted_or_approved()`: ERROR si `state in {'done', 'approved'}`.
  Error: `"You cannot delete a posted or approved expense."`
- `sheet_id` FK a hr.expense.sheet (sin cascade, ondelete=set null, L100-107).

**hr.expense.sheet (`@api.ondelete` L525-529):**
- `_unlink_except_posted_or_paid()`: ERROR si `state in {'post', 'done'}`.
  Error: `"You cannot delete a posted or paid expense."`

### hr.leave

**Constraint de borrado (`@api.ondelete` L835-849):**
- `_unlink_if_correct_states()`:
  - **Usuarios normales** (sin group_hr_holidays_user): solo `confirm`, `validate1`, `cancel` Y NO en el pasado.
  - **HR managers**: solo `cancel` o `confirm`.
  Error: `"You cannot delete a time off which is in %s state"` / `"You cannot delete a time off which is in the past"`
- `unlink()` (L851-853): Llama `_post_leave_cancel()` antes de borrar.

---

