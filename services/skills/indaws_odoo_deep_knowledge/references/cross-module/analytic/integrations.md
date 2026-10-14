# Analytic Integrations — Odoo 18.0

> Documento de **propagation chains**: cómo la distribución analítica viaja
> sale → purchase → stock → mrp → project → timesheet → expense → pos → budget → reporting.
> Cada chain cruza a los gotchas del dominio (prefijo **ANG**, ver `gotchas.md`).
> Para el detalle del comportamiento y el `path:line` de cada caveat, sigue el ID.
> Sección §11 (v19 changes) al final.

## 1. Sale → Invoice → Analytic Lines

### Propagation chain
```
sale.order.line.analytic_distribution
  │ _compute_analytic_distribution() @depends: partner_id, product_id
  │ Params: product_id, product_categ_id, partner_id, partner_category_id, company_id
  ↓
sale.order.line._prepare_invoice_line()
  │ _set_analytic_distribution(inv_line_vals)
  │ Copies: SOL.analytic_distribution → AML.analytic_distribution
  ↓
account.move.line._related_analytic_distribution()
  │ Fallback: if no distribution on AML, pull from sale_line_ids[0]
  ↓
account.move.action_post()
  └─ AML._create_analytic_lines() → account.analytic.line records
```
> Copy verbatim, no recompute: **ANG9**. AAL only on POST (draft = sin líneas): **ANG11**.
> Editar una AAL hace back-sync a la distribución del AML: **ANG12**.

### Validation points
- `SO.action_quotation_send()` → `_validate_analytic_distribution()`
- `SO.action_confirm()` → `_validate_analytic_distribution()`
- `account.move.action_post()` → `_validate_analytic_distribution()` (mandatory plans check)
> Mandatory plans bloquean el post si no suma 100%: **ANG4** (applicability `company_dependent`: **ANG31**).

### sale_project enhancement
If project context active (ctx_project, product.project_id, or order.project_id):
- Gets project's analytic accounts
- Merges missing plans into existing distribution
- Does NOT replace already-set plans
> Merge multi-plan vía cartesian product: **ANG6**. El plan "Project" usa `account_id`, no `x_plan{N}_id`: **ANG5**.

### Reward lines (sale_loyalty)
- Reward/discount lines are separate lines with own `analytic_distribution` (typically empty)
- Do NOT inherit analytic from the original SO line
- Need distribution model or custom override for analytic tracking
> Caveat formalizado: **ANG13**.

## 2. Purchase → Bill → Analytic Lines

### Propagation chain
```
purchase.order.line.analytic_distribution
  │ _compute_analytic_distribution() @depends: product_id, order_id.partner_id
  │ Same matching params as Sale
  ↓
POL._prepare_invoice_line()
  │ Copies: POL.analytic_distribution → AML.analytic_distribution
  ↓
account.move.line._related_analytic_distribution()
  │ Fallback: if no distribution on AML, pull from purchase_line_id
  ↓
account.move.action_post()
  └─ AML._create_analytic_lines() → account.analytic.line records
```

> Distribución por score, no por sequence (un PO line puede recibir un modelo genérico o específico): **ANG3**. AAL solo en POST: **ANG11**.

### Anglo-Saxon price difference
`purchase_stock` → `_prepare_pdiff_aml_vals()`:
- COGS adjustment lines inherit `analytic_distribution` from the invoice line
- Both expense and purchase line account entries get the distribution

### Per-line distribution
Each `purchase.order.line` has independent `analytic_distribution`. Different lines can target different projects/departments.

## 3. Stock → Valuation → Analytic Lines

### Entry point
```
stock.move._inverse_picked() (state → done)
  └─ _account_analytic_entry_move()
     └─ _prepare_analytic_lines()
        └─ account.analytic.account._perform_analytic_distribution(
             distribution, amount, unit_amount, existing_lines, self)
```

### _get_analytic_distribution() (stub)
Base `stock_account` returns `{}` (empty, `stock_account/models/stock_move.py:772-773`, `StockMove._get_analytic_distribution`). Must be overridden:
- `sale_stock`: pulls from `sale_line_id.analytic_distribution` via `_prepare_procurement_values()` (`sale_stock/models/stock.py:27-28`, `StockMove._prepare_procurement_values`)
- Stock move carries `analytic_distribution` from procurement values
> Moves sin origen SO (manual, MRP component, internal) → sin analítica: **ANG10** (generalizado a dropship/back-order/manual en **ANG36**).

### _perform_analytic_distribution() (core engine)
Located in `stock_account/models/analytic_account.py`:
1. Parse account IDs from distribution keys
2. For existing AAL: update if in distribution, delete if not
3. For new accounts: create via `obj._prepare_analytic_line_values()`
4. Round using `plan._calculate_distribution_amount()`
5. Supports `additive=True` mode (add to existing instead of replace)

