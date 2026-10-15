# Compatibilidad de API por Version de Odoo

> **Ultima actualizacion:** 2026-03-09
> **Versiones cubiertas:** 14.0, 15.0, 16.0, 17.0, 18.0

---

## 1. Cambios Relevantes por Version

### 14.0 -> 15.0

| Area | Cambio | Impacto en testing |
|------|--------|-------------------|
| `sale.order` | Se introduce el wizard `sale.order.cancel` para cancelar SO confirmados. Antes, `action_cancel()` cancelaba directamente sin wizard | Usar contexto `disable_cancel_warning=True` para bypass del wizard via API |
| `account.move` | Unificacion completa de facturas y asientos contables bajo `account.move` (iniciada en 13.0, consolidada en 14.0-15.0). Metodo `action_post()` reemplaza definitivamente a `post()` | Usar siempre `action_post()`, nunca `post()` |
| `stock.move` | Campo `product_uom` renombrado a `product_uom_id` (~15.0) | Verificar nombre del campo segun version |
| `purchase.order` | Sin cambios significativos en API de transicion de estados | -- |
| `hr.expense` | Refactorizacion del flujo de gastos. `action_submit_expenses` empieza a migrar hacia `action_submit_sheet` | Verificar nombre del metodo segun version |
| `project.task` | Sistema de stages (kanban) sin cambios mayores. Campo `state` aun no existe como selection independiente | Usar `stage_id` para controlar estado |

### 15.0 -> 16.0

| Area | Cambio | Impacto en testing |
|------|--------|-------------------|
| `account.move` | Introduccion de `inalterable_hash` para journals con `restrict_mode_hash_table=True`. Los asientos hasheados no pueden volver a draft | Verificar `inalterable_hash` antes de intentar `button_draft()` |
| `account.move` | Campo `posted_before` introducido para audit trail | Afecta eliminacion si `check_account_audit_trail=True` |
| `hr.leave` | Metodo `action_draft()` renombrado a `action_reset_confirm()` (~16.0, verificar) | Usar `action_reset_confirm()` en 16.0+ |
| `sale.order` | Campo `locked` se gestiona con `action_lock()` / `action_unlock()` | Verificar que SO no este locked antes de cancelar |
| `stock.picking` | `button_validate()` introduce logica de auto-fill: si `quantity == 0`, rellena con `product_uom_qty` automaticamente | Simplifica testing de validacion |
| `mrp.production` | Sin cambios significativos en API de estados | -- |

### 16.0 -> 17.0

| Area | Cambio | Impacto en testing |
|------|--------|-------------------|
| **product.product** | **CAMBIO CRITICO:** `type='product'` (storable) se elimina. Ahora se usa `type='consu'` + `is_storable=True` para productos almacenables | Todos los flujos que crean productos storable deben actualizarse |
| `project.task` | **CAMBIO CRITICO:** Se introduce campo `state` como selection con valores: `01_in_progress`, `02_changes_requested`, `03_approved`, `1_done`, `1_canceled`, `04_waiting_normal`. El campo es computed con inverse | Cambiar de manipular solo `stage_id` a usar tambien `state` |
| `hr.expense.sheet` | Refactorizacion mayor: `action_sheet_move_create()` se elimina (~17.0). La aprobacion (`_do_approve`) ahora crea moves automaticamente. Se introduce `action_sheet_move_post()` para publicar | Actualizar flujo completo de gastos |
| `account.move` | `need_cancel_request` introducido para e-factura (localizaciones). Bloquea `button_draft()` si devuelve True | Verificar antes de resetear a draft en instancias con localizacion fiscal |
| `account.payment` | Estados expandidos: se introducen `in_process`, `paid`, `canceled`, `rejected` como estados separados (~17.0, verificar) | Verificar estados disponibles segun version |
| `stock.picking` | Contexto `skip_sanity_check` disponible para bypass de validaciones en testing | Util para tests automatizados |

### 17.0 -> 18.0

