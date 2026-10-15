<!-- verified: v18.0 @ 2026-06 -->
# Account Budget -- Odoo 18.0 Enterprise -- EE

> **Enterprise**. Módulo `account_budget` en `odoo_enterprise/18/account_budget`, depende de `accountant`+`purchase`. ⚠️ El modelo cambió radicalmente entre versiones: **v17 = `crossovered.budget`** (ver `legacy_v17.md`), **v18+ = `budget.analytic`/`budget.line`** (este doc), **v19 = purchase extraído a `account_budget_purchase`** (ver `v19_changes.md`). En v17 dependía sólo de `account` (estaba en Community); v18 lo hace Enterprise-only.

## 1. Model Map

| Model | Type | Inherits | Description |
|---|---|---|---|
| `budget.analytic` | New | `mail.thread`, `mail.activity.mixin` | Budget header with dates, type, state machine |
| `budget.line` | New | `analytic.plan.fields.mixin` | Budget line with amounts and computed actuals |
| `budget.report` | New (SQL view, `reports/budget_report.py`) | `analytic.plan.fields.mixin` | UNION of budget lines + analytic lines + PO lines for reporting |
| `budget.split.wizard` | Transient | -- | Generates budgets split by period (month/quarter/year) x analytic plans |
| `account.analytic.account` | Inherited | -- | Adds `budget_line_ids` O2M |
| `purchase.order` | Inherited | -- | Adds `is_above_budget`, `is_analytic` computed fields |
| `purchase.order.line` | Inherited | -- | Adds `analytic_json`, `budget_line_ids` (computed O2M), `is_above_budget` |

Dependencies: `accountant` (Enterprise), `purchase`.

Post-init hook: `post_init_hook` (`account_budget/__init__.py:9`, `post_init_hook`) calls `account.analytic.plan._sync_all_plan_column()` (base `analytic`, `odoo18/addons/analytic/models/analytic_plan.py::_sync_all_plan_column`) to create dynamic analytic plan columns on `budget.line` and `budget.report` for all existing plans.

## 2. budget.analytic -- Fields & Lifecycle

### Fields

| Field | Type | Notes |
|---|---|---|
| `name` | Char | Required |
| `parent_id` | M2O -> budget.analytic | "Revision Of", cascade delete |
| `children_ids` | O2M -> budget.analytic | Revisions of this budget |
| `user_id` | M2O -> res.users | Responsible, defaults to current user |
| `date_from` | Date | Required |
| `date_to` | Date | Required |
| `state` | Selection | draft / confirmed / revised / done / canceled |
| `budget_type` | Selection | revenue / expense / both (default: expense) |
| `budget_line_ids` | O2M -> budget.line | Copied on duplicate |
| `company_id` | M2O -> res.company | Defaults to env.company |

### State Machine

```
draft --> confirmed --> done
  |          |
  v          v
canceled   revised (auto when child is confirmed)
```

- `action_budget_confirm()`: Sets parent to 'revised' if parent was 'confirmed'. Sets self to 'revised' if it already has children, otherwise 'confirmed'.
- `action_budget_done()`: Direct state set.
- `action_budget_cancel()`: Direct state set.
- `action_budget_draft()`: Direct state set (can revert from any state).
- Delete only allowed in draft or canceled.

### Revision System

`create_revised_budget()` copies the budget (with `budget_type` preserved), sets `parent_id` to original, prefixes name with "REV", and posts a message with a link to the revision on the original budget.

### Dynamic View Patching

`_get_view()` calls `analytic.plan.fields.mixin._patch_view()` to inject dynamic analytic plan columns into the embedded budget lines list view. This is the same mechanism used across all analytic-aware models.

## 3. Budget Lines (budget.line)

### Fields

