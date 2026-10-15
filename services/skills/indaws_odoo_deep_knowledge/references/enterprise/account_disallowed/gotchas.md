<!-- verified: v18.0 @ 2026-06 -->
# account_disallowed_expenses Gotchas — Odoo 18.0 Enterprise

EE only. Every entry verified against source with `path:line`. Paths relative to `odoo_enterprise/18/`.

---

**ADE1. `unique_code` is GLOBAL, not per-company** _ES: No me deja repetir el mismo código de categoría de gastos no deducibles en otra empresa, dice que ya existe._
`account_disallowed_expenses/models/account_disallowed_expenses.py:20-22`, `AccountDisallowedExpensesCategory._sql_constraints` — `_sql_constraints = [('unique_code', 'UNIQUE(code)', ...)]`, no `company_id` in the constraint. Impact: a multi-company DB cannot reuse code `CAR` across companies. Must prefix (`BE_CAR`, `FR_CAR`). Belgian localization sidesteps this with numeric codes `1201`, `1202`… (`l10n_be_disallowed_expenses/data/account_disallowed_expenses.xml:4-40`, `demo_disallowed_expenses_category_1201`).

**ADE2. `current_rate` ignores today's-dated rate (strict `<`)** _ES: He creado una categoría de gasto no deducible con porcentaje desde hoy y aparece como 'sin tarifa', no coge el % hasta mañana._
`account_disallowed_expenses/models/account_disallowed_expenses.py:43`, `AccountDisallowedExpensesCategory._get_current_rates` — SQL `WHERE date_from < CURRENT_DATE`. A rate with `date_from = today` is NOT current today; a brand-new category whose only rate starts today shows `(No Rate)` until tomorrow. Prepping next year's rate is fine for display; same-day go-live is not.

**ADE3. `_onchange_internal_group` SILENTLY clears the category** _ES: Al cambiar el tipo de cuenta contable se borra sola la categoría de gasto no deducible sin avisar._
`account_disallowed_expenses/models/account_account.py:12-15`, `AccountAccount._onchange_internal_group` — if `internal_group not in ('income','expense')` the category is set to `None`, no warning. Reclassifying an account to asset/liability/equity drops its disallowed config silently. (v19 widens the allowed set to include `'asset'` — see [[v19_changes]].)

**ADE4. v18 → v19 module/model rename breaks references** _ES: Tras actualizar a v19 el módulo de gastos no deducibles se llama distinto y el código personalizado que lo referenciaba deja de funcionar._
v18 `account_disallowed_expenses/__manifest__.py:4` (`Disallowed Expenses`) vs v19 `odoo_enterprise/19/account_fiscal_categories/__manifest__.py:3` (`Account Fiscal Report`). Module name, model names (`account.disallowed.expenses.category` → `account.fiscal.category`) and the `disallowed_expenses_category_id` field all change. Custom/Studio code referencing the old names BREAKS on v19. See [[v19_changes]].

**ADE5. v18 → v19 rates move from category to account** _ES: Después de migrar a v19 los porcentajes de gasto no deducible que estaban en la categoría no aparecen en las cuentas._
v18 rates hang off the category (`account_disallowed_expenses/models/account_disallowed_expenses.py:15` `rate_ids` One2many on category; rate FK `category_id` at line 79). v19 rates hang off the account (`odoo_enterprise/19/account_fiscal_categories/models/account_account.py::rate_ids,36` — `rate_ids` on account, `account.account.fiscal.rate.related_account_id`). Migration must DUPLICATE each category rate onto every linked account. Verify post-upgrade.

**ADE6. v19 adds `'asset'` to the allowed internal groups** _ES: En v19 puedo asignar categoría fiscal también a cuentas de activo, cosa que antes no dejaba._
v18 `account_disallowed_expenses/models/account_account.py:14`, `AccountAccount._onchange_internal_group` checks `('income','expense')`; v19 `odoo_enterprise/19/account_fiscal_categories/models/account_account.py::_onchange_internal_group` checks `('asset','expense','income')`. Pre-existing asset accounts won't auto-receive a category, but become eligible.

**ADE7. Display name renders `(No Rate)`, not blank** _ES: El nombre de la categoría muestra el texto '(No Rate)' o '(Sin tarifa)' y me descuadra las búsquedas por nombre._
`account_disallowed_expenses/models/account_disallowed_expenses.py:27-28`, `AccountDisallowedExpensesCategory._compute_display_name` — `rate = record.current_rate or _('No Rate')`, name `f'{code} - {name} ({rate})'`. The literal `(No Rate)` (translatable) appears in the UI; search/filters by display name must account for it.