| Area | Cambio | Impacto en testing |
|------|--------|-------------------|
| **product.product** | Consolidacion del cambio: `type='product'` ya no existe como valor valido. Solo `consu` (con `is_storable`) y `service` | Obligatorio usar `is_storable=True` para productos almacenables |
| `sale.order` | `action_done` eliminado. El estado `done` ya no existe. Se usa `action_lock()` para bloquear SO | No intentar transicionar a `done` |
| `sale.order` | `action_draft()` acepta estados `cancel` y `sent` como precondicion. Filtra silenciosamente (no lanza error si otro estado) | Tener en cuenta el filtrado silencioso |
| `purchase.order` | `button_approve()` puede poner PO directamente en `done` si `company.po_lock == 'lock'` | Verificar `po_lock` de la compania para anticipar estado final |
| `account.move` | Deteccion de montos anomalos deshabilitada por defecto (`disable_abnormal_invoice_detection` default True). Via API el wizard no aparece | No es necesario el bypass via API |
| `account.move` | `partner_id` requerido en `_post()` para invoices (no en `create()`) | Incluir `partner_id` siempre antes de publicar |
| `mrp.production` | Estado `to_close` documentado entre `progress` y `done`. `unlink()` llama `action_cancel()` automaticamente — solo `done` es irremovible | Simplifica limpieza: solo `done` bloquea eliminacion |
| `hr.expense.sheet` | Flujo definitivo: submit -> approve (crea moves draft) -> post (publica moves) -> done. `action_sheet_move_create` **NO EXISTE** | Usar `action_sheet_move_post()` para publicar |
| `hr.leave` | Estado inicial es `confirm` (To Approve), NO `draft`. `action_draft()` **NO EXISTE** — usar `action_reset_confirm()` desde `cancel` o `refuse` | Ajustar todos los tests que asuman estado inicial `draft` |
| `hr.leave` | Estado `cancel` disponible en hr.leave pero NO en hr.leave.allocation | Diferente manejo de cancelacion segun modelo |
| `project.task` | Campo `state` es computed con `_compute_state`. Depende de `depend_on_ids` (tareas bloqueantes). Si tiene dependencias abiertas, auto-cambia a `04_waiting_normal` | El state puede cambiar automaticamente por dependencias |
| `stock.move` | Restriccion de `unlink()` con `@api.ondelete`: solo permite borrar en `draft`/`cancel` si tiene `move_orig_ids` o `move_dest_ids` | Desconectar moves de la cadena antes de eliminar, o cancelar primero |

### 18.0 -> 19.0

> Validado contra codigo fuente Odoo 19.0

| Area | Cambio | Impacto en testing |
|------|--------|-------------------|
| **`hr.leave`** | **NO hay renombrado en 19.0** (corregido 2026-08-06, verificado contra fuente). `date_from`/`date_to` y `request_date_from`/`request_date_to` **coexisten desde v17** con la misma definicion: los primeros son `compute='_compute_date_from_to', store=True` y el core avisa *"should therefore never be set directly"* (`hr_holidays/models/hr_leave.py:174-175`, `HolidaysRequest.date_from`, v17 @3e1233ed1; v19 `hr_holidays/models/hr_leave.py:172-173` @d1d161033). **Y `request_unit_hours`, `tz_mismatch` y `private_name` NO son nuevos de 19.0** (corregido 2026-08-17): los tres se declaran ya en v17 y v18, un `fields.` por campo y major. Esa lista era la unica parte sin medir del parrafo | Escribir `date_from`/`date_to` esta mal **en todas las versiones**, no solo en 19.0: usar siempre `request_date_from`/`request_date_to` |
| **`hr.expense`** | **CAMBIO IMPORTANTE:** Campo `state` es ahora completamente computed (no escribible directamente). El campo escribible es `approval_state` (Selection: 'draft', 'submitted', 'approved', 'refused'). Nuevos estados: `'in_payment'`, `'paid'` como estados intermedios post-contabilizacion | No escribir `state` directamente. Usar metodos de accion: `action_submit()`, `action_approve()`, `action_refuse()`. Nuevos campos: `approval_date`, `split_expense_origin_id`, `payment_mode` |
| **`account.move`** | `action_post()` puede retornar un wizard de confirmacion si el importe es anomalo (`abnormal_amount_warning=True`). En 18.0 estaba deshabilitado por defecto; en 19.0 se activa mas facilmente. Nuevos campos computed: `abnormal_amount_warning`, `abnormal_date_warning` | Incluir siempre `disable_abnormal_invoice_detection: True` en contexto al publicar facturas via API |
| **`account.move`** | `@api.ondelete` adicionales en 19.0: `_unlink_account_audit_trail_except_once_post()` verifica `posted_before` + audit trail con mayor granularidad | Borrado de asientos mas restrictivo. El patron de Tier 3 SQL sigue siendo el fallback para casos bloqueados |
| **`stock.picking`** | **NADA DE ESTO ES NUEVO EN 19.0** (corregido 2026-08-15, verificado contra fuente). El campo `is_locked` existe desde **14.0** (`stock/models/stock_picking.py`: `odoo14:372`, `odoo18:662`, `odoo19:658`); `action_cancel()` escribe `is_locked: True` desde 14.0 también; y `action_toggle_is_locked()` está desde 14.0 con el cuerpo idéntico. Lo único que cambia en 19.0 es que el campo pasa a `copy=False` (`odoo19:658`) | Verificar `is_locked` antes de cancelar o validar un picking — pero **en todas las versiones**, no solo en 19.0. Un test que dé por hecho que en 18.0 el picking no se bloquea al cancelar está equivocado |
| **`mrp.production`** | Nuevo campo `production_group_id` (Many2one a `mrp.production.group`). Nuevo campo `reservation_state` (Selection: 'confirmed', 'assigned', 'waiting') mas granular que el campo previo. Estados del modelo sin cambios (draft, confirmed, progress, to_close, done, cancel) | `reservation_state` reemplaza parcialmente la logica de `availability`. Verificar disponibilidad de materiales via `reservation_state` |
| **`crm.lead`** | Nuevos campos: `phone_state`, `email_state` (Selection), `won_status` (Selection: 'won', 'lost', 'pending'), `lost_reason_id` (Many2one). Nueva constraint: leads en Won stage deben tener `probability = 100` | Si se crean leads en stage Won en tests, incluir `probability: 100` |
| **`repair.order`** | Nuevo campo `repair_properties` (fields.Properties) — no requerido para create. Campo `parts_availability_state` (Selection) nuevo. Metodos de accion preservados: `action_repair_done`, `action_repair_cancel`, `action_repair_start`, `action_repair_end`, `action_validate` | Sin cambios rupturistas en metodos de flujo |
| **`project.task`** | Campo `state` con los mismos valores que en 18.0 (`01_in_progress`, `1_done`, etc.). Nuevo campo `is_closed` (Boolean computed: True si state en CLOSED_STATES). `_compute_is_closed()` como metodo alternativo para verificar cierre | Usar `is_closed` para filtrar tareas cerradas mas facilmente: `domain: [["is_closed", "=", True]]` |
| **`sale.order`** | Nuevos campos: `prepayment_percent` (Float, con constraint 0-100), `pending_email_template_id` (Many2one). `action_cancel()` valida adicionalmente que la orden no este locked antes de cancelar | Si SO esta locked (`locked=True`), ejecutar primero `action_unlock()` antes de cancelar |
| **`maintenance`** | Nuevo campo `equipment_properties_definition` (PropertiesDefinition) en `maintenance.equipment.category`. `@api.ondelete` en categoria de equipos. Campos nuevos: `mtbf`, `mttr`, `estimated_next_failure`, `latest_failure_date` | Sin cambios rupturistas en flujos de mantenimiento |
| **`product.product`** | Sin cambios en API de producto. `is_storable=True` sigue siendo correcto para productos almacenables | Sin cambios |

