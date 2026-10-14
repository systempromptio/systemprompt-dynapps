<!-- verified: v19.0 @ 2026-06 (source: odoo_enterprise/19/account_budget + account_budget_purchase) -->
# Account Budget v19 Changes -- Breaking Changes & Migration -- EE

## Major Architectural Change: Purchase Functionality Extracted

The `purchase` dependency and all purchase-related code has been moved to a NEW module: `account_budget_purchase`.

### What Moved Out

| Component | v18 location | v19 location |
|---|---|---|
| `purchase_order.py` model | `account_budget/models/` | `account_budget_purchase/models/` |
| `purchase_order_line.py` model | `account_budget/models/` | `account_budget_purchase/models/` |
| `purchase_views.xml` | `account_budget/views/` | `account_budget_purchase/views/` |
| PO lines query in budget report | `budget_report.py._get_pol_query()` | `account_budget_purchase/reports/` |
| `purchase` dependency | `account_budget.__manifest__` | `account_budget_purchase.__manifest__` |
| Purchase-related demo data | `account_budget/data/` | `account_budget_purchase/demo/` |

`account_budget_purchase` depends on `['account_budget', 'purchase']` and is `auto_install: True`, so it installs automatically when both `account_budget` and `purchase` are present.

### Impact

- **v18 custom code** inheriting `purchase.order` or `purchase.order.line` from `account_budget` must now depend on `account_budget_purchase` instead.
- Budget reports without purchase module installed will no longer show committed PO amounts (cleaner separation).

## Committed Amount / Committed Fields Removed from Core

### budget.line changes

| Change | Detail |
|---|---|
| `committed_amount` field | **REMOVED** from `budget.line` |
| `committed_percentage` field | **REMOVED** from `budget.line` |
| `_compute_all()` | No longer computes `committed:sum`, only `achieved:sum` |
| `line_type` selection on `budget.report` | `'committed'` value **REMOVED** (only `budget` and `achieved` remain) |
| `committed` field on `budget.report` | **REMOVED** |

The committed concept now lives entirely in `account_budget_purchase`. The base budget module only tracks budget vs achieved.

## New: Theoretical Amount in SQL View

v19 adds a `theoretical` field to `budget.report` computed directly in SQL:
```sql
CASE WHEN NOW() < bl.date_to AND NOW() > bl.date_from
     THEN ((NOW()::DATE - bl.date_from::DATE + 1) / (bl.date_to::DATE - bl.date_from::DATE + 1)::FLOAT) * bl.budget_amount
     WHEN NOW() < bl.date_from THEN 0
     ELSE bl.budget_amount
END
```
This is more efficient than the Python-based computation in v18 and fixes the off-by-one on day 1 (returns 0 before `date_from`).

## New: Date Constraint on budget.analytic

v19 adds `@api.constrains('date_from', 'date_to')` directly on `budget.analytic` (`_check_dates`, `odoo_enterprise/19/account_budget/models/budget_analytic.py::_check_dates`). In v18, dates were only validated indirectly via budget lines.

## Revised Budget Naming Changed

| v18 | v19 |
|---|---|
| `_('REV %s', budget.name)` | `f"{base_name} - REV({datetime})"` with `_get_revised_budget_name()` |

v19 uses a regex (`_get_revised_budget_name`, `odoo_enterprise/19/account_budget/models/budget_analytic.py::_get_revised_budget_name`) to strip previous revision suffixes and appends a timestamp: `"Budget 2025 - REV(2025-03-15 14:30)"`. Successive revisions no longer stack prefixes (`REV REV REV...`).

## currency_id Now Computed Instead of Related

| v18 | v19 |
|---|---|
| `related='company_id.currency_id'` | `compute='_compute_currency_id'` with fallback to `env.company.currency_id` |

v19 also adds a `_field_to_sql()` override for `currency_id` to handle SQL-level resolution with COALESCE for NULL company.

## Multi-Currency Aggregation Support

`_read_group_select` and `_read_group_postprocess_aggregate` now support `sum_currency` aggregator for `achieved_amount` and `theoritical_amount`. This converts each record's amount to the company currency before summing, enabling correct multi-currency pivot views.

## New Action: action_open_budget_report

v19 adds `action_open_budget_report()` on `budget.analytic` that opens the budget report pivot view filtered by the current budget. In v18, the only way to see the report was through `action_open_budget_lines()` and then drilling into entries.

## action_open_budget_entries Updated

v19 sets explicit `views` key `[(False, 'list'), (False, 'pivot'), (False, 'graph')]` instead of relying on the action's default views.

## action_open_budget_lines Context Default

v19 adds `default_budget_analytic_id` to context when opened from a single budget, enabling inline creation of lines pre-filled with the budget.

## _inherit String vs List

| v18 | v19 |
|---|---|
| `_inherit = 'analytic.plan.fields.mixin'` | `_inherit = ['analytic.plan.fields.mixin']` |

Minor normalization to list format. No functional change, but affects `super()` MRO if custom modules use string format.

## Security: groups_id -> group_ids

The security XML uses `group_ids` instead of `groups_id` for `res.users` records. This reflects the Odoo 19 field rename on `res.users`.

## Budget Split Wizard

Minor: adds `@api.model` decorator on `default_get()` and uses `Date` import directly. No functional change.

---

## Migration Checklist (v18 -> v19)

1. **If depending on `account_budget` for purchase features**: Add dependency on `account_budget_purchase` or make it conditional.
2. **If reading `committed_amount` / `committed_percentage`**: These fields no longer exist on `budget.line` in core. Check if `account_budget_purchase` restores them or adapt code.
3. **If querying `budget.report` for `committed` field or `line_type='committed'`**: These no longer exist in core. Move to `account_budget_purchase` dependency.
4. **If overriding `_compute_all()`**: Signature changed -- no longer returns `(committed, achieved)` tuple, just `achieved`.
5. **If overriding `create_revised_budget()` or relying on revision naming**: Name format changed from prefix to suffix with timestamp.
6. **If using `currency_id` as related field in SQL**: Now computed with COALESCE fallback. Update any raw SQL references.
7. **If using `groups_id` in XML for res.users**: Rename to `group_ids`.
8. **If extending `budget.report._table_query`**: The UNION no longer includes the POL query. If you need it, depend on `account_budget_purchase`.