**ADE8. `disallowed_expenses_category_id` has `check_company=True`** _ES: No me deja asignar una categoría de gasto no deducible de otra empresa, salta un error de coherencia de compañía._
`account_disallowed_expenses/models/account_account.py:10`, `AccountAccount.disallowed_expenses_category_id`. Cross-company assignment is blocked at the ORM. Setting the category from a wrong-company context (even via `[(6,0,...)]`) raises a company-consistency error.

**ADE9. Rate stored as a whole-number Float (e.g. `25.0`), divided by 100 in the report** _ES: El importe no deducible del informe me sale multiplicado por cien porque el porcentaje se guarda como número entero._
`account_disallowed_expenses/models/account_disallowed_expenses.py:77`, `AccountDisallowedExpensesRate.rate` (`rate = fields.Float('Disallowed %')`) and the report formula `SUM(aml.balance * rate.rate) / 100` (`account_disallowed_expenses/report/account_disallowed_expenses_report.py:131`, `DisallowedExpensesCustomHandler._get_query`). Custom code treating the value as a 0.25 fraction inflates the disallowed amount 100×.

**ADE10. `active=False` hides but does NOT unlink** _ES: He archivado la categoría de gasto no deducible pero las cuentas siguen enlazadas y al reactivarla vuelve todo._
`account_disallowed_expenses/models/account_disallowed_expenses.py:14`, `AccountDisallowedExpensesCategory.active` — standard `active` flag. Linked accounts keep `disallowed_expenses_category_id` pointing at the archived category; re-activating restores every linkage instantly.

**ADE11. `auto_install=True` — installs with `account_reports`, no easy opt-out** _ES: Se me ha instalado solo el informe/menú de gastos no deducibles al activar contabilidad y no lo necesito._
v18 `account_disallowed_expenses/__manifest__.py:19` and v19 `odoo_enterprise/19/account_fiscal_categories/__manifest__.py:18`. The fleet bridge is also `auto_install=True` (`account_disallowed_expenses_fleet/__manifest__.py:21`, installs when `account_accountant_fleet` + base are present). Customers get the report/menu whether they need it or not.

**ADE12. v19 report label is "Fiscal Report" / "Account Fiscal Report"** _ES: En v19 el informe se llama 'Informe Fiscal' y la columna cambió de nombre, la documentación y capturas de v18 ya no coinciden._
v18 report name `Disallowed Expenses Report` (`account_disallowed_expenses/data/account_disallowed_expenses_report.xml:4`, `disallowed_expenses_report`) vs v19 `Fiscal Report` (`odoo_enterprise/19/account_fiscal_categories/data/account_fiscal_report.xml:4`, `fiscal_report`); the v19 disallowed column is renamed `Fiscal Amount` / `deductible_amount` (lines 22-23). v18 training docs/screenshots mislead v19 users.

**ADE13. v19 replaces `_sql_constraints` with `models.Constraint`** _ES: Al heredar la categoría en v19 la restricción de código único está declarada distinto y mi override deja de aplicarse._
v18 `account_disallowed_expenses/models/account_disallowed_expenses.py:20-22`, `AccountDisallowedExpensesCategory._sql_constraints` (list syntax) vs v19 `odoo_enterprise/19/account_fiscal_categories/models/account_fiscal_category.py::_unique_code` (`_unique_code = models.Constraint('UNIQUE(code)', ...)`). Custom inheritance overriding the constraint must update the declaration.

**ADE14. `_search_display_name` splits on the first space** _ES: Si el código de la categoría tiene un espacio no lo encuentro al buscar; solo busca hasta el primer espacio._
`account_disallowed_expenses/models/account_disallowed_expenses.py:52`, `AccountDisallowedExpensesCategory._search_display_name` — `code_value = value.split(' ')[0]`. Typing `CAR expenses` matches code `CAR%`. Codes that themselves contain spaces are unsearchable by code. (v19 keeps the same split at `odoo_enterprise/19/account_fiscal_categories/models/account_fiscal_category.py::code_value`.)

**ADE15. v19 category field on `account.account` is indexed (`btree_not_null`); v18 was not** _ES: Al actualizar a v19 con muchas cuentas contables el proceso se queda bloqueado creando un índice en la tabla._
The field is **renamed** across versions: v18 `disallowed_expenses_category_id` (`odoo_enterprise/18/account_disallowed_expenses/models/account_account.py::disallowed_expenses_category_id`) has **no** index; v19 renames it to `fiscal_category_id` in the renamed module `account_fiscal_categories` (`odoo_enterprise/19/account_fiscal_categories/models/account_account.py::fiscal_category_id`) and adds `index='btree_not_null'`. Upgrading a large `account_account` table triggers index creation (table lock) — schedule a maintenance window.

---