---

## 2. Tabla de Metodos por Modelo y Version

### sale.order

| Metodo | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|--------|------|------|------|------|------|------|-------|
| `action_confirm()` | SI | SI | SI | SI | SI | SI | Estable en todas las versiones |
| `action_cancel()` | SI (directo) | SI (wizard) | SI (wizard) | SI (wizard) | SI (wizard) | SI (wizard+locked check) | Desde 15.0~ muestra wizard. En 19.0: verifica adicionalmente que `locked=False`. Bypass: `disable_cancel_warning=True` |
| `_action_cancel()` | SI | SI | SI | SI | SI | SI | Metodo interno. Estable |
| `action_done()` | SI | SI | SI | SI~ | NO | NO | Eliminado en 18.0. Usar `action_lock()` |
| `action_draft()` | SI | SI | SI | SI | SI | SI | Precondicion: `cancel` o `sent`. Filtra silenciosamente |
| `action_lock()` | NO | NO | SI~ | SI | SI | SI | `self.locked = True` |
| `action_unlock()` | NO | NO | SI~ | SI | SI | SI | `self.locked = False`. **En 19.0: ejecutar antes de `action_cancel()` si SO esta locked** |
| `unlink()` | draft/cancel | draft/cancel | draft/cancel | draft/cancel | draft/cancel | draft/cancel | Estable en todas las versiones |

### purchase.order

| Metodo | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|--------|------|------|------|------|------|------|-------|
| `button_confirm()` | SI | SI | SI | SI | SI | SI | Estable. Puede ir a `to approve` o `purchase` segun `_approval_allowed()` |
| `button_approve()` | SI | SI | SI | SI | SI | SI | Bypass de doble validacion. Resultado puede ser `done` si `po_lock=='lock'` |
| `button_cancel()` | SI | SI | SI | SI | SI | SI | Sin restriccion de estado propio. Solo valida facturas asociadas |
| `button_draft()` | SI | SI | SI | SI | SI | SI | Sin restricciones especiales |
| `button_done()` | SI | SI | SI | SI | SI | SI | Pone PO en `done` (locked) |
| `button_unlock()` | SI~ | SI~ | SI | SI | SI | SI | Devuelve PO a `purchase` |
| `action_create_invoice()` | SI | SI | SI | SI | SI | SI | Crea factura de proveedor |
| `unlink()` | solo cancel | solo cancel | solo cancel | solo cancel | solo cancel | solo cancel | Estable. Mas restrictivo que sale.order (no permite draft) |

