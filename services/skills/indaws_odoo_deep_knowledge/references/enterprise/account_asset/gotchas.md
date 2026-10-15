# Account Asset Gotchas — Odoo 18.0 Enterprise — EE
<!-- verified: v18.0 @ 2026-06 -->

## Depreciation Computation

**ASG1: 30-day month assumption breaks real-world expectations** (`account_asset.py → DAYS_PER_MONTH/DAYS_PER_YEAR`, `_get_delta_days`) _ES: Las cuotas de amortización no cuadran con los días reales del calendario: un activo a 5 años sale como 1800 días, no 1825._
When `prorata_computation_type != 'daily_computation'`, Odoo uses 30 days/month and 360 days/year (`DAYS_PER_MONTH = 30`, `DAYS_PER_YEAR = DAYS_PER_MONTH * 12`, `account_asset/models/account_asset.py:15-16`, `DAYS_PER_MONTH`). `_get_delta_days` (`:724`) prorates partial months against the *real* month length (`end_of(start_date,'month').day`) but scales each month to 30. A 5-year asset = 1800 days, not 1825/1826. This means depreciation amounts in constant_periods mode do NOT match actual calendar days. Use `daily_computation` if calendar accuracy matters.

**ASG2: Prorata date silently resets to fiscal year start when prorata_computation_type='none'** (`account_asset.py → _compute_prorata_date`) _ES: La fecha de inicio de amortización se cambia sola al 1 de enero (inicio del ejercicio) en vez de respetar la fecha de compra._
With `prorata_computation_type='none'` and an `acquisition_date`, `_compute_prorata_date` (`:273`) sets `prorata_date = company_id.compute_fiscalyear_dates(acquisition_date).get('date_from')`. An asset acquired on Nov 15 with calendar FY gets prorata_date = Jan 1. Every other value (`'constant_periods'`, `'daily_computation'`) keeps `prorata_date = acquisition_date`. This changes the depreciation schedule dramatically and is often unexpected.