**ADE16. Report rate is per-AML-date, NOT the category's `current_rate`** _ES: El porcentaje que veo en el nombre de la categoría no coincide con el que aplica el informe a líneas antiguas._
`account_disallowed_expenses/report/account_disallowed_expenses_report.py:143-149`, `DisallowedExpensesCustomHandler._get_query` — the report's `LEFT JOIN` selects the rate whose `date_from <= aml.date` (DESC LIMIT 1). The `current_rate` Char field (lines 31-47, gated by `date_from < CURRENT_DATE`) is **display only** and may differ from what the report applies to historical lines. Do not assume the badge in the category name equals the rate used in figures.

**ADE17. Cancelled moves excluded, drafts conditionally included** _ES: El informe de gastos no deducibles suma también los apuntes en borrador cuando activo 'todas las entradas'._
`account_disallowed_expenses/report/account_disallowed_expenses_report.py:157`, `DisallowedExpensesCustomHandler._get_query` y `account_disallowed_expenses/report/account_disallowed_expenses_report.py:169`, `DisallowedExpensesCustomHandler._get_query` — `move.state != 'cancel'` always; `move.state = 'posted'` only when NOT "all entries". With the "all entries" filter on, draft moves are counted in disallowed totals. Tax-declaration figures must run with all-entries OFF.

**ADE18. Fleet bridge makes a vehicle mandatory on vendor bills for `car_category` accounts** _ES: Desde que se instaló el módulo de flota, en las facturas de proveedor con cuentas de coche me obliga a poner un vehículo._
`account_disallowed_expenses_fleet/models/account_move.py:10-13`, `AccountMoveLine._compute_need_vehicle` — `_compute_need_vehicle` is True when the account's category has `car_category=True` AND `move_type == 'in_invoice'`. The `car_category` read is done with `.sudo()` (line 13), so the requirement fires even if the current user has no read access to the category (record rules bypassed). Installing `account_disallowed_expenses_fleet` (auto_install) silently turns vehicle into a required field on bill lines using car-expense accounts. See [[integrations]].

**ADE19. Fleet applies the MINIMUM of category-rate and vehicle-rate, not the vehicle's** _ES: He puesto una tarifa por vehículo más alta que la de la categoría y no cambia nada; solo baja si la pongo más baja._
`account_disallowed_expenses_fleet/report/account_disallowed_expenses_report.py:66-78`, `DisallowedExpensesFleetCustomHandler._get_query` (SQL `CASE`) and the Python mirror `_get_current_rate` (lines 280-295, `min(account_rate, fleet_rate)`). When a line's account has a category rate AND its vehicle has a `fleet.disallowed.expenses.rate`, the report uses `LEAST(category_rate, vehicle_rate)`. If only one exists, that one is used; if a `fleet_rate` row exists it always wins consideration even when smaller. Setting a per-vehicle rate HIGHER than the category does nothing; setting it LOWER reduces the disallowed amount. Counter-intuitive for anyone expecting the vehicle rate to override.

**ADE20. `vehicle_split` defaults to True — the fleet report splits by vehicle out of the box** _ES: El informe de flota me agrupa las líneas por vehículo de serie y sale con distinta forma que el informe normal._
`account_disallowed_expenses_fleet/report/account_disallowed_expenses_report.py:24`, `DisallowedExpensesFleetCustomHandler._custom_options_initializer` — `options['vehicle_split'] = previous_options.get('vehicle_split', True)`. First open of the fleet report groups lines per vehicle (category → vehicle → account → rate), not the flat category → account hierarchy of the base report. Users comparing the two reports see a different shape even with identical data.

**ADE21. Fleet `multi_rate_in_period` ORs the vehicle-rate check onto the category check** _ES: Me salta el aviso de varias tarifas en el periodo solo porque cambió la tarifa de un vehículo, no la de la categoría._
`account_disallowed_expenses_fleet/report/account_disallowed_expenses_report.py:34`, `DisallowedExpensesFleetCustomHandler._custom_options_initializer` — `options['multi_rate_in_period'] = options.get('multi_rate_in_period') or bool(rg)` where `rg` counts vehicles with >1 `fleet.disallowed.expenses.rate` in the period. So the multi-rate warning/expansion can trigger from a vehicle rate change even when no category rate changed in the period.

**ADE22. Fleet warns about expense lines that have a vehicle but whose account has NO disallowed category** _ES: Hay líneas con vehículo y cuenta sin categoría que no aparecen en el informe, y el total de no deducible sale corto._
`account_disallowed_expenses_fleet/report/account_disallowed_expenses_report.py:36-52`, `DisallowedExpensesFleetCustomHandler._customize_warnings` — `_customize_warnings` `_read_group`s posted expense AML with `vehicle_id != False` and `account_id.disallowed_expenses_category_id = None`, raising `warning_missing_disallowed_category`. Such lines are SILENTLY ABSENT from the figures (the base query INNER-joins the category, line 142), so the disallowed total under-reports until those accounts are categorized. `action_open_accounts` (lines 305-312) opens the offending accounts.