### account.move

| Metodo | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|--------|------|------|------|------|------|------|-------|
| `action_post()` | SI | SI | SI | SI | SI | SI (puede devolver wizard) | En 19.0: si `abnormal_amount_warning=True`, devuelve wizard de confirmacion. Bypass: contexto `disable_abnormal_invoice_detection=True` |
| `post()` | Deprecado | NO | NO | NO | NO | NO | Usar `action_post()` |
| `button_draft()` | SI | SI | SI | SI | SI | SI | **El guard NO es de ~16.0 y la condicion cambia en v18** — ver la tabla de abajo (remedido 2026-08-17) |

> **Cuando bloquea `button_draft()` un asiento hasheado — medido 2026-08-17, y no es lo que decia
> esta pagina.** El apunte `Desde ~16.0 verifica inalterable_hash via _check_draftable()` era una
> aproximacion (la tilde lo delataba) y falla por los dos lados: ni empieza en v16, ni la condicion
> es la misma en v17 que en v18+.
>
> | major | ¿bloquea? | Condicion real | Cita |
> |---|---|---|---|
> | v16 | **NO** | `button_draft` (`:3908`) solo corta por diferencia de cambio (`:3934`) y tax cash basis (`:3941`). **Ningun guard mira el hash**, aunque el campo `inalterable_hash` ya existe (`:256`) | `odoo16/addons/account/models/account_move.py` |
> | v17 | SI, **pero por el diario, no por el hash** | se extrae a `_check_draftable` (`:4229`) y corta con `if move.restrict_mode_hash_table and move.state == 'posted'` → *"You cannot modify a posted entry of this journal because it is in strict mode"* | `:4263-4264 @3e1233ed1` |
> | v18 | SI, **por el hash del asiento** | `if move.inalterable_hash:` → *"You cannot reset to draft a locked journal entry"* | `:5691 @99f5a33c64` |
> | v19 | idem v18 | idem | `:6277 @d1d161033` |
>
> **La diferencia importa al montar un test:** en **v17** un asiento posted en un diario con
> `restrict_mode_hash_table` esta bloqueado **aunque todavia no tenga hash**; en **v18/v19** solo lo
> esta si `inalterable_hash` tiene valor. O sea que el mismo escenario de prueba —diario en modo
> estricto, asiento recien posteado sin hashear— **pasa a draft en 18/19 y no en 17**.

| `button_cancel()` | SI | SI | SI | SI | SI | SI | Internamente hace `button_draft()` + cancel |
| `_reverse_moves()` | SI | SI | SI | SI | SI | SI | Reversion directa sin wizard |
| `_check_draftable()` | NO | NO | SI~ | SI | SI | SI | Introducido ~16.0. Verifica hash, exchange_move, tax_cash_basis |
| `unlink()` | SI | SI | SI | SI | SI | SI (mas restrictivo) | En 19.0: `_unlink_account_audit_trail_except_once_post()` con mayor granularidad. `posted_before=True` bloquea con audit trail |
| `action_validate_moves_with_confirmation()` | NO | NO | NO | NO | NO | SI | Nuevo en 19.0. Wizard de confirmacion para montos anomalos |

### stock.picking

| Metodo | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|--------|------|------|------|------|------|------|-------|
| `action_confirm()` | SI | SI | SI | SI | SI | SI | Estable |
| `action_assign()` | SI | SI | SI | SI | SI | SI | Auto-confirma si esta en draft |
| `button_validate()` | SI | SI | SI | SI | SI | SI | Auto-fill de cantidades si quantity==0 |
| `action_cancel()` | SI | SI | SI | SI | SI | SI (escribe is_locked) | En 19.0: al cancelar, escribe `is_locked=True` en el picking. Estable para flujos de test |
| `action_toggle_is_locked()` | SI | SI | SI | SI | SI | SI | **Estable desde 14.0 al menos** — no es nuevo en 19.0. Cuerpo idéntico en 14/15/16/17/18/19: `ensure_one()` + `self.is_locked = not self.is_locked`. Verificado 2026-08-15 en `stock/models/stock_picking.py` (`odoo14:1031`, `odoo16:1188`, `odoo18:1542`, `odoo19:1529`) |
| `unlink()` | SI | SI | SI | SI | SI | SI | Cancela moves automaticamente antes de eliminar |