**ASG3: asset_lifetime_days is 0 when method_number is 0 — division by zero guard** (`account_asset.py → _compute_board_amount`) _ES: Error de división por cero al calcular la amortización de un activo con cero periodos configurados._
`_compute_board_amount` returns `0, 0` early when `float_is_zero(self.asset_lifetime_days, 2) or float_is_zero(residual_amount, 2)` (`:583`). But `_get_linear_amount` (`:552`) divides by `self.asset_lifetime_days` with no guard, so any custom code calling it directly with a 0-lifetime asset will crash. Assets with "0 depreciations" are valid (they don't lose value) but rare.

**ASG4: Rounding accumulation strategy — cumulative vs per-period** (`account_asset.py → _get_linear_amount`) _ES: Las cuotas de amortización varían un céntimo entre periodos y el último periodo ajusta el resto para cuadrar el valor residual._
`_get_linear_amount` (`:552`) computes `amount_after_expected - round(amount_expected_previous_period)` (both derived from `total_depreciable_value * days / asset_lifetime_days`). This avoids rounding drift over time but means individual period amounts may differ by 0.01. The last period is forced to the residual via `_get_depreciation_amount_end_of_lifetime` (`:1186`), absorbing any remaining difference.

**ASG5: degressive_then_linear child assets use parent's transition point** (`account_asset.py → _compute_board_amount`) _ES: En amortización degresiva y luego lineal, revalorizar el activo padre desplaza el punto donde el hijo pasa a lineal._
For a child (`parent_id` set) with `degressive_then_linear`, the code (`:610-627`) reads `parent_cumulative_depreciation` / `parent_depreciable_value` from the parent's moves at `mv.date <= self.prorata_date` and scales the child's depreciable value by `(1 + parent_cumulative/parent_depreciable)`, then by `asset_lifetime_days / parent.asset_lifetime_days`. If `parent_depreciable_value` is zero (`currency.is_zero`), it falls back to pure linear for the child (`:619`). Revaluing the parent can shift the child's degressive-to-linear transition.

**ASG6: already_depreciated_amount_import silently skips initial depreciation entries** (`account_asset.py → _recompute_board`) _ES: Tras importar amortización previa, las primeras cuotas salen a cero o reducidas sin avisar._
In `_recompute_board` (`:649`), when there are no posted moves, `imported_amount = already_depreciated_amount_import` is subtracted from the first computed amounts until consumed (`:678-687`): if `abs(imported) <= abs(amount)` it deducts and zeroes import, else it sets `amount = 0` and carries the remainder. This can zero out several leading periods. Set it wrong at migration and the first N entries are reduced/zero with no warning.

## Fiscal Year & Period Issues

**ASG7: Fiscal year mismatch silently shifts depreciation dates** (`account_asset.py → _get_end_period_date`) _ES: Cambiar el ejercicio fiscal de la empresa no recalcula el cuadro de amortización de los activos ya existentes._
`_get_end_period_date` (`:710`) uses `company_id.compute_fiscalyear_dates(start).get('date_to')`. If the company's fiscal year is Apr-Mar and the asset starts in June, the first yearly period ends March 31. Changing the company fiscal year after assets exist does NOT recompute existing boards (no recompute trigger on company FY config).

**ASG8: Monthly depreciation capped at fiscal year end** (`account_asset.py → _get_end_period_date`, `_recompute_board`) _ES: La amortización mensual se corta a fin de ejercicio fiscal y eso altera el cálculo degresivo del año siguiente._
When `method_period='1'`, `_get_end_period_date` returns `min(end_of_month, fiscal_year_end)` (`:719-721`). In `_recompute_board` (`:702-704`), when `period_end == period_end_fiscalyear_date` the loop resets `residual_declining = residual_amount` and bumps `start_yearly_period` by one year, which affects degressive calculations in the next FY.

**ASG9: Lock date blocks disposal but not depreciation posting** (`account_asset.py → set_to_close`) _ES: El cierre por fecha bloquea la baja del activo pero los asientos de amortización se quedan en borrador sin contabilizar._
`set_to_close` (`:887`) raises UserError if `disposal_date <= company_id._get_user_fiscal_lock_date(journal_id)`. But normal `validate()`/`compute_depreciation_board()` can create moves spanning past the lock date — those simply fail to post (stay draft / `auto_post`). Check for draft depreciation moves periodically.

## Disposal & Sale Traps

**ASG10: Disposal of asset with children requires disposing children first** (`account_asset.py → set_to_close`) _ES: No deja vender un activo si tiene revalorizaciones/hijos sin dar de baja primero, pero al desechar los cierra todos igualmente._
`set_to_close` (`:887`) sets `full_asset = self + self.children_ids` to 'close' (`:894-895`) and creates disposal moves for ALL. But it raises UserError *only* when `invoice_line_ids` is truthy (Sell) and a child is `state in ('draft','open') or value_residual > 0` (`:892`). So "Dispose" (no invoice lines) closes children regardless; "Sell" forces you to dispose each child increase first.

**ASG11: gain_account_id / loss_account_id on res.company are easily missed** (`account_asset.py → _get_disposal_moves`) _ES: La baja del activo genera un asiento descuadrado que falla al contabilizar porque faltan las cuentas de beneficio/pérdida en la empresa._
If `company_id.gain_account_id`/`loss_account_id` is unset, `difference_account` is empty (`:1162`) and `get_line` is skipped for falsy accounts (`:1173`, see ASG31). No upfront error — you get an unbalanced move that fails at posting with a cryptic "unbalanced" error. Note: `sell_dispose` *does* block one specific clash — gain/loss account equal to the depreciation account (`account_asset/wizard/asset_modify.py:395`, `AssetModify.sell_dispose`) — but not the empty case.

**ASG12: Disposal move date = disposal_date, not today** (`account_asset.py → _get_disposal_moves`) _ES: El asiento de baja usa la fecha de la baja indicada, no la de hoy, y puede caer en un periodo cerrado._
The disposal move uses `date: disposal_date` from the wizard, not `fields.Date.today()` (`:1168-1169`). If the user enters a past date, the entry may fall within a locked period — and `set_to_close` only checks the lock date for the disposal date itself (`:890`), against the asset's journal user fiscal lock date specifically.

**ASG13: net_gain_on_sale only set on parent, not children** (`account_asset.py → set_to_close`) _ES: El beneficio o pérdida de la venta solo se registra en el activo padre, no en cada activo hijo por separado._
`net_gain_on_sale = currency.round(selling_price - self.book_value)` is set on `self` only (`:904-905`), not on `full_asset`. Since `book_value` is recursive (`_compute_book_value`, `:329`, includes children), this is correct in total, but children individually don't track their gain/loss.

## State Machine & Workflow

**ASG14: Cancelling an asset reverses ALL posted moves, not just future ones** (`account_asset.py → set_to_cancelled`) _ES: Cancelar un activo revierte todos los asientos ya contabilizados, incluso de años anteriores en periodos cerrados._
`set_to_cancelled` (`:923`) calls `_cancel_future_moves(datetime.date.min)` (`:942`), which reverses EVERY posted non-reversed move regardless of date (the `m.date > date` filter is trivially true vs `date.min`, `:1107-1112`). This can create reversal entries in the current period for moves from years ago, potentially touching closed periods.

**ASG15: Pausing and resuming does NOT extend the asset lifetime** (`account_asset.py → _compute_paused_prorata_date`; `asset_modify.py → modify`) _ES: Pausar y reanudar un activo no alarga su vida útil: las cuotas restantes se apilan en menos periodos y salen más altas._
On resume, `modify` adds `number_days` to `asset_paused_days` (`account_asset/wizard/asset_modify.py:264-268`, `AssetModify.modify`), which shifts `paused_prorata_date` forward (`account_asset/models/account_asset.py:282`, `class AccountAsset`). `_get_last_day_asset` (`:750`) is `paused_prorata_date + method_period*method_number months - 1 day`, so the *whole* schedule slides forward by the pause gap; `method_number` is unchanged. Remaining depreciation compresses into fewer calendar periods → larger per-period amounts.

**ASG16: onchange_consistent_board wipes ALL draft depreciation moves** (`account_asset.py → onchange_consistent_board`) _ES: Al cambiar cualquier dato del activo en el formulario se borra todo el cuadro de amortización en borrador sin pedir confirmación._
Changing original_value, salvage_value, acquisition_date, method, method_progress_factor, method_period, method_number, prorata_computation_type, already_depreciated_amount_import or prorata_date triggers `onchange_consistent_board` (`:429`) which writes `{'depreciation_move_ids': [Command.set([])]}` (`:434`) — UNLINKING all (draft) moves. Accidentally change a value in the form and the board is wiped; no confirmation dialog.

**ASG17: validate() posts ALL depreciation moves including future ones** (`account_asset.py → validate`) _ES: Al validar el activo se contabilizan de golpe todas las cuotas, incluidas las de fechas futuras._
`validate` (`:855`) sets state 'open', computes the board if empty, then `depreciation_move_ids.filtered(state != 'posted')._post()` (`:880`). `compute_depreciation_board` only `_post()`s moves for open-state assets (`:645-647`); future-dated moves rely on the standard `auto_post` mechanism at posting time.

## Auto-Creation & Invoice Integration

**ASG18: Auto-create only fires on positive lines of invoices, models filtered by parent company** (`account_move.py → _auto_create_asset`) _ES: La creación automática de activos desde facturas no se dispara en abonos, líneas negativas ni modelos de otra empresa hermana._
`_auto_create_asset` (`:194`) requires `is_invoice()`, `can_create_asset`, `create_asset != 'no'`, non-zero `price_total`, no existing `asset_ids`, no `tax_line_id`, AND `price_total > 0` (`:203-211`). Credit-note/negative lines are skipped. Lines on `out_invoice`/`out_refund` whose account `internal_group == 'asset'` are also skipped (`:211`). Models are filtered to `model.company_id in move_line.company_id.parent_ids` (`:224`) — a model in a sibling company won't apply.

**ASG19: Multiple models per account create N assets per line** (`account_move.py → _auto_create_asset`) _ES: Una línea de factura genera varios activos multiplicados por cantidad y por número de modelos asociados a la cuenta._
The loop iterates `for model_id in model_ids or [None]` (`:234`), so 3 models on the account = 3 assets per qualifying line. Combined with `multiple_assets_per_line` (`units_quantity = max(1, int(quantity))`, `:218-222`), a line with qty=5 and 3 models creates 15 assets. Rarely intended.

**ASG20: Reversing/cancelling an invoice deactivates assets, doesn't delete them** (`account_move.py → button_cancel`; `account_asset.py → _check_active`) _ES: Al cancelar o revertir la factura el activo no se borra, solo se archiva, y luego no se puede reactivar limpio._
`button_cancel` (`account_asset/models/account_move.py:174`, `AccountMove.button_cancel`) does `search([('original_move_line_ids.move_id','in',self.ids)]).write({'active': False})` (`:177`). Assets stay in the DB, just archived. The `_check_active` constraint (`account_asset/models/account_asset.py:441`, `AccountAsset.onchange_consistent_board`) raises if you try to keep/unarchive a record whose `state not in ('close','model')`, so a running asset cannot simply be reactivated into a clean state.

## Currency & Multi-Company

**ASG21: Asset always uses company currency, not invoice currency** (`account_asset.py → currency_id`, `_compute_related_purchase_value`) _ES: El activo siempre usa la moneda de la empresa y no refleja cambios de tipo de cambio posteriores a la factura._
`currency_id = related='company_id.currency_id'` (`:30`). All values are in company currency. `_compute_related_purchase_value` (`:293`) sums `original_move_line_ids.mapped('balance')` (already converted to company currency at the invoice). Depreciation moves convert again at depreciation date (`_prepare_move_for_asset_depreciation`, `account_asset/models/account_move.py:268`, `AccountMove._prepare_move_for_asset_depreciation`). Subsequent exchange-rate changes are NOT reflected on the asset value.

**ASG22: Multi-company rule uses parent_of, not child_of** (`security/account_asset_security.xml`) _ES: En multiempresa un usuario de la filial ve los activos de sus hijas pero no los de la empresa matriz, al revés de lo esperado._
Both rules (`account_asset_multi_company_rule`, `account_asset_group_multi_company_rule`) use `domain_force = [('company_id', 'parent_of', company_ids)]` (`account_asset/security/account_asset_security.xml:8`, `account_asset_multi_company_rule` y `account_asset/security/account_asset_security.xml:15`, `account_asset_group_multi_company_rule`). A user in a child company sees assets of its company and its descendants — NOT the parent company's. Opposite of what some expect.

## Report & Display

**ASG23: Depreciation Schedule report uses raw SQL via _cr.execute and caches assets in a dict** (`account_assets_report.py → _query_values`, `_generate_report_lines_without_grouping`) _ES: Los campos personalizados añadidos al activo no aparecen en el informe de amortizaciones y el informe consume mucha memoria._
`_query_values` (`account_asset/models/account_assets_report.py:353`, `AssetsReportCustomHandler._query_values`) builds a `Query`/`SQL` manually and runs `self._cr.execute(sql)` (`:431`). Custom fields added to account.asset won't appear unless you also override the SQL. `_generate_report_lines_without_grouping` (`:52`) caches all assets in `assets_cache = {asset.id: asset for asset in browse(all_asset_ids)}` (`:74`), so large reports can consume significant memory.

**ASG24: Report groups children (gross increases) under parent automatically** (`account_assets_report.py → _query_lines`, `_get_parent_asset_values`) _ES: En el informe de amortizaciones las revalorizaciones (activos hijos) se agrupan dentro del padre y no salen como líneas propias._
`_query_lines` (`:155`) buckets rows where `al['parent_id']` is set into `children_lines[parent]` (`:164-167`) and `_get_parent_asset_values` (`:187`) folds each child's opening/depreciation/closing values into the parent row (`:228`). Children are not shown as separate top-level lines.

**ASG25: Book value shows 0 for fully depreciated closed assets (salvage removed)** (`account_asset.py → _compute_book_value`) _ES: Un activo totalmente amortizado y cerrado muestra valor contable 0 aunque tenga valor residual configurado._
`_compute_book_value` (`:329`) computes `value_residual + salvage_value + children book values`, then subtracts `salvage_value` again when `state == 'close'` AND all moves are posted (`:333-334`). A closed asset with salvage_value=500 shows book_value=0, not 500. Accounting-correct but confusing in the list view.

## Wizard-Specific

**ASG26: Modify wizard date + 1 day for gross increase and board restart** (`asset_modify.py → modify`) _ES: Al revalorizar un activo, el aumento y el nuevo cuadro empiezan el día siguiente a la fecha indicada, no ese mismo día._
On a value increase, the child asset gets `acquisition_date` and `prorata_date = self.date + relativedelta(days=1)` (`account_asset/wizard/asset_modify.py:326`, `AssetModify.modify`, y `:329`, mismo método), the revaluation move is dated `self.date + 1 day` (`account_asset/wizard/asset_modify.py:299`, `AssetModify.modify`), and the board recomputes from `restart_date = self.date + 1` (resume path uses `self.date`, `account_asset/wizard/asset_modify.py:357`, `AssetModify.modify`). Re-evaluate on Dec 31 → increase starts Jan 1.

**ASG27: Modify wizard creates and auto-validates gross increase in one step** (`asset_modify.py → modify`) _ES: El asistente de revalorización crea y contabiliza el aumento en un solo paso, sin borrador para revisar las cuentas._
The child asset is created (`:318`) and `asset_increase.validate()` is called immediately (`:339`), posting all its depreciation moves. No draft state for review. If the accounts are wrong you must cancel the child asset afterwards.

**ASG28: Resume after pause requires date strictly after pause date** (`asset_modify.py → modify`) _ES: Al reanudar un activo pausado da error si la fecha es igual o anterior a la fecha de la pausa._
`number_days = asset._get_delta_days(date_before_pause, self.date) - 1` (`:264`); the `-1` is because "pause+resume same day = 0 gap". If `compare_amounts(number_days, 0) < 0` it raises "You cannot resume at a date equal to or before the pause date" (`:265-266`). `date_before_pause` = max move date, else acquisition_date (`:260`).

**ASG29: Modify wizard pre-fills params but residual recomputes per date** (`asset_modify.py → create`, `_compute_value_residual`) _ES: En el asistente de modificación el valor residual mostrado cambia según la fecha que pongas._
`create` (`:216`) pre-populates `method_number`, `method_period`, `salvage_value`, and the three accounts from the asset when not provided (`:223-234`). But `value_residual` is `compute='_compute_value_residual'` depending on `date` (`:111-114`), calling `asset._get_residual_value_at_date(date)` — so the displayed residual changes as you change the wizard date.

**ASG30: _create_move_before_date can produce zero-amount entries that are silently skipped** (`account_asset.py → _create_move_before_date`) _ES: Si das de baja el mismo día de la última amortización, el asiento intermedio no se crea y el historial queda raro._
`_create_move_before_date` (`:1042`) computes prorated depreciation up to `date`; if `float_is_zero(amount, ...)` (`:1095`) no move is inserted. On disposal the same day as the last posted depreciation, the disposal move references no intermediate depreciation, which can look odd in the chatter.

## Disposal Move Internals (verified EE18 `account_asset/models/account_asset.py:1127-1178`, `AccountAsset._get_disposal_moves`, `def _get_disposal_moves`; remedido 2026-08-18, el rango empezaba en `:1115`, doce líneas antes del `def`)

**ASG31: Disposal lines with a falsy account are silently dropped from the move** (`account_asset.py → _get_disposal_moves`) _ES: Las líneas del asiento de baja con cuenta vacía desaparecen silenciosamente y el asiento queda descuadrado._
`_get_disposal_moves` builds `line_ids` with `[get_line(...) for amount, account in line_datas if account]` (`account_asset/models/account_asset.py:1173`, `AccountAsset._get_disposal_moves`). Any line whose `account` is `False` is simply not created. This is how the unbalanced-entry trap in ASG11 happens: if `gain_account_id`/`loss_account_id` is empty, the `difference` line vanishes and the move is unbalanced (fails at posting), with no upfront validation pointing at the missing company account.

**ASG32: initial_account picks the original invoice account only if it is unique** (`account_asset.py → _get_disposal_moves`) _ES: Si el activo se creó desde varias cuentas contables distintas, la baja no revierte correctamente contra las cuentas originales._
`initial_account = original_move_line_ids.account_id if len(...account_id) == 1 else account_asset_id` (`account_asset/models/account_asset.py:1149`, `AccountAsset._get_disposal_moves`). If the asset was created from invoice lines hitting more than one account, the disposal reverses against `account_asset_id` instead of the real original accounts. With multi-account origins, the disposal entry may not perfectly reverse the original capitalization. (Note `_compute_account_asset_id` at `:229` raises "All the lines should be from the same account" for direct-input assets, so multi-account origins arise from auto-created/edited assets.)

**ASG33: depreciated_amount in disposal includes already_depreciated_amount_import and uses copysign** (`account_asset.py → _get_disposal_moves`) _ES: El beneficio o pérdida de la baja sale mal si la amortización importada al migrar se configuró con un importe erróneo._
`account_asset/models/account_asset.py:1151-1155`, `AccountAsset._get_disposal_moves`: `depreciated_amount = currency.round(copysign(sum(moves where date<=disposal_date depreciation_value) + already_depreciated_amount_import, -initial_amount))`. The imported (pre-Odoo) depreciation is folded into the reversal. If `already_depreciated_amount_import` was set wrong at migration, the disposal gain/loss is off by exactly that amount — and it only surfaces at disposal, years later.

**ASG34: _get_residual_value_at_date ignores reversed entries by design** (`account_asset.py → _get_residual_value_at_date`) _ES: El valor residual que muestra el asistente ignora los asientos revertidos y no coincide con el valor contable real._
`_get_residual_value_at_date` (`account_asset/models/account_asset.py:1198`, `AccountAsset._get_depreciation_amount_end_of_lifetime`) filters `not mv.reversed_entry_id` and walks `asset_depreciation_beginning_date` descending (`:1204-1208`). It computes the theoretical residual "as it should be in a normal flow", deliberately ignoring reversals (docstring `:1199-1203`). So the value the modify/sell wizard shows for a given date can differ from the real posted book value when reversals exist on the board. Don't use the wizard's displayed residual as an audit figure.

## New Gotchas

**ASG35: depreciation_value on a closing/disposal move is computed, not the raw line balance** (`account_move.py → _compute_depreciation_value`) _ES: El valor de amortización mostrado en un asiento de baja es un importe derivado, no el importe realmente contabilizado._
`_compute_depreciation_value` (`account_asset/models/account_move.py:69`, `class AccountMove`) normally sums the expense-side lines (`_get_asset_depreciation_line`, `:313`). But for a disposal move of a purchase-type asset (a line equals the full original value AND `len(line_ids) > 2`, `:79-83`), it overrides `depreciation_value` to `original_value - salvage_value - (line_ids[1] debit/credit)` (`:84-90`). So the `depreciation_value` shown on a disposal entry is a derived figure, not the literal posted amount — don't read it as "the depreciation booked in this entry".

**ASG36: Tax is NOT recomputed on depreciation/disposal entries — _get_computed_taxes returns existing tax_ids** (`account_move.py → _get_computed_taxes`) _ES: Los asientos de amortización no recalculan impuestos: al cambiar la posición fiscal siguen con los impuestos originales o vacíos._
`account.move.line._get_computed_taxes` (`account_asset/models/account_move.py:338`, `AccountMoveLine._get_computed_taxes`) returns `self.tax_ids` verbatim whenever `move_id.asset_id` is set, bypassing fiscal-position tax computation. If you change the fiscal position or tax mapping after the asset exists, depreciation entries keep their original (or empty) taxes. Custom logic that relies on `_get_computed_taxes` recomputing will not fire on asset moves.

**ASG37: Reversing a depreciation move silently rolls its value into the next draft, or appends a new period** (`account_move.py → _reverse_moves`) _ES: Revertir una cuota de amortización no la anula: reparte el importe hacia adelante y añade un periodo extra al cuadro._
`_reverse_moves` (`account_asset/models/account_move.py:139`, `AccountMove._reverse_moves`) for an asset move: if a draft move exists it does `first_draft.depreciation_value += move.depreciation_value` (`:150`); else (asset not closing) it creates a brand-new depreciation move dated `last_date + 1 period` (`:158-164`). So reversing a posted depreciation does not just cancel it — it redistributes the amount forward, extending the board by one period if no draft remains. The reversal is also auto-tagged back onto the asset (`:168`).

**ASG38: copy() of an asset always returns to draft and re-defaults the asset account** (`account_asset.py → copy_data`, `create`) _ES: Al duplicar un activo en marcha la copia vuelve siempre a borrador y hay que revalidarla; no se puede clonar activo en marcha._
`copy_data` (`account_asset/models/account_asset.py:501`, `AccountAsset.unlink`) keeps `state='model'` only for models, names the copy `"X (copy)"`, and force-copies `account_asset_id`. `create` (`:510`) then forces `vals['state']='draft'` for anything that isn't an explicit model and raises "Some required values are missing" if a non-draft state is passed with only account/journal keys (`:512-514`). You cannot duplicate a running asset into a running asset — the copy is always draft and must be re-validated.

**ASG39: Changing depreciation/expense account or journal rewrites only future unlocked draft+posted move lines** (`account_asset.py → write`) _ES: Cambiar las cuentas o el diario del activo solo afecta a los asientos futuros no bloqueados; los históricos mantienen las cuentas viejas._
`write` (`account_asset/models/account_asset.py:528`, `AccountAsset.create`) iterates depreciation moves and, only for moves whose `date > user_fiscal_lock_date`, rewrites `account_depreciation_id` onto `line_ids[::2]` and `account_depreciation_expense_id` onto `line_ids[1::2]` (`:535-546`). Moves on/before the lock date keep their old accounts. So switching an asset's accounts gives a board with mixed accounts across the lock boundary — the historical entries are deliberately left untouched.

**ASG40: name and original_value compute are store=True readonly=False — manual edits stick but throw warnings** (`account_asset.py → _compute_value`, `_display_original_value_warning`) _ES: Si se edita el valor original a mano salta un aviso de que no cuadra con el valor de compra, y no deja añadir/quitar facturas una vez activo._
`original_value` is `compute='_compute_value', store=True, readonly=False` (`:118`). When `original_move_line_ids` exist, `_compute_value` (`:209`) overwrites it with `related_purchase_value (+ non_deductible_tax_value)` and raises "All the lines should be posted" if any origin line is draft (`:215-216`). A manual override survives via `create` re-applying `vals['original_value']` (`:519-521`), but the onchange `_display_original_value_warning` (`:390`) flags any mismatch with the purchase value. `_check_related_purchase` (`:459`) hard-blocks adding/removing bills once the asset is not draft/model.

**ASG41: gross_increase_value and analytic distribution are computed sudo / weighted by balance** (`account_asset.py → _compute_gross_increase_value`, `_compute_analytic_distribution`) _ES: El valor de las revalorizaciones y la distribución analítica se calculan saltando permisos de empresa y ponderados por saldo, no copiados tal cual._
`gross_increase_value` is `compute_sudo=True` (`:128`) — it sums children `original_value` regardless of the current user's company access, so the figure can include children the user cannot otherwise read. `_compute_analytic_distribution` (`:240`) recomputes the asset's distribution as a balance-weighted blend of the origin lines' distributions (`distribution * line.balance / amount_total`) — it is NOT a straight copy, and a near-zero total balance leaves the existing distribution untouched (`:245`).
