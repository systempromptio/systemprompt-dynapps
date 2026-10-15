<!-- verified: v18.0 @ 2026-06 -->
# account_3way_match v19 Changes

The module EXISTS in v19: `odoo_enterprise/19/account_3way_match/`. Diffs below are the full output of `diff -r 18/account_3way_match 19/account_3way_match`, verified file-by-file. The previous core.md claimed only "two minor changes" — that was incomplete: the dashboard model was substantially refactored.

## 1. `models/account_invoice.py` — two logic changes

**`line_subsection` excluded in aggregation** (`account_3way_match/account_invoice.py:62`, `AccountMove._compute_release_to_pay`):
```diff
- for invoice_line in invoice.invoice_line_ids.filtered(lambda l: l.display_type not in ('line_section', 'line_note')):
+ for invoice_line in invoice.invoice_line_ids.filtered(lambda l: l.display_type not in ('line_section', 'line_subsection', 'line_note')):
```
v19 introduces nested subsection invoice lines; they are now skipped in `_compute_release_to_pay`. Impact: subsection lines never affect `release_to_pay`. (v18 ref: `account_3way_match/account_invoice.py:61`, `AccountMove._compute_release_to_pay`.)

**Decimal-precision key renamed** (`account_3way_match/account_invoice.py:113`, `AccountMoveLine._can_be_paid`):
```diff
- precision = self.env['decimal.precision'].precision_get('Product Unit of Measure')
+ precision = self.env['decimal.precision'].precision_get('Product Unit')
```
Impact: custom code calling `precision_get('Product Unit of Measure')` in v19 no longer resolves the same precision record, silently changing rounding in all four `float_compare` calls. MUST update. (v18 ref: `account_3way_match/account_invoice.py:112`, `AccountMoveLine._can_be_paid`.)

(One cosmetic diff: a blank line added at `account_3way_match/account_invoice.py:11`, `AccountMove`.)

## 2. `models/account_journal_dashboard.py` — refactored (NOT minor)

**Imports**: `from odoo.osv import expression` → `from odoo.fields import Domain` (`account_3way_match/account_journal_dashboard.py:3`). `# -*- coding: utf-8 -*-` header dropped.

**`_get_open_sale_purchase_query` removed; replaced by `_get_to_pay_select`** (`odoo_enterprise/18/account_3way_match/models/account_journal_dashboard.py:20-21`, `AccountJournal._get_open_sale_purchase_query`):
```python
def _get_to_pay_select(self):
    # OVERRIDE
    return SQL("release_to_pay IN ('yes', 'exception') AS to_pay")
```
v18 overrode the whole query builder (`account_3way_match/account_journal_dashboard.py:20-42`, `AccountJournal`) and re-declared the full `_where_calc` + `selects` list. v19's base `account` refactor splits the select out, so the module now overrides only the tiny select hook. Impact: any custom module that itself overrode `_get_open_sale_purchase_query` to coexist with 3-way-match breaks on v19 — must move to `_get_to_pay_select`.

**`_get_draft_sales_purchases_query` rewritten to `Domain` API** (`account_3way_match/account_journal_dashboard.py:37-43`, `AccountJournal`):
```diff
- domain = expression.AND([
-     [('state', '=', 'draft'), ('payment_state', 'in', ('not_paid', 'partial'))],
-     expression.OR([domain_sale, domain_purchase])
- ])
- return self.env['account.move']._where_calc([
-     *self.env['account.move']._check_company_domain(self.env.companies),
-     *domain
- ])
+ domain = Domain.AND([
+     self.env['account.move']._check_company_domain(self.env.companies),
+     Domain('state', '=', 'draft'),
+     Domain('payment_state', 'in', ('not_paid', 'partial')),
+     Domain.OR([domain_sale, domain_purchase]),
+ ])
+ return self.env['account.move']._search(domain, bypass_access=True)
```
Returns a `_search` (with `bypass_access=True`) instead of a `_where_calc` query object. The functional gate is unchanged (`invoice_date_due < today OR release_to_pay = 'yes'`, `account_3way_match/account_journal_dashboard.py:33-35`, `AccountJournal._get_open_sale_purchase_query`).

## 3. `views/account_invoice_view.xml` — date-filter syntax

`bills_to_validate` filter domain (`account_3way_match/views/account_invoice_view.xml:28`, `account_invoice_filter_inherit_account_3way_match`):
```diff
- ('invoice_date_due', '&lt;', time.strftime('%Y-%m-%d'))
+ ('invoice_date_due', '&lt;', 'today')
```
v19 uses the `'today'` dynamic-date literal instead of the legacy `time.strftime(...)`. Functionally equivalent. (Also minor whitespace inside the domain tuple.) No other view diffs; `account_journal_dashboard_view.xml` is identical between versions.

## 4. `__manifest__.py`

```diff
- 'category': 'Inventory/Purchase',
+ 'category': 'Supply Chain/Purchase',
  ...
+ 'author': 'Odoo S.A.',
```
(`account_3way_match/__manifest__.py:6` and added `author` after the `data` block.) `depends`, `license: 'OEEL-1'`, and no-`auto_install` are unchanged.

## Summary of impact
- Custom code reading `'Product Unit of Measure'` precision → BREAKS silently in v19.
- Custom dashboard overrides of `_get_open_sale_purchase_query` → BREAK (method gone; use `_get_to_pay_select`).
- Subsection lines now ignored in 3-way matching.
- Everything else (concept, per-line/aggregation logic, manual override, field set) is unchanged.