### stock.move

| Metodo | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|--------|------|------|------|------|------|------|-------|
| `_action_confirm()` | SI | SI | SI | SI | SI | SI | Metodo interno (prefijo `_`) |
| `_action_done()` | SI | SI | SI | SI | SI | SI | Metodo interno |
| `_action_cancel()` | SI | SI | SI | SI | SI | SI | ERROR si `state=='done'` y no scrapped. Estable |
| `unlink()` | SI | SI | SI | SI | SI | SI | Restriccion: solo draft/cancel si tiene move_orig/dest_ids |

### mrp.production

| Metodo | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|--------|------|------|------|------|------|------|-------|
| `action_confirm()` | SI | SI | SI | SI | SI | SI | Estable. En 19.0: ajustes para UoM en tracking serial |
| `action_cancel()` | SI | SI | SI | SI | SI | SI | ERROR si `state=='done'`. Estable |
| `button_mark_done()` | SI | SI | SI | SI | SI | SI | Puede mostrar wizard `mrp.immediate.production` o `mrp.production.backorder` |
| `action_toggle_is_locked()` | SI | SI | SI | SI | SI~ | SI | Permite editar MO confirmada |
| `unlink()` | draft/cancel | draft/cancel | draft/cancel | draft/cancel~ | cualquier excepto done | cualquier excepto done | En 18.0+, `unlink()` llama `action_cancel()` automaticamente |

### project.task

| Aspecto | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|---------|------|------|------|------|------|------|-------|
| Sistema de estados | Solo `stage_id` | Solo `stage_id` | Solo `stage_id` | `stage_id` + `state` | `stage_id` + `state` (computed) | `stage_id` + `state` (computed) + `is_closed` | En 19.0: nuevo campo `is_closed` (Boolean computed: True si state in CLOSED_STATES) |
| Estados disponibles | N/A | N/A | N/A | `01_in_progress`, `02_changes_requested`, `03_approved`, `1_done`, `1_canceled`, `04_waiting_normal` | Igual que 17.0 | Igual que 17.0-18.0 | `state` es computed con inverse. `is_closed` es alternativa para filtrar |
| `toggle_active()` | SI | SI | SI | SI | SI | SI | Archivar/desarchivar. Estable |
| `action_archive()` | NO | NO | NO | NO | NO | SI | Nuevo en 19.0. Equivalente a `toggle_active()` cuando `active=True` |
| `unlink()` | Sin restriccion | Sin restriccion | Sin restriccion | Sin restriccion | Sin restriccion | Sin restriccion | Elimina subtareas automaticamente en todas las versiones |

### hr.expense / hr.expense.sheet

| Metodo | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|--------|------|------|------|------|------|------|-------|
| `action_submit_expenses()` | SI | SI | Deprecado~ | NO | NO | NO | Renombrado. Usar metodo de sheet |
| `action_submit_sheet()` | NO~ | SI~ | SI | SI | SI | SI | Metodo del sheet para enviar a aprobacion |
| `approve_expense_sheets()` | SI | SI | SI~ | Deprecado~ | NO | NO | Renombrado en versiones recientes |
| `action_approve_expense_sheets()` | NO~ | SI~ | SI | SI | SI | `action_approve()` | **CAMBIO en 19.0:** metodo renombrado a `action_approve()` (verificar). El campo directo es `approval_state`, no `state` |
| `action_sheet_move_create()` | SI | SI | SI | SI~ | **NO EXISTE** | **NO EXISTE** | **ELIMINADO desde 18.0.** Los moves se crean en la aprobacion |
| `action_sheet_move_post()` | NO | NO | NO | NO~ | SI | SI | Publica los moves creados en la aprobacion |
| `action_reset_expense_sheets()` | SI | SI | SI | SI | SI | SI | Estable |

**Campo `state` en hr.expense (19.0):**
En 19.0 el campo `state` del gasto es **completamente computed** y refleja: draft → submitted → approved → in_payment → paid / refused. El campo escribible para la aprobacion es `approval_state`.

### hr.leave

| Metodo | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 | Notas |
|--------|------|------|------|------|------|------|-------|
| `action_approve()` | SI | SI | SI | SI | SI | SI | Si `validation_type=='both'` -> state='validate1'; sino `action_validate()` |
| `action_validate()` | SI | SI | SI | SI | SI | SI | Segunda aprobacion (o unica si validation_type!='both') |
| `action_refuse()` | SI | SI | SI | SI | SI | SI | Estable |
| `action_draft()` | SI | SI | Deprecado~ | NO | **NO EXISTE** | **NO EXISTE** | Renombrado a `action_reset_confirm()` desde ~16.0 |
| `action_reset_confirm()` | NO | NO | SI~ | SI | SI | SI | Precondicion: `cancel` o `refuse`. Resultado: `confirm` |
| `action_cancel()` | NO~ | NO~ | SI~ | SI | SI | SI | Abre wizard de cancelacion |
| `unlink()` | confirm | confirm | confirm + condiciones | confirm + condiciones | confirm/validate1/cancel | confirm/validate1/cancel | Restricciones granulares en versiones recientes |

