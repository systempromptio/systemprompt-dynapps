# Analytic v19 Changes — Breaking & Notable

## 1. analytic.mixin — Search Method Rewrite

### _condition_to_sql() → _search_analytic_distribution() [BREAKING]
```python
# Odoo 18 — override _condition_to_sql for custom search
def _condition_to_sql(self, alias, fname, operator, value, query):
    if fname == 'analytic_distribution': ...

# Odoo 19 — new dedicated search method + Domain API
def _search_analytic_distribution(self, operator, value):
    # Returns Domain objects instead of SQL
    return Domain.custom(to_sql=...)
```
**Impact:** Any custom module overriding `_condition_to_sql()` for analytic_distribution must migrate to `_search_analytic_distribution()`. Uses new `Domain` API (`from odoo.fields import Domain`).

### _read_group_groupby() — New `alias` parameter [BREAKING]
```python
# Odoo 18
def _read_group_groupby(self, groupby_spec, query):

# Odoo 19
def _read_group_groupby(self, alias, groupby_spec, query):
```
**Impact:** All custom overrides of `_read_group_groupby()` on analytic models must add `alias` as first parameter.

### Domain API migration
- `osv.expression.OR` → `Domain.OR` or `Domain([...] | [...])`
- Negative operators: Odoo 19 checks `Domain.NEGATIVE_OPERATORS` explicitly
- `_search_distribution_analytic_account_ids()` now supports `any`, `not any`, `any!`, `not any!` operators

### init() table existence check
- Odoo 18: direct SQL query
- Odoo 19: `table_exists(self.env.cr, self._table)` utility function

### filtered_domain() refactored
- Odoo 19 uses `Domain.map_conditions()` API instead of context-based workaround

## 2. analytic.plan.fields.mixin — Class Rename

```python
# Odoo 18
class AnalyticPlanFields(models.AbstractModel):

# Odoo 19
class AnalyticPlanFieldsMixin(models.AbstractModel):
```
**Impact:** Code referencing `AnalyticPlanFields` by class name (rare, but possible in tests) needs update.

### product_uom_id field on account.analytic.line
- Odoo 18: `domain="[('category_id', '=', product_uom_category_id)]"`, string='Unit of Measure'
- Odoo 19: domain removed entirely, `product_uom_category_id` related field removed, string='Unit'
**Impact:** Modules relying on UoM category filtering on analytic lines lose client-side validation.

### _inherit syntax
- Odoo 18: `_inherit = 'analytic.plan.fields.mixin'` (string)
- Odoo 19: `_inherit = ['analytic.plan.fields.mixin']` (list)
**Impact:** Cosmetic, no functional change.

## 3. account.analytic.account

### partner_id field
- Odoo 18: `auto_join=True`
- Odoo 19: `bypass_search_access=True`, added `index='btree_not_null'`

### plan_id field
- Odoo 19: added `index=True`

### New aggregation: sum_currency
```python
# Odoo 19 — new currency-aware aggregation
_read_group_select() supports: 'balance:sum_currency', 'debit:sum_currency', 'credit:sum_currency'
_read_group_postprocess_aggregate() converts each record to company currency independently
```
**Impact:** Custom reporting using `_read_group()` on analytic accounts can now request currency-aware sums.

### Null safety in balance computation
- Odoo 19: `_compute_debit_credit_balance()` checks `if not plan:` → sets 0
- `self._context` → `self.env.context` (consistent pattern)

## 4. account.analytic.plan

### parent_id, applicability plan_id
- Odoo 19: added `index='btree_not_null'` for performance

### Cache clearing
- Odoo 18: `self.env.registry.clear_cache()`
- Odoo 19: `self.env.registry.clear_cache('stable')` — targets stable cache only

### Empty recordset safety
- `_compute_all_analytic_account_count()`: Odoo 19 adds early return if `self.ids` empty

## 5. account.move.line

### Field changes affecting analytic
| Field | Odoo 18 | Odoo 19 |
|-------|---------|---------|
| `move_id` | `auto_join=True` | `bypass_search_access=True` |
| `account_id` | `auto_join=True` | `bypass_search_access=True`, removed `('deprecated', '=', False)` from domain |
| `payment_id` | `auto_join=True` | `bypass_search_access=True` |
| `tax_tag_invert` | Boolean field | **REMOVED** |
| `extra_tax_data` | N/A | **NEW** Json field for tax computation engine |

### auto_join deprecation [BREAKING PATTERN]
All `auto_join=True` replaced with `bypass_search_access=True` across Odoo 19. Not analytic-specific but affects all analytic search queries that join through these fields.

## 6. Enterprise: account_reports

### Analytic groupby
- `_create_column_analytic()`: Odoo 19 adds check `if has_selected_budgets and not options['selected_horizontal_group_id']:`
- Empty header changed from `{'name': ''}` to `{'name': _("Total")}` (i18n improvement)

## 7. Enterprise: account_budget

### No structural changes found
Budget models (budget.analytic, budget.line, budget.report) appear unchanged between v18 and v19 in the files examined.

## Migration Checklist

| Priority | Action | Impact |
|----------|--------|--------|
| **HIGH** | Replace `_condition_to_sql()` overrides with `_search_analytic_distribution()` | Search breaks |
| **HIGH** | Add `alias` param to `_read_group_groupby()` overrides | GroupBy breaks |
| **HIGH** | Replace `auto_join=True` with `bypass_search_access=True` on relational fields | Access control |
| **MEDIUM** | Migrate `osv.expression.OR` to `Domain` API | Search expressions |
| **MEDIUM** | Handle new `sum_currency` aggregates in custom reports | Reporting |
| **LOW** | Update `self._context` to `self.env.context` | Code consistency |
| **LOW** | Remove UoM domain from analytic line customizations | Client validation |
| **LOW** | Update `AnalyticPlanFields` class references | Tests |
