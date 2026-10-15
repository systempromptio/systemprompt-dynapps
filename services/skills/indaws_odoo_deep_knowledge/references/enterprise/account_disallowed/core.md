<!-- verified: v18.0 @ 2026-06 -->
# account_disallowed_expenses — Non-deductible Expense Tracking (Belgian-style)

Module path: `odoo_enterprise/18/account_disallowed_expenses/` (**EE only**).
`auto_install=True`, `OEEL-1`, depends `['account_reports']` (`account/__manifest__.py:9` y `account/__manifest__.py:18-20`).
**RENAMED in v19 to `account_fiscal_categories`** with breaking model changes — see [[v19_changes]].
For gotchas (ADE1..ADE22 with verified `path:line`) see [[gotchas]].

## Purpose (v18)

Tracks expenses that are partially or fully non-deductible for corporate tax purposes (Belgium, France, etc.). Each disallowed category has a `%` rate that varies over time (e.g. car expenses, restaurant meals). Used in the tax declaration to add back the disallowed portion to taxable income. Manifest name is literally `Disallowed Expenses` (`account_disallowed_expenses/__manifest__.py:4`).

## v18 Models

| Model | File | Purpose |
|-------|------|---------|
| `account.disallowed.expenses.category` | `account_disallowed_expenses/models/account_disallowed_expenses.py:8`, `class AccountDisallowedExpensesCategory` | Category: name, code, active, `rate_ids`, `account_ids`, `current_rate` (computed Char), `company_id` |
| `account.disallowed.expenses.rate` | `account_disallowed_expenses/models/account_disallowed_expenses.py:72`, `AccountDisallowedExpensesRate` | Time-bound rate: `rate` (Float %), `date_from`, `category_id` (ondelete=cascade), `company_id` (required) |
| `account.account` (extended) | `account_disallowed_expenses/models/account_account.py:7`, `AccountAccount` | Adds `disallowed_expenses_category_id` |

Note: in v18 rates live on the **category** (`rate_ids` One2many at `account_disallowed_expenses/models/account_disallowed_expenses.py:15`, `class AccountDisallowedExpensesCategory`). All accounts sharing a category share one rate timeline. This is inverted in v19 (rates per account) — see [[v19_changes]].

## Configuration

```
1. Create category "Car expenses" code='CAR' company=Belgium
2. Add time-bound rates on the category:
   - 2020-01-01 → 25%
   - 2024-01-01 → 30% (legislative change)
3. Assign category to relevant expense accounts (account.account.disallowed_expenses_category_id)
4. Run "Disallowed Expenses Report" for the period
```

## Current Rate Logic (display only)

`_compute_current_rate` (`account_disallowed_expenses/models/account_disallowed_expenses.py:31-35`, `class AccountDisallowedExpensesCategory`) calls `_get_current_rates` (`account_disallowed_expenses/models/account_disallowed_expenses.py:37-47`, `AccountDisallowedExpensesCategory._get_current_rates`) which runs:

```sql
SELECT DISTINCT category_id,
       first_value(rate) OVER (PARTITION BY category_id ORDER BY date_from DESC)
FROM account_disallowed_expenses_rate
WHERE date_from < CURRENT_DATE
AND category_id IN %(ids)s
```