| Field | Type | Notes |
|---|---|---|
| `name` | Char | Related to `budget_analytic_id.name` |
| `sequence` | Integer | Default 10 |
| `budget_analytic_id` | M2O -> budget.analytic | Required, cascade, indexed |
| `date_from` / `date_to` | Date | Related stored from parent budget |
| `currency_id` | M2O -> res.currency | Related from company |
| `budget_amount` | Monetary | Manual input: the budgeted amount |
| `achieved_amount` | Monetary | Computed from `budget.report` (invoiced/billed) |
| `achieved_percentage` | Float | achieved / budget |
| `committed_amount` | Monetary | Computed: achieved + confirmed PO not yet invoiced |
| `committed_percentage` | Float | committed / budget |
| `theoritical_amount` | Monetary | Time-proportional expected amount |
| `theoritical_percentage` | Float | theoretical / budget (aggregator=avg) |
| `company_id` | M2O | Related stored from parent |
| `is_above_budget` | Boolean | True when achieved > budget |
| `budget_analytic_state` | Selection | Related stored from parent state |
| Dynamic `x_plan_*` | M2O -> account.analytic.account | One per analytic plan, from `analytic.plan.fields.mixin` |

### _compute_all() -- Achieved & Committed

Reads from `budget.report` SQL view via `_read_group()`:
```python
self.env['budget.report'].with_context(budget_report_budget_line_ids=self.ids)._read_group(
    domain=[('budget_line_id', 'in', self.ids)],
    groupby=['budget_line_id'],
    aggregates=['committed:sum', 'achieved:sum'],
)
```
The context key `budget_report_budget_line_ids` is used inside the SQL view to add a WHERE filter, avoiding a full table scan.

### _compute_theoritical_amount() -- Time-Proportional Budget

```
elapsed = min(max(today, date_from), date_to) - date_from + 1 day
total = date_to - date_from + 1 day
theoretical = (elapsed / total) * budget_amount
```

- Before `date_from`: theoretical = budget_amount * (1 day / total days) -- the +1 day means it's never exactly 0
- After `date_to`: theoretical = budget_amount
- Between: linear interpolation

### Custom _read_group for Computed Fields

`achieved_amount:sum`, `theoritical_amount:sum`, and `theoritical_percentage:avg` are not stored, so they override `_read_group_select()` and `_read_group_postprocess_aggregate()` to fetch recordsets and manually aggregate in Python. This is necessary because these computed fields cannot be aggregated in SQL.

### action_open_budget_entries()

Opens `budget.report` action filtered by the budget line's analytic accounts across all plans. Builds a dynamic domain by iterating all analytic plans and adding filters for non-empty plan fields.

## 4. Budget vs Actual Computation

The `budget.report` model (`_auto = False`) is a SQL UNION of three queries:

### Query 1: Budget Lines (bl)
Source: `budget_line` table. Produces rows with `line_type='budget'`, `budget=budget_amount`, `committed=0`, `achieved=0`.

### Query 2: Analytic Lines (aal)
Source: `account_analytic_line` joined to `budget_line` on:
- Date range overlap (`aal.date BETWEEN bl.date_from AND bl.date_to`)
- Matching analytic plan columns (NULL on budget line = wildcard match)
- Company match OR budget line has no company

Sign logic:
- Expense budgets: `aal.amount * -1` (analytic lines are negative for expenses)
- Revenue budgets: `aal.amount * 1`

Account type filtering:
- Expense budget: only expense accounts, or NULL account with non-invoice category, or negative "other"
- Revenue budget: only income accounts, or NULL account with positive "other"
- "Both" type: no filtering

Two sub-queries UNIONed: one for matching company, one for NULL company on budget line.

### Query 3: Purchase Order Lines (pol)
Source: `purchase_order_line` with confirmed/done POs, CROSS JOINed with `JSONB_TO_RECORDSET(pol.analytic_json)` to explode the analytic distribution.

Committed amount formula:
```
(price_subtotal / product_qty) * (product_qty - qty_invoiced) / currency_rate * distribution_rate
```

Only included when `product_qty > qty_invoiced` (uninvoiced portion) and budget type is not 'revenue'.

The `analytic_json` field on `purchase.order.line` is a stored computed JSON that denormalizes `analytic_distribution` into a format with `rate` and plan column names, enabling the CROSS JOIN.

