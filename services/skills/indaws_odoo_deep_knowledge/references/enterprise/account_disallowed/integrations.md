# account_disallowed_expenses Integrations

EE only. Verified cross-module interactions, Odoo 18.0. Paths relative to `odoo_enterprise/18/` unless noted.

## (a) `account.account` ↔ category, gated by `internal_group`

`account_disallowed_expenses/models/account_account.py:10`, `AccountAccount.disallowed_expenses_category_id` adds `disallowed_expenses_category_id` (Many2one, `check_company=True`). `_onchange_internal_group` (lines 12-15) clears it whenever the account leaves `('income','expense')`. The category exposes the reverse `account_ids` One2many with `check_company=True` (`account_disallowed_expenses_fleet/models/account_disallowed_expenses.py:17`, `AccountDisallowedExpensesCategory._compute_display_name`). So the link is bidirectional and company-consistent, and only meaningful for income/expense accounts.

## (b) `account.move.line` aggregation in the report (the engine reads AML directly)

The report does NOT use stored computed fields on AML — the custom handler issues raw SQL over `account_move_line` (`account_disallowed_expenses/report/account_disallowed_expenses_report.py:105-181`, `DisallowedExpensesFleetCustomHandler._get_query`):
- `FROM account_move_line aml` joined to `account_account` then `account_disallowed_expenses_category` (`category` via `account.disallowed_expenses_category_id = category.id`, lines 141-142).
- Rate resolved **per line date**: `LEFT JOIN account_disallowed_expenses_rate rate` on the subquery picking `date_from <= aml.date` DESC LIMIT 1 (lines 143-149).
- `total_amount = SUM(aml.balance)`, `disallowed_amount = SUM(aml.balance * rate.rate) / 100` (lines 122,131).
- Filters: `aml.company_id IN report companies`, `aml.date` within the period, `move.state != 'cancel'`, optionally `move.state = 'posted'` (lines 153-169).
- Grouping/hierarchy: category → account → rate, with `_dynamic_lines_generator` / expand functions (lines 13-30, 257-279). Drill-down to journal items via `open_journal_items` building a domain on `account_id.disallowed_expenses_category_id` / `account_id` / `vehicle_id` (lines 56-103).

## (c) Hard dependency on `account_reports` (the report engine)

`account_disallowed_expenses/__manifest__.py:9` `depends=['account_reports']`. The report record is an `account.report` (`account_disallowed_expenses/data/account_disallowed_expenses_report.xml:3`, `disallowed_expenses_report`) whose `custom_handler_model_id` points at `account.disallowed.expenses.report.handler` (`account_disallowed_expenses/data/account_disallowed_expenses_report.xml:10`), and that handler `_inherit = 'account.report.custom.handler'` (`account_disallowed_expenses/report/account_disallowed_expenses_report.py:10`, `class DisallowedExpensesCustomHandler`). Without `account_reports` there is no engine, no columns, no caret options. This is why the module is EE-only.

## (d) Fleet integration — `account_disallowed_expenses_fleet` (auto_install bridge)

Module `odoo_enterprise/18/account_disallowed_expenses_fleet/`, `depends=['account_accountant_fleet','account_disallowed_expenses']`, `auto_install=True` (`account/__manifest__.py:8` y `account/__manifest__.py:21`). What it adds:

- **Vehicle-required-on-bills**: category gains `car_category` Boolean (`account_disallowed_expenses_fleet/models/account_disallowed_expenses.py:10`, `AccountDisallowedExpensesCategory.car_category`). AML `_compute_need_vehicle` is True when the line's account category has `car_category` AND `move_type == 'in_invoice'` (`account_disallowed_expenses_fleet/models/account_move.py:10-13`, `class AccountMoveLine`). Bank-rec widget mirrors this via `_compute_vehicle_required` (`account_disallowed_expenses_fleet/models/bank_rec_widget_line.py:9-12`, `class BankRecWidgetLine`). So picking a car-expense account on a vendor bill forces a `vehicle_id`.
- **Display name override**: for `car_category` categories the rate is dropped from `display_name` (`account_disallowed_expenses_fleet/models/account_disallowed_expenses.py:13-18`, `AccountDisallowedExpensesCategory._compute_display_name`).
- **Per-vehicle rates**: new model `fleet.disallowed.expenses.rate` (`account_disallowed_expenses_fleet/models/fleet_vehicle.py:12-20`, `class FleetDisallowedExpensesRate`) with `rate`, `date_from`, `vehicle_id`; `fleet.vehicle` gains `rate_ids` (line 9). This lets disallowance vary by vehicle (e.g. Belgian CO₂-based car deduction), independent of the category rate.
- **Report split by vehicle**: a separate report `data/account_disallowed_expenses_fleet_report.xml`; the handler adds `vehicle_id` grouping. The base handler already anticipates a `vehicle_split` option and a `fleet.vehicle` line model (`account_disallowed_expenses/report/account_disallowed_expenses_report.py:75`, `DisallowedExpensesCustomHandler.open_journal_items` y `account_disallowed_expenses/report/account_disallowed_expenses_report.py:85`, `DisallowedExpensesCustomHandler.open_journal_items` y `account_disallowed_expenses/report/account_disallowed_expenses_report.py:92`, `DisallowedExpensesCustomHandler.open_journal_items`).
- **Deferred reports**: `account.deferred.report.handler` is extended to carry `vehicle_id` through deferral lines (`account_disallowed_expenses_fleet/models/account_deferred_reports.py:8-32`, `DeferredReportCustomHandler`, `account_disallowed_expenses_fleet/models/account_move.py:17-27`, `AccountMoveLine._get_deferred_lines_values`).

v19 equivalents exist: `odoo_enterprise/19/account_fiscal_categories_fleet` and `odoo_enterprise/19/l10n_be_fiscal_categories_fleet`.

## (e) Localization / corporate-tax declaration consumers

- **Belgium**: `odoo_enterprise/18/l10n_be_disallowed_expenses` `depends=['account_disallowed_expenses']` (`account_disallowed_expenses/__manifest__.py:13`) and ships the standard Belgian disallowed-expense categories as demo/data (`l10n_be_disallowed_expenses/data/account_disallowed_expenses.xml:4-40`, `demo_disallowed_expenses_category_1201`, codes `1201`..). There is also `odoo_enterprise/18/l10n_be_account_disallowed_expenses_fleet`. These feed the Belgian corporate-tax "disallowed expenses" reconciliation.
- **v19**: the return-check framework is wired in directly — `account.return.check.template` records (`odoo_enterprise/19/account_fiscal_categories/data/account_return_check_template.xml:5-28`, `account_return_check_template_meal_expense_adjustment`) point their `action_id` at `account_fiscal_categories.action_account_fiscal_report` and pass `selected_variant_id`/`sections_source_id = fiscal_report`, so the fiscal report becomes a tax-return validation check (e.g. meal-expense limit). See [[v19_changes]].
