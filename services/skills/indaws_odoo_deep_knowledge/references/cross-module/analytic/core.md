<!-- verified: v18.0 @ 2026-06 -->
# Analytic Accounting Core — Odoo 18.0

## 1. Architecture Overview

```
account.analytic.plan (dimension: Project, Department, Cost Center)
  ├── default_applicability: optional | mandatory | unavailable
  ├── applicability_ids → account.analytic.applicability (per business_domain rules)
  ├── account_ids → [account.analytic.account, ...]
  └── Dynamic columns: account_id (project plan) or x_plan{id}_id (other plans)

account.analytic.account (value: "Project Alpha", "Sales Dept")
  ├── plan_id → plan, root_plan_id (denormalized)
  ├── partner_id, company_id, code, active
  ├── balance/debit/credit (computed from analytic lines)
  └── line_ids → account.analytic.line (via auto_account_id)

account.analytic.line (actual cost/revenue entry)
  ├── Inherits: analytic.plan.fields.mixin
  ├── date, amount, unit_amount, name
  ├── move_line_id → account.move.line (cascade delete)
  ├── general_account_id (computed from move_line_id, account module)
  ├── category: 'other' (base analytic) + 'invoice'/'vendor_bill' (account) + 'manufacturing_order' (mrp_account)
  └── {plan_column}: account_id per plan (dynamic)
       Note: NO move_id field — link to journal entry is via move_line_id (ondelete=cascade) only

account.analytic.distribution.model (auto-fill rules)
  ├── Inherits: analytic.mixin
  ├── Matching: partner_id, partner_category_id, company_id
  ├── Account module adds: product_id, product_categ_id, account_prefix
  └── Scoring system: higher specificity wins (NOT sequence)
```

## 2. Mixins

### analytic.mixin — JSON Distribution Field
Inherited by: `account.move.line`, `sale.order.line`, `purchase.order.line`, `hr.expense`, `account.asset`, `account.analytic.distribution.model`

| Field | Type | Properties |
|-------|------|-----------|
| `analytic_distribution` | Json | compute, store, copy=True, readonly=False |
| `analytic_precision` | Integer | store=False, from decimal.precision "Percentage Analytic" |
| `distribution_analytic_account_ids` | M2M (computed) | For searching — use this, NOT the JSON field |

**JSON format:**
```json
{"account_id_1,account_id_2": 50.0, "account_id_3": 50.0}
```
- Keys: comma-separated account IDs (one per plan in multi-plan scenarios)
- Values: percentages (must sum to 100% per mandatory plan)
- Special key `__update__`: signals partial plan update in `_merge_distribution()`

**Key methods:**
- `_validate_distribution(**kwargs)` — checks mandatory plans sum to 100%. Only runs with `context.validate_analytic=True`
- `_merge_distribution(old, new)` — if `__update__` key present, preserves non-changing plans proportionally (cartesian product)
- `_sanitize_values(vals, precision)` — rounds all percentages. Called on create/write
- `_query_analytic_accounts(table)` — SQL: `regexp_split_to_array(jsonb_path_query_array(analytic_distribution, '$.keyvalue()."key"')::text, '\D+')`
- `_condition_to_sql()` — custom SQL for search operators (=, !=, ilike, in). Converts account display_name → IDs automatically
- `_read_group_groupby()` — explodes JSON keys into separate rows with `regexp_matches(..., '\d+', 'g')`. Only supports `__count` aggregation

### analytic.plan.fields.mixin — Dynamic Plan Columns
Inherited by: `account.analytic.line`, `budget.line`, `budget.report`

| Field | Type | Properties |
|-------|------|-----------|
| `account_id` | M2O (analytic.account) | Project plan column, ondelete='restrict', index=True |
| `auto_account_id` | M2O (computed) | Context-dependent magic field via `analytic_plan_id` context |
| `x_plan{id}_id` | M2O (dynamic) | Created per root plan via ir.model.fields |
| `x_plan{id}_id_{depth}` | M2O (related) | Hierarchy grouping for sub-plans |

**Key methods:**
- `_get_plan_fnames()` — returns all plan column names on this model
- `_get_analytic_accounts()` — returns account recordset from all plan fields
- `_get_distribution_key()` — returns comma-separated IDs for JSON key
- `_get_mandatory_plans(company, business_domain)` — returns mandatory plan list
- `_patch_view(arch, view, view_type)` — dynamically adds plan field nodes after `account_id` in views
- `_check_account_id()` — constraint: at least one account must be set
- `fields_get()` — dynamically updates field strings/domains per plan name

## 3. GIN Index on analytic_distribution

Created in `analytic.mixin.init()` for every model with stored `analytic_distribution`:
```sql
CREATE INDEX IF NOT EXISTS {table}_analytic_distribution_accounts_gin_index
ON {table} USING gin(
  regexp_split_to_array(
    jsonb_path_query_array(analytic_distribution, '$.keyvalue()."key"')::text,
    '\D+'
  )
)
```
Extracts account IDs from JSON keys → inverted index. Search uses `&&` (array intersection) operator.

## 4. Dynamic Column System