### Edge cases
Cada caso está formalizado como gotcha (ver `gotchas.md`); `path:line` allí.

| Scenario | Behavior | Gotcha |
|----------|----------|--------|
| **Returns** | Return moves trigger inverse analytic entries (negative amounts); si el neto es 0 se hace `unlink()` de la AAL original (no deja par +/-) | **ANG34** |
| **Dropshipping** | Moves still carry sale_line_id; normal analytic flow | **ANG36** |
| **Back-orders** | Split moves keep their own analytic_distribution (not merged) | **ANG36** |
| **Anglo-Saxon** (`valuation='real_time'`) | SVL account move lines propagate analytic_distribution; amount = SVL exacto | **ANG35** |
| **Continental / manual** (no real_time) | Estima al `standard_price` en picking; reconcilia vía SVL periódica | **ANG35** |
| **Landed costs** | No explicit analytic handling in base; applies via SVL (hereda la del move) | **ANG37** |

## 4. MRP → Work Orders → Analytic Lines

### Manufacturing Order
- MO itself does NOT have `analytic_distribution` field
- Analytic propagation is through **work orders** and **stock moves**
> MO sin campo analytic: **ANG15**. CE aplica la misma distribución a todos los moves: **ANG16**.

### Work Orders (CE + Enterprise)
Work centers have `analytic_distribution`. During WO duration changes:
```
mrp.workorder._create_or_update_analytic_entry()
  └─ _perform_analytic_distribution(
       workcenter.analytic_distribution,
       amount = -(hours * costs_hour),
       unit_amount = hours,
       existing_lines, self)
```

Fields on analytic lines from WO:
- `category = 'manufacturing_order'`
- `ref = production.name`
- `name = "[WC] workcenter_name"`
- `product_id`, `product_uom_id = hour UoM`
> Costes WO siempre negativos (`amount = -(hours*costs_hour)`): **ANG17**. `extra_cost` no genera AAL separada: **ANG18**.

### Component/Finished Good moves
- Analytic via stock_account flow (see §3)
- CE: MO's analytic distribution applies uniformly to ALL moves — no per-component assignment
- Enterprise: work orders can have own distribution but only for labor costs

### Labour cost posting (mrp_account)
`_post_labour()` → creates account.move for work center costs → links to workorder time entries

## 5. Project → Analytic Account

### Auto-creation
`project.project._create_analytic_account()`:
- Auto-creates `account.analytic.account` linked to project plan
- Name synced between project and analytic account
- Deleting project deletes empty analytic accounts

> `allow_timesheets` exige `account_id` (auto-creado al activar; imports que saltan el toggle fallan): **ANG19**.

### Key fields on project.project
| Field | Type | Notes |
|-------|------|-------|
| `account_id` | M2O (analytic.account) | Linked analytic account, copy=False |
| `company_id` | Computed/inverse | From analytic account / partner |

### Profitability data sources
`project.project._get_profitability_items()` aggregates from:

| Source | Module | Revenue/Cost | Method |
|--------|--------|-------------|--------|
| Timesheets | sale_timesheet | Both | Groups by `timesheet_invoice_type` |
| Invoices | project_account | Both | `account.move.line` by account_id |
| Stock valuation | project_stock_account | Cost | `stock.valuation.layer` by product |
| Expenses | hr_expense | Cost | Posted/done expenses matching analytic account |
| Purchase orders | purchase | Cost | Direct-linked AND analytic-matching POs |
> Riesgo de doble conteo entre fuentes (hook `_get_already_included_...`): **ANG21**. El panel "to bill"/"to invoice" lee AML draft, no AAL: **ANG33** (ver PRJG33).

## 6. Timesheet = Analytic Line

Timesheets ARE `account.analytic.line` records (not a separate model).

### Creation
- `project_id` required (→ determines `account_id`)
- `task_id` optional
- `_timesheet_preprocess_get_accounts()` populates all analytic plan fields from project

### Amount computation
```python
cost = employee._hourly_cost()  # or project mapping cost
amount = -(unit_amount * cost)
amount = employee.currency_id._convert(amount, account.currency_id, company, date)
```
Only recalculated when `unit_amount`, `employee_id`, or `account_id` change.
> Cambiar `hourly_cost` del empleado no recalcula timesheets ya existentes: **ANG20**.

### Sale Timesheet (billing)
Adds `so_line` (sale order line to bill), `timesheet_invoice_type`:
- `billable_time`, `billable_fixed`, `billable_milestones`, `billable_manual`
- `non_billable`, `timesheet_revenues`, `other_revenues`, `other_costs`

## 7. Expense → Invoice → Analytic Lines