### Performance Note
`_table_query` calls `flush_model()` on 5 models before building the SQL. The context key `budget_report_budget_line_ids` adds a WHERE clause to narrow down results. Without it, the query scans all budget lines.

## 5. Analytic Integration (Plans, Accounts, Distribution)

### analytic.plan.fields.mixin

Both `budget.line` and `budget.report` inherit this mixin. It dynamically creates `Many2one` fields to `account.analytic.account` for each analytic plan (e.g., `x_plan1_id`, `x_plan2_id`).

### How Matching Works

Budget lines can specify an analytic account for zero or more plans. When computing actuals:
- If the budget line has a value for a plan column, only analytic lines with that exact account are matched.
- If the budget line has NULL for a plan column, all analytic lines match regardless of their value for that plan.
- This means a budget line with NO analytic accounts set will match ALL analytic lines in the date range (wide catch-all).

### analytic_json (Purchase Lines)

Stored JSON computed from `analytic_distribution`. Format:
```json
[{"rate": 0.6, "x_plan1_id": 5}, {"rate": 0.4, "x_plan1_id": 7}]
```
This enables SQL-level CROSS JOIN for budget report computation without Python processing.

### account.analytic.account Extension

Adds `budget_line_ids` O2M via the dynamically generated field (the `auto_account_id` is the analytic account field from the mixin). Also patches views with `_patch_view()`.

## 6. Reporting (Budget Analysis, Dashboards)

### budget.report (SQL View)

- View modes: list, pivot, graph
- `action_open_reference()`: Opens the source document -- for analytic lines, navigates to the related journal entry's business document; for others, opens the record form.
- Fields available for pivot/graph: `budget`, `committed`, `achieved`, `date`, `line_type`, `company_id`, `user_id`, plus all dynamic analytic plan fields.

### Budget Split Wizard

`budget.split.wizard` generates multiple budgets from a date range:
1. Select date range and period (month/quarter/year)
2. Select analytic plans -- the wizard creates a Cartesian product of all accounts in those plans
3. Creates one `budget.analytic` per period, each with budget lines for every combination of analytic accounts
4. All created with `budget_type='expense'` (hardcoded)
5. Returns a list view of all created budget lines grouped by budget

## 7. Multi-Company Behavior

- `budget.analytic.company_id` defaults to `env.company`, but is optional (can be False).
- Three global `ir.rule` records enforce multi-company isolation:
  - `budget_comp_rule` on `budget.analytic`
  - `budget_lines_comp_rule` on `budget.line`
  - `budget_report_comp_rule` on `budget.report`
- All use: `['|', ('company_id', 'in', company_ids), ('company_id', '=', False)]`
- Budget lines with `company_id=False` match analytic lines from ANY company in the SQL view.
- The SQL report handles company matching with two sub-queries per source: one for exact company match, one for NULL company on the budget line.

## 8. Security & Groups

### Access Control (ir.model.access.csv)

| Model | Group | R | W | C | D |
|---|---|---|---|---|---|
| budget.analytic | account.group_account_readonly | 1 | 0 | 0 | 0 |
| budget.analytic | account.group_account_user | 1 | 1 | 1 | 1 |
| budget.line | account.group_account_readonly | 1 | 0 | 0 | 0 |
| budget.line | account.group_account_user | 1 | 1 | 1 | 1 |
| budget.line | base.group_user | 1 | 1 | 1 | 0 |
| budget.report | base.group_user | 1 | 0 | 0 | 0 |
| budget.split.wizard | account.group_account_user | 1 | 1 | 1 | 1 |

Key observations:
- Any internal user (`base.group_user`) can read budget reports.
- Any internal user can read/write/create budget lines (but not delete).
- Only accountants (`group_account_user`) can delete budget lines and manage budgets.
- Read-only accountants can view but not modify.

### Record Rules

All three rules are global (no group restriction) and use `company_ids` (the multi-company field on res.users).

### Group Assignment on Install

`security/account_budget_security.xml` adds `analytic.group_analytic_accounting` to `base.user_root` and `base.user_admin` via data records (NOT via the post-init hook — that hook only syncs plan columns).