**Estado inicial y campos de fecha por version:**

| Version | Estado inicial | Campo de fecha inicio | Campo de fecha fin | Notas |
|---------|---------------|----------------------|--------------------|-------|
| 14.0-16.0 | `confirm` | `date_from` | `date_to` | Sin `draft` en hr.leave |
| **17.0-19.0** | `confirm` | **`request_date_from`** | **`request_date_to`** | `date_from`/`date_to` existen pero son **computed y no se escriben** (verificado v17 y v19). No es un cambio de 19.0 |

---

## 3. Campos Renombrados/Eliminados por Version

| Modelo | Campo | Version del cambio | Antes | Despues | Notas |
|--------|-------|-------------------|-------|---------|-------|
| `product.product` | `type` (storable) | **18.0** | `type='product'` | `type='consu'` + `is_storable=True` | **CRITICO, y el corte es v18 no v17** (corregido 2026-08-06). En 17.0 `stock` sigue anadiendo `('product', 'Storable Product')` por `selection_add` (`stock/models/product.py:658-663`, `ProductTemplate`, v17 @3e1233ed1) |
| `product.product` | `is_storable` | **18.0** | No existia | `is_storable` (Boolean, `compute='compute_is_storable'`, `store=True`, `readonly=False`) | `stock/models/product.py:704`, `ProductTemplate.is_storable`, @99f5a33c64. **No existe en 17.0** (0 ocurrencias) |
| `stock.move` | `product_uom` | ~15.0 | `product_uom` | `product_uom_id` | Renombrado para consistencia con convencion `_id`. Verificar version exacta |
| `hr.expense` | `unit_amount` / `total_amount` | ~17.0 | `unit_amount` como campo principal | `total_amount` / `total_amount_currency` preferido | Ambos funcionan pero tests 18.0 usan `total_amount` |
| `hr.expense.sheet` | `state` valores | ~17.0-18.0 | `draft`, `submit`, `approve`, `post`, `done`, `cancel` | Idem pero es computed | Estado del sheet se computa desde `approval_state` y `account_move_ids` |
| `account.move` | `inalterable_hash` | ~16.0 | No existia | `inalterable_hash` (Char) | Hash criptografico que bloquea `button_draft()`. Introducido con `restrict_mode_hash_table` |
| `account.move` | `posted_before` | ~16.0 | No existia | `posted_before` (Boolean) | Usado por audit trail para impedir eliminacion de asientos previamente publicados |
| `account.move` | `need_cancel_request` | ~17.0 | No existia | `need_cancel_request` (Computed) | Hook de localizacion para e-factura. Bloquea `button_draft()` |
| `account.move` | `secured` | ~16.0 | No existia | `secured` (Computed Boolean) | True si `inalterable_hash` tiene valor |
| `sale.order` | `state` valores | 18.0 | `draft`, `sent`, `sale`, `done`, `cancel` | `draft`, `sent`, `sale`, `cancel` | Estado `done` eliminado en 18.0. Reemplazado por `locked=True` |
| `sale.order` | `locked` | ~16.0 | No existia como campo separado | `locked` (Boolean) | Antes se usaba `state='done'` para bloquear |
| `project.task` | `state` | 17.0 | No existia (solo `stage_id`) | `state` (Selection, computed) | Valores: `01_in_progress`, `02_changes_requested`, `03_approved`, `1_done`, `1_canceled`, `04_waiting_normal` |
| `project.task` | `CLOSED_STATES` | 17.0 | No existia | `{'1_done': 'Done', '1_canceled': 'Cancelled'}` | Constante del modelo para identificar estados de cierre |
| `mrp.production` | `state` valor `to_close` | ~17.0-18.0 | No existia (verificar) | `to_close` (To Close) | Entre `progress` y `done`. Indica produccion completa pero MO no cerrada |
| `stock.picking` | `picked` | ~17.0-18.0 | No existia | `picked` (Boolean en stock.move) | Se auto-establece en `button_validate()` si hay quantity pero no picked |
| `account.payment` | `state` valores | ~17.0 | `draft`, `posted`, `sent`, `reconciled`, `cancelled` | `draft`, `in_process`, `paid`, `canceled`, `rejected` | Refactorizacion significativa de estados |
| `account.payment` | `state='cancelled'` | ~17.0 | `cancelled` (doble L) | `canceled` (una L) | Cambio ortografico. Verificar version exacta |