### Analytic on hr.expense
`hr.expense` inherits `analytic.mixin`. Auto-computed via distribution models:
```python
params = {
    'product_id', 'product_categ_id',
    'partner_id': employee.work_contact_id.id,
    'partner_category_id': employee.work_contact_id.category_id.ids,
    'account_prefix': expense.account_id.code,
    'company_id'
}
```

### Flow
```
hr.expense.analytic_distribution
  → hr.expense.sheet (approve)
  → account.move created
  → account.move.line.analytic_distribution = expense.analytic_distribution
  → On posting → account.analytic.line records
```

### Split
`_split_amounts()` creates copies per tax bracket — each copy keeps same `analytic_distribution`.
> No hay analítica por tramo de impuesto: **ANG14**.

## 8. POS — NO Native Analytic Support

### Current state
- `pos.order.line` does NOT inherit `analytic.mixin`
- Closing entry (`pos.session._create_account_move()`) does NOT set `analytic_distribution`
- `_accumulate_amounts()` aggregates by payment method, not by analytic account
> Order line sin `analytic.mixin`: **ANG23**. Closing entry sin distribución (POS invisible en analítica): **ANG22**.

### Injection points
| Method | Line | What to override |
|--------|------|-----------------|
| `_get_sale_vals()` | Revenue lines | Add `analytic_distribution` |
| `_get_tax_vals()` | Tax lines | Add distribution |
| Stock expense/output lines | COGS | Add distribution |
| Receivable lines | Payments | Usually not needed |

### Indaws solution
`indaws_pos_analytic` extends `account.analytic.distribution.model` with `pos_config_id` to filter distributions per POS configuration. See `indaws.md`.

## 9. Budget (Enterprise: account_budget)

### Models
```
budget.analytic (header)
  ├── state: draft → confirmed → revised → done | canceled
  ├── budget_type: revenue | expense | both
  ├── parent_id/children_ids: revision tracking
  └── budget_line_ids → budget.line

budget.line (inherits analytic.plan.fields.mixin)
  ├── budget_amount: Monetary
  ├── date_from, date_to: from parent
  ├── Computed: achieved_amount, committed_amount, theoritical_amount
  └── Plan fields: account_id, x_plan{id}_id (match analytic accounts)

budget.report (SQL view — virtual model)
  ├── Query 1: budget_line → budget amounts
  ├── Query 2: account.analytic.line → achieved amounts (invoiced)
  └── Query 3: purchase.order.line → committed amounts (confirmed PO, not yet invoiced)
```

### Budget vs Actual matching
Budget lines match analytic lines by:
- Date range (aal.date within bl.date_from/date_to)
- Plan-by-plan account match (exact, not parent/child)
- Company consistency
- Budget type (revenue/expense) vs account type
> Match exacto por plan, no parent/child: **ANG24**. Achieved viene de AAL (devengo, no caja): **ANG26**.

### Purchase integration
- `purchase.order.line.analytic_json`: JSON for budget SQL queries
- `budget_line_ids`: computed, finds matching budget lines
- `is_above_budget`: warning when committed + uncommitted > budget_amount
- `create_revised_budget()`: copies parent with REV prefix → parent moves to revised state
> Committed solo cuenta POs confirmados (draft excluido): **ANG25**.

## 10. Reporting (Enterprise: account_reports)

### Analytic groupby — shadowing mechanism
Any account report can become analytic-grouped via `filter_analytic_groupby`:

1. Creates temp view `analytic_temp_account_move_line`
2. Transforms analytic lines into account.move.line schema:
   - `amount` → `debit` (if < 0: abs), `credit` (if > 0), `balance` (negated)
   - Multi-plan columns → single `analytic_distribution` field
3. Intercepts `account.move.line._where_calc()`:
   - Replaces table reference with temp view when `account_report_analytic_groupby` in context
4. Standard report logic runs unchanged on transformed data

### Available on all reports
General Ledger, Trial Balance, Partner Ledger, Aged Payable/Receivable, Tax Report, Cash Flow, P&L, Balance Sheet, Sales Report, Journal Report.

> El shadow table niega los importes (`balance = -aal.amount`): **ANG28**. Multi-plan groupby = column explosion: **ANG29**.

### Filtering
```python
# Real table — uses GIN index + array intersection
('analytic_distribution', 'in', account_ids)  # uses && operator

# Shadow table — single ID comparison
('analytic_distribution', 'IN', account_ids)
```
> GIN index solo si `analytic_distribution` es stored: **ANG27**. `_read_group` sobre `analytic_distribution` solo soporta `__count`: **ANG8**. `ilike` resuelve display_name→IDs (puede matchear de más): **ANG32**.

### Audit trail
Cell click → transforms domain from move_line to analytic_line → field mapping: `account_id` → `general_account_id` → opens analytic line action.

## 11. v19 changes (analytic)