Returns the most recent rate whose `date_from < CURRENT_DATE` (strict — today's-dated rate is NOT considered current). If none → `current_rate = None`, display shows `(No Rate)`. This `current_rate` is **display metadata only**; it is NOT what the report uses to compute disallowed amounts (the report picks the rate per AML date — see [[integrations]] and Reporting below).

## Display Name

`_compute_display_name` (`account_disallowed_expenses/models/account_disallowed_expenses.py:24-29`, `AccountDisallowedExpensesCategory._compute_display_name`): `<code> - <name> (<current_rate or 'No Rate'>)`, e.g. `CAR - Car expenses (25%)`. The rate is formatted `'%g%%'` (`account_disallowed_expenses/models/account_disallowed_expenses.py:35`, `AccountDisallowedExpensesCategory._compute_current_rate`). The fleet bridge overrides this to drop the rate for `car_category` rows (`account_disallowed_expenses_fleet/models/account_disallowed_expenses.py:13-18`, `AccountDisallowedExpensesCategory._compute_display_name`).

## Search

`_search_display_name` (`account_disallowed_expenses/models/account_disallowed_expenses.py:49-59`, `class AccountDisallowedExpensesCategory`): first space-separated token (`value.split(' ')[0]`, line 52) matched against `code` via `=ilike '<token>%'` OR full `value` against `name` via the operator. Negative operators are detected via `expression.NEGATIVE_TERM_OPERATORS` and wrapped in `['!', ...]` (lines 53-57).

## Account Linking

`account.account.disallowed_expenses_category_id` Many2one, `check_company=True` (`account_disallowed_expenses/models/account_account.py:10`, `class AccountAccount`).
`_onchange_internal_group` (`account_disallowed_expenses/models/account_account.py:12-15`, `class AccountAccount`): clears the category if `internal_group` is not in `('income', 'expense')`. Silent — no warning. Switching an account from expense to asset drops the disallowed config.

## Reporting

`account_disallowed_expenses/data/account_disallowed_expenses_report.xml:3`, `disallowed_expenses_report` defines an `account.report` (`Disallowed Expenses Report`) with columns `total_amount`, `rate` (percentage), `disallowed_amount`. Its engine is the custom handler `account.disallowed.expenses.report.handler` (`account_disallowed_expenses/report/account_disallowed_expenses_report.py:8`, `class DisallowedExpensesCustomHandler`, referenced at `account_disallowed_expenses/data/account_disallowed_expenses_report.xml:10`, `disallowed_expenses_report`), built on `account.report.custom.handler` — i.e. fully dependent on the `account_reports` engine.

The handler reads `account.move.line` directly via raw SQL (`account_disallowed_expenses/report/account_disallowed_expenses_report.py:105-181`, `DisallowedExpensesCustomHandler._get_query`):
- joins AML → `account_account` → `account_disallowed_expenses_category`
- `LEFT JOIN account_disallowed_expenses_rate` picking the rate whose `date_from <= aml.date` ordered DESC LIMIT 1 (lines 143-149) — **rate selected per AML date, not the category's current_rate**
- `disallowed_amount = SUM(aml.balance * rate.rate) / 100` (line 131) — confirms rate stored as e.g. `25.0`, divided by 100.
- excludes cancelled moves (`move.state != 'cancel'`, line 157) and, unless "all entries", restricts to `posted` (line 169).

Hierarchy: category → account → rate (when multiple rates in the period; `multi_rate_in_period` set at lines 32-42). A multi-rate warning is raised at lines 44-46.

The per-line rate ultimately shown is resolved in Python by `_get_current_rate` (`account_disallowed_expenses/report/account_disallowed_expenses_report.py:362-363`, `DisallowedExpensesCustomHandler._get_current_rate`): it returns the single `account_rate` of the group, or `None` if the group spans multiple rates. The disallowed amount is the SQL-computed `account_disallowed_amount` (`_get_current_disallowed_amount`, lines 365-366).

### Fleet override (account_disallowed_expenses_fleet)

The fleet handler `_inherit`s the base handler (`account_disallowed_expenses_fleet/report/account_disallowed_expenses_report.py:8-11`, `DisallowedExpensesFleetCustomHandler`) and changes the rate logic: the disallowed amount uses the **minimum** of the category rate and the vehicle's `fleet.disallowed.expenses.rate` — SQL `CASE` at lines 66-78, Python mirror `_get_current_rate` `min(account_rate, fleet_rate)` at lines 280-295. `_get_current_disallowed_amount` returns `fleet_disallowed_amount` when any line has a vehicle (line 297-300). `vehicle_split` defaults True (line 24); `multi_rate_in_period` is OR'd with a per-vehicle multi-rate check (line 34). See [[gotchas]] ADE19-ADE22 and [[integrations]].

## Constraints

`_sql_constraints` `unique_code` = `UNIQUE(code)` (`account_disallowed_expenses/models/account_disallowed_expenses.py:20-22`, `class AccountDisallowedExpensesCategory`) — **globally unique, NOT per-company**. A multi-company DB cannot use code `CAR` for two companies; prefix codes (`BE_CAR`, `FR_CAR`).

## Multi-company

- `category.company_id` Many2one, no `required`, NULL allowed (`account_disallowed_expenses/models/account_disallowed_expenses.py:16`, `class AccountDisallowedExpensesCategory`) → NULL category visible in all companies.
- `rate.company_id` **required**, defaults to `env.company` (`account_disallowed_expenses/models/account_disallowed_expenses.py:80`, `class AccountDisallowedExpensesRate`).
- `account_ids` / `disallowed_expenses_category_id` carry `check_company=True` (`account_disallowed_expenses/models/account_disallowed_expenses.py:17`, `class AccountDisallowedExpensesCategory`, `account_disallowed_expenses/models/account_account.py:10`, `class AccountAccount`).