---

## 4. Wizards por Version

| Wizard | Version introducida | Reemplaza a | Notas |
|--------|-------------------|-------------|-------|
| `sale.order.cancel` | ~15.0 | Cancelacion directa de `action_cancel()` | Muestra confirmacion antes de cancelar SO confirmado. Bypass: `disable_cancel_warning=True` |
| `sale.advance.payment.inv` | 14.0 (estable) | -- | Wizard para crear factura desde SO. Estable en todas las versiones |
| `sale.order.discount` | ~17.0-18.0 | -- | Aplicar descuento global. No critico para testing |
| `account.move.reversal` | 14.0 (estable) | -- | Wizard de reversion de asientos. Metodo `refund_moves()`. Requiere `company_id` (required=True) |
| `validate.account.move` | ~17.0 | -- | Confirmacion por montos anomalos. Deshabilitado por defecto via API (`disable_abnormal_invoice_detection` default True) |
| `stock.return.picking` | 14.0 (estable) | -- | Devoluciones de picking done. Estable |
| `stock.backorder.confirmation` | 14.0 (estable) | -- | Validacion parcial de picking. Bypass: `skip_backorder=True` en contexto |
| `mrp.immediate.production` | 14.0 (estable) | -- | Si no se establece `qty_producing`. Estable |
| `mrp.production.backorder` | 14.0 (estable) | -- | Si qty_producing < product_qty. Contexto: `mo_ids_to_backorder` (NO `skip_backorder` para MRP) |
| `hr.expense.refuse.wizard` | ~16.0 (verificar) | Rechazo directo | `action_refuse_expense_sheets()` devuelve wizard, no rechaza directamente. El rechazo real es via `_do_refuse(reason)` |

---

## 5. Contextos Especiales por Version

| Contexto | Disponible desde | Modelo | Efecto | Notas |
|----------|-----------------|--------|--------|-------|
| `tracking_disable=True` | 14.0 | Todos (mail) | Deshabilita tracking en chatter | Recomendado para todos los tests API |
| `mail_notrack=True` | 14.0 | Todos (mail) | Evita tracking de cambios | Alternativa a `tracking_disable` |
| `mail_create_nolog=True` | 14.0 | Todos (mail) | No crea log de creacion | Complementario a `mail_notrack` |
| `no_reset_password=True` | 14.0 | res.users | No envia mail de reset password | Usado al crear usuarios de test |
| `disable_cancel_warning=True` | ~15.0 | sale.order | Salta wizard `sale.order.cancel` | Indispensable para cancelar SO via API |
| `force_delete=True` | ~16.0 | account.move | Bypass de restricciones de cadena de secuencia y audit trail en `unlink()` | Usar con precaucion. No bypasea lock dates |
| `skip_invoice_sync=True` | ~16.0 | account.move | Evita sincronizacion con facturas | Usado internamente por `unlink()` |
| `dynamic_unlink=True` | ~16.0 | account.move | Permite borrado dinamico de lineas | Usado internamente por `unlink()` |
| `disable_abnormal_invoice_detection=True` | ~17.0 | account.move | Salta wizard de montos anomalos | Default True — via API no aparece el wizard |
| `skip_backorder=True` | 14.0 | stock.picking | Salta wizard de backorder | Para stock. MRP usa `mo_ids_to_backorder` |
| `cancel_backorder=True/False` | ~16.0 (verificar) | stock.picking | Decide si cancelar backorders en `_action_done()` | Usado internamente tras `skip_backorder` |
| `skip_sanity_check=True` | ~17.0 (verificar) | stock.picking | Salta validaciones de sanidad en `button_validate()` | Util para testing automatizado |
| `picking_ids_not_to_backorder` | 14.0~ | stock.picking | Lista de picking IDs que NO deben generar backorder | Alternativa a `skip_backorder` |
| `mo_ids_to_backorder` | ~17.0 (verificar) | mrp.production | Lista de MO IDs que deben generar backorder. Sin este contexto, ninguna genera backorder | Para MRP, NO usar `skip_backorder` |
| `skip_activity=True` | ~17.0 | mrp.production | Salta log de actividad en cancelacion | Util para operaciones masivas |
| `leave_skip_date_check=True` | ~16.0 (verificar) | hr.leave | Salta validacion de fechas en `unlink()` | Permite borrar ausencias pasadas |
| `prefetch_fields=False` | 14.0 | Todos (ORM) | Evita prefetch de campos en operaciones masivas | Optimizacion de rendimiento para borrados masivos |