`analytic` v19 sigue en `version 1.2` pero ahora **depende de `uom`** (v18 no; `v19 analytic/__manifest__.py` `depends = ['base','mail','uom']`). Cambios verificados que afectan a las chains de arriba:

### account.analytic.line
- **Mixin renombrado**: `AnalyticPlanFields` → `AnalyticPlanFieldsMixin` (el `_name` sigue siendo `analytic.plan.fields.mixin`). `v18 analytic/models/analytic_line.py:11`, `class AnalyticPlanFields` vs `v19 analytic/models/analytic_line.py:11`, `class AnalyticPlanFieldsMixin`. Impacto: código que importe/herede la **clase** por nombre se rompe; herencia por `_inherit='analytic.plan.fields.mixin'` no cambia.
- **UoM simplificada**: desaparece `product_uom_category_id` y el `domain` por categoría; `product_uom_id` pasa a `string='Unit'`. `v18:181-186` → `v19:188-191`. Ligado a la nueva dependencia `uom`.
- **Nuevo `default_get`**: si `default_auto_account_id` está en contexto, precarga la columna `x_plan{N}_id` del plan de esa cuenta (`v19 analytic/models/analytic_line.py:101-107`, `AnalyticPlanFieldsMixin.default_get`).
- **Filtro de año fiscal reescrito**: v18 interceptaba `('date','=','fiscal_start_year')` en `_condition_to_sql` (`v18:262`); v19 lo sustituye por el campo no almacenado `fiscal_year_search` con `search='_search_fiscal_date'` (`v19:223,272`). Migración: usar `('fiscal_year_search','=',True)`.

### account.analytic.distribution / analytic.mixin
- **`_condition_to_sql` eliminado → `search=` real**: el `analytic_distribution` ahora declara `search="_search_analytic_distribution"` que devuelve `Domain.custom(to_sql=...)` (`v18 analytic/models/analytic_mixin.py:73`, `AnalyticMixin._condition_to_sql` → `v19 analytic/models/analytic_mixin.py:16-21`, `AnalyticMixin.analytic_distribution` declara el `search=`, resuelto en `v19 analytic/models/analytic_mixin.py:80-112`, `AnalyticMixin._search_analytic_distribution`). Misma semántica (`&&` GIN, resolución display_name→IDs) pero vía la nueva API `Domain`. `_search_distribution_analytic_account_ids` ahora maneja operadores `any`/`not any` (`v19:64-72`).
- **`init()` usa `table_exists()`** en vez de query manual a `information_schema` (`v19 analytic/models/analytic_mixin.py:34`, `AnalyticMixin.init`); el GIN index es idéntico.
- **Operadores negativos**: `_search_auto_account` devuelve `NotImplemented` para `Domain.NEGATIVE_OPERATORS` (`v19 analytic_line.py`).
> Migración de overrides de `_condition_to_sql` sobre analítica: **ANG38**.

### account.analytic.plan / account
- Índices añadidos: `parent_id` (`index='btree_not_null'`), `analytic_plan_id`, `plan_id` (`index=True`), `account.analytic.account.partner_id` pasa de `auto_join=True` a `bypass_search_access=True` + `index='btree_not_null'` (v19: `analytic/models/analytic_plan.py:29-36`, `AccountAnalyticPlan.parent_id`; `analytic/models/analytic_plan.py:411`, `AccountAnalyticApplicability.analytic_plan_id`; `analytic/models/analytic_account.py:38-42`, `AccountAnalyticAccount.plan_id`; `analytic/models/analytic_account.py:67-74`, `AccountAnalyticAccount.partner_id`). Impacto: name_search más rápido; código que dependía de `auto_join` en joins manuales debe revisarse.
- `plan.unlink()` ahora hace `clear_cache('stable')` en vez de `clear_cache()` (`v19 analytic/models/analytic_plan.py:269`, `AccountAnalyticPlan.unlink`).
- `_read_group` de balance/debit/credit soporta variantes `:sum_currency` (conversión a moneda de compañía) además de `:sum` (`v19 analytic/models/analytic_account.py:130-160`, `AccountAnalyticAccount._read_group_select`).

### Downstream sin cambios de mecanismo
- `stock_account._get_analytic_distribution()` sigue siendo stub `{}` (`v19 stock_account/models/stock_move.py:246`, `StockMove._get_analytic_distribution`).
- `mrp_account` mantiene `_create_or_update_analytic_entry` / `_perform_analytic_distribution` (`v19 mrp_account/models/mrp_workorder.py:42-49`, `MrpWorkorder._create_or_update_analytic_entry`).
- `account_budget` continúa siendo **Enterprise** (`odoo_enterprise/19/account_budget`).
- `account_reports` mantiene el shadow `analytic_temp_account_move_line` (Enterprise).
