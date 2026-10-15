# account_disallowed_expenses → account_fiscal_categories (v19)

EE only. In v19 the module is **renamed and re-architected**. New path: `odoo_enterprise/19/account_fiscal_categories/` (verified to exist). All `path:line` below are v19 unless noted.

## Module rename

| | v18 | v19 |
|---|-----|-----|
| Module | `account_disallowed_expenses` | `account_fiscal_categories` |
| Manifest name | `Disallowed Expenses` (`v18 account_disallowed_expenses/__manifest__.py:4`) | `Account Fiscal Report` (`account_disallowed_expenses/__manifest__.py:3`) |
| `depends` | `['account_reports']` | `['account_reports']` (`account_disallowed_expenses/__manifest__.py:8`) |
| `auto_install` | `True` | `True` (`account_disallowed_expenses/__manifest__.py:18`) |
| author | (none) | `Odoo S.A.` (`account_disallowed_expenses/__manifest__.py:20`) |

## Model / field mapping (verified)

| v18 | v19 | v19 ref |
|-----|-----|---------|
| `account.disallowed.expenses.category` | `account.fiscal.category` | `account_fiscal_categories/models/account_fiscal_category.py:8`, `AccountFiscalCategory`, v19 |
| `account.disallowed.expenses.rate` (FK `category_id`) | `account.account.fiscal.rate` (FK `related_account_id`) | `account_fiscal_categories/models/account_account.py:29-37`, `AccountAccountFiscalRate` |
| `account.account.disallowed_expenses_category_id` | `account.account.fiscal_category_id` | `account_disallowed_expenses/models/account_account.py:9`, `class AccountAccount` |
| `category.rate_ids` (One2many on category) | **removed** — rates now `account.rate_ids` | `account_disallowed_expenses/models/account_account.py:10`, `class AccountAccount` |
| `category.current_rate` (computed **Char**) | **removed** — `account.current_rate` (computed **Float**) | `account_disallowed_expenses/models/account_account.py:11`, `class AccountAccount` y `account_disallowed_expenses/models/account_account.py:13-21`, `AccountAccount._onchange_internal_group` |
| `_sql_constraints` list | `models.Constraint` (`_unique_code`) | `account_fiscal_categories/models/account_fiscal_category.py:17-20`, `AccountFiscalCategory._unique_code`, v19` |

## Architectural shift — rates per CATEGORY → rates per ACCOUNT

v18: rate timeline belongs to the category (`v18 account_disallowed_expenses/models/account_disallowed_expenses.py:15`, `AccountDisallowedExpensesCategory.rate_ids` y `v18 account_disallowed_expenses/models/account_disallowed_expenses.py:79`, `AccountDisallowedExpensesRate.category_id`); every account sharing a category shares one rate.

v19: rate timeline belongs to the account:
- `account.rate_ids = One2many('account.account.fiscal.rate', 'related_account_id')` (`account_disallowed_expenses/models/account_account.py:10`, `class AccountAccount`)
- `account.account.fiscal.rate.related_account_id` (required, `ondelete='cascade'`) (`account_fiscal_categories/models/account_account.py:36`, `AccountAccountFiscalRate.related_account_id`)
- `account.current_rate` is computed in **Python** (not SQL window function): iterate `rate_ids` ordered `date_from desc` (`_order` line 32), take the first with `date_from <= context_today` (`account_disallowed_expenses/models/account_account.py:13-21`, `AccountAccount._onchange_internal_group`). Note v19 uses `<=` (today's rate IS current), vs v18's strict `<` (see [[gotchas]] ADE2).

Migration impact: a v18 category rate must be DUPLICATED onto each linked account in v19 ([[gotchas]] ADE5). Granular but more setup.

## `account.fiscal.category` (v19)

`account_fiscal_categories/models/account_fiscal_category.py:8-39`, `AccountFiscalCategory`, v19:
- `name` (translate), `code` (required), `active`, `company_id`, `account_ids` One2many (`check_company=True`, line 15).
- **No** `rate_ids`, **no** `current_rate`. Pure metadata.
- `_unique_code = models.Constraint('UNIQUE(code)', "Fiscal category code should be unique.")` (lines 17-20) — still global, not per-company.
- `_compute_display_name`: `f"{code} - {name}"` if code else `name` (lines 22-25) — **no rate** in the name anymore (v18 always appended `(rate)`).
- `_search_display_name` (lines 27-39): same first-token-on-space split (line 34), but now returns `Domain(...)` objects / `NotImplemented` for negative operators (`Domain.NEGATIVE_OPERATORS`, line 29) — modern domain API.

## `account.account` (v19)

`account_disallowed_expenses/models/account_account.py:6-26`, `AccountAccount._inherit`:
- `fiscal_category_id` Many2one `check_company=True`, `index='btree_not_null'` (line 9) — indexed (v18 was not; [[gotchas]] ADE15).
- `_onchange_internal_group` clears `fiscal_category_id` unless `internal_group in ('asset','expense','income')` (lines 23-26) — now includes `'asset'` ([[gotchas]] ADE6).

## Report (v19)

`account_fiscal_categories/data/account_fiscal_report.xml:3-26`, `fiscal_report`, v19:
- `account.report` named `Fiscal Report` (line 4); handler `account.fiscal.report.handler` (`account_fiscal_categories/report/account_fiscal_report_handler.py:8`, `AccountFiscalReportHandler`, v19, ref line 10).
- Columns: `total_amount` → label **Base Total** (line 13), `rate` (percentage), `deductible_amount` → label **Fiscal Amount** (lines 21-23). The v18 `disallowed_amount` column is gone — v19 frames it as deductible/fiscal amount.
- SQL rate join now goes through the account, not the category: `JOIN account_fiscal_category category ON account.fiscal_category_id = category.id` then `LEFT JOIN account_account_fiscal_rate rate` on subquery `r2.date_from <= aml.date` matched by `related_account_id = account.id` (`account_fiscal_categories/report/account_fiscal_report_handler.py:160-167`, `AccountFiscalReportHandler._get_query`, v19`). Confirms the per-account rate model end to end.

## New: return-check templates

`data/account_return_check_template.xml` (in manifest at `account_disallowed_expenses/__manifest__.py:16`) creates `account.return.check.template` records that hook the fiscal report into the tax-return validation flow — `action_id = action_account_fiscal_report`, passing `selected_variant_id`/`sections_source_id = fiscal_report` (lines 5-28). v18 had no such templates. See [[integrations]] (e).

## v19 fleet bridges

`odoo_enterprise/19/account_fiscal_categories_fleet` and `odoo_enterprise/19/l10n_be_fiscal_categories_fleet` replace the v18 `account_disallowed_expenses_fleet` / `l10n_be_account_disallowed_expenses_fleet`.