### How plans create database columns
1. **Root plan** → stored Many2one: `account_id` (project plan) or `x_plan{id}_id` (others)
2. **Child plan** → related (computed) Many2one: `x_plan{id}_id_{depth}`
3. Created via `ir.model.fields` with BTREE index + `IS NOT NULL` filter
4. `_sync_plan_column()` called on plan name change, parent change, or deletion
5. Plan rename → raw SQL UPDATE to migrate analytic lines to new column

### Column name resolution
```python
plan._strict_column_name()  # → 'account_id' if project plan, else f'x_plan{id}_id'
plan._column_name()         # → delegates to root_id._strict_column_name()
plan._hierarchy_name()      # → (depth, f'{column}_{depth}') for sub-plan grouping
```

## 5. Applicability System

### Scoring (highest wins) — `_get_score()` `analytic/models/analytic_plan.py:417-426`, `AccountAnalyticApplicability._get_score`, account override `account/models/account_analytic_plan.py:59-76`
| Criterion | Score |
|-----------|-------|
| Company set on rule AND `company_id` in kwargs | +0.5 |
| Business domain matches | +1 |
| Business domain doesn't match (and kwargs has one) | -1 (disqualify) |
| `business_domain` absent from kwargs | returns just the company score (no domain term) |
| Account prefix matches (account module) | +1 |
| Account prefix doesn't match | -1 (disqualify) |
| Product category matches (account module) | +1 |
| Product category doesn't match | -1 (disqualify) |

A single -1 disqualifies the whole rule (account override returns -1 immediately if base score is -1, `analytic/models/analytic_plan.py:60-61`, `AccountAnalyticPlan.account_ids`). Rules are only considered if `not rule.company_id or not kwargs company or they match` (`analytic/models/analytic_plan.py:247-252`, `AccountAnalyticPlan._get_applicability`). The 0.5 weight exists so company is a tiebreaker, never enough alone to beat a domain match.

**Business domains:** `general` (base), `invoice`/`bill` (account), `timesheet` (hr_timesheet), `expense` (hr_expense), `manufacturing_order` (mrp_account), `purchase_order` (purchase). Each is added via `selection_add` on `account.analytic.applicability.business_domain`.

## 6. Distribution Model Auto-Fill

`account.analytic.distribution.model._get_distribution(vals)` (`analytic/models/analytic_distribution_model.py → _get_distribution`):
1. Get applicable models via `_get_applicable_models(vals)` — models are searched ordered by `sequence, id desc` (`analytic/models/analytic_distribution_model.py:14`, `AccountAnalyticDistributionModel._order`)
2. For each model in order, skip if any of its accounts' root plans is already in `applied_plans`
3. Otherwise merge with union operator `|` and add its root plans to `applied_plans`
4. Return combined distribution — first model (by sequence) to fill a root plan wins it (see ANG3)

**Matching fields:** `partner_id`, `partner_category_id`, `company_id` (base, via `_create_domain` → `[(fname,'in',[value,False])]`; `partner_category_id` adds `False`); `product_id`, `product_categ_id`, `account_prefix` added by the account module. `account_prefix` is special: `_create_domain` returns `[]` (no SQL) and matching is done in Python by `startswith` over `;`/`,`-split prefixes (`account/models/account_analytic_distribution_model.py:34-49`, `AccountAnalyticDistributionModel._get_applicable_models`).

## 7. Analytic Line Creation from Invoice

### Trigger: `account.move.action_post()` → `line._create_analytic_lines()`

1. `_validate_analytic_distribution()` — checks mandatory plans on product lines
2. For each move line → `_prepare_analytic_lines()`:
   - Parse each distribution key (comma-separated account IDs)
   - Calculate: `amount = -(line.balance * percentage / 100)`
   - Track cumulative distribution_on_each_plan for rounding
   - Set plan columns, general_account_id, partner_id, product_id, category
3. `_round_analytic_distribution_line()` — rounds amounts, distributes rounding error proportionally
4. Batch create with `skip_analytic_sync=True` context

### Back-sync: `_inverse_analytic_distribution()` (when distribution edited on posted move)
1. Fetch old distribution from DB (atomic SQL)
2. Merge old + new via `_merge_distribution()`
3. Unlink old analytic lines
4. Recreate via `_create_analytic_lines()`
5. Context `skip_analytic_sync=True` prevents recursion

### Reverse sync: `_update_analytic_distribution()` (when analytic lines modified directly)
- Recalculates `analytic_distribution` JSON from existing `analytic_line_ids`
- Formula: `{key: -analytic_line.amount / line.balance * 100}`

## 8. account.analytic.account Balance Computation

```python
_compute_debit_credit_balance():  # analytic/models/analytic_account.py:140-178
  # For each plan bucket, two _read_group on account.analytic.line:
  #   groupby = [plan._column_name(), 'currency_id']   # NOT plan_id — the dynamic column
  #   credit group: domain amount >= 0 ; debit group: amount < 0
  # Converts each currency sum to company currency at fields.Date.today() rate (NOT line date)
  # debit = -sum(debit), credit = sum(credit), balance = credit - debit
  # from_date/to_date context keys filter lines (:151-154); rate still today's
```
Note: `debit`/`credit` Monetary fields are gated behind `account.group_account_readonly` once `account` is installed (`account/models/account_analytic_account.py:18-19`, `AccountAnalyticAccount.debit`). See ANG43.