---

## 6. Guia Rapida de Creacion de Productos por Version

Dado que el cambio en `product.product` es critico y afecta multiples modulos:

```
Version 14.0 - 17.0:
  Producto almacenable: {'type': 'product', 'name': '...'}
  Producto consumible:  {'type': 'consu', 'name': '...'}
  Servicio:             {'type': 'service', 'name': '...'}

Version 18.0+:
  Producto almacenable: {'type': 'consu', 'is_storable': True, 'name': '...'}
  Producto consumible:  {'type': 'consu', 'name': '...'}   (is_storable default False)
  Servicio:             {'type': 'service', 'name': '...'}

NOTA: 'type=product' genera ValueError en 18.0+, NO en 17.0 (corregido 2026-08-06
      contra fuente: en v17 lo anade stock por selection_add y is_storable no existe)
```

---

## 7. Matriz de Compatibilidad Resumida

Leyenda: SI = disponible y funcional, NO = no existe, ~ = verificar version exacta, RENAMED = renombrado

| Funcionalidad | 14.0 | 15.0 | 16.0 | 17.0 | 18.0 |
|--------------|------|------|------|------|------|
| `sale.order.action_confirm()` | SI | SI | SI | SI | SI |
| `sale.order.action_cancel()` directo | SI | wizard | wizard | wizard | wizard |
| `sale.order` estado `done` | SI | SI | SI | SI~ | NO |
| `purchase.order.button_confirm()` | SI | SI | SI | SI | SI |
| `account.move.action_post()` | SI | SI | SI | SI | SI |
| `account.move` hash fiscal | NO | NO | SI~ | SI | SI |
| `account.move` audit trail | NO | NO | SI~ | SI | SI |
| `stock.picking.button_validate()` auto-fill | NO | NO | NO~ | SI~ | SI |
| `product.product` type='product' | SI | SI | SI | NO | NO |
| `product.product` is_storable | NO | NO | NO | SI | SI |
| `project.task.state` selection | NO | NO | NO | SI | SI |
| `hr.leave.action_draft()` | SI | SI | NO~ | NO | NO |
| `hr.leave.action_reset_confirm()` | NO | NO | SI~ | SI | SI |
| `hr.expense.sheet.action_sheet_move_create()` | SI | SI | SI | SI~ | NO |
| `hr.expense.sheet.action_sheet_move_post()` | NO | NO | NO | NO~ | SI |
| Approve crea moves (expense) | NO | NO | NO | NO~ | SI |

---

## 8. Metodos Privados vs RPC

Los metodos con prefijo `_` son privados en Odoo. Aunque tecnicamente pueden ser invocados via XML-RPC/JSON-RPC, **NO deben usarse** porque pueden cambiar sin aviso entre versiones y no tienen las mismas validaciones que los metodos publicos.

| Metodo privado | Modelo | Alternativa RPC | Notas |
|---------------|--------|-----------------|-------|
| `_action_cancel` | sale.order | `action_cancel` + ctx `{'disable_cancel_warning': True}` | Sin contexto devuelve wizard `sale.order.cancel` en vez de cancelar |
| `_create_invoices` | sale.order | Wizard `sale.advance.payment.inv` → `create_invoices` | Contexto requerido: `{"active_model": "sale.order", "active_ids": [<so_id>]}` |
| `_action_confirm` | stock.move | No hay alternativa publica directa | Solo usar desde `stock.picking.action_confirm()` |
| `_action_done` | stock.move | No hay alternativa publica directa | Solo usar desde `stock.picking.button_validate()` |
| `_action_cancel` | stock.move | No hay alternativa publica directa | Solo usar desde `stock.picking.action_cancel()` |
| `_reverse_moves` | account.move | Wizard `account.move.reversal` → `refund_moves` | Metodo directo tolerable para reversion automatica |
| `_do_approve` | hr.expense.sheet | `action_approve_expense_sheets` | El metodo publico invoca `_do_approve` internamente |

**Regla general:** Si existe un metodo publico (sin prefijo `_`) o un wizard que cumple la misma funcion, **usar siempre la alternativa publica**. Los metodos privados solo son aceptables como ultimo recurso documentado.

---

*Generado: 2026-03-09 | Basado en: analisis de codigo fuente Odoo 14.0-18.0 y documentos de fase del skill odoo-flow-tester*
*Precision: Los datos de 18.0 estan verificados contra codigo fuente. Las versiones 14.0-17.0 se basan en conocimiento general de la evolucion de Odoo y pueden contener imprecisiones marcadas con "~" o "verificar".*
