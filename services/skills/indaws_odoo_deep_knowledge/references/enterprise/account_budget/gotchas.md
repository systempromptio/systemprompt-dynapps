<!-- verified: v18.0 @ 2026-06 -->
# Account Budget Gotchas -- Odoo 18.0 Enterprise -- EE

## ABG1: NULL Analytic on Budget Line = Match Everything _ES: El presupuesto consume muchísimo más de lo real y captura apuntes analíticos que no le corresponden porque la línea no tiene cuenta analítica asignada._

If a budget line has no analytic account set for a plan, the SQL condition is `(bl.plan_col IS NULL OR aal.plan_col = bl.plan_col)` -- NULL always passes. A budget line with zero analytic filters will capture ALL analytic lines in the date range, massively inflating achieved/committed amounts. Always set at least one analytic account per budget line.

## ABG2: Theoretical Amount Off-by-One at Day Boundaries _ES: El importe teórico del presupuesto no sale a cero el primer día; ya muestra un pequeño porcentaje aunque no haya pasado tiempo._

The theoretical calculation (`budget.line._compute_theoritical_amount`, `account_budget/models/budget_line.py:77`, `BudgetLine._compute_theoritical_amount`) adds `timedelta(days=1)` to both the total and elapsed periods. On `date_from` itself, elapsed = 1 day (not 0). This means the theoretical amount is never exactly zero, even on the first day. For a 365-day budget, day 1 shows ~0.27% as theoretical. (v19 moves theoretical to the SQL view; verify the boundary there.)

## ABG3: Budget Type "both" Has No Sign Flip for Achieved _ES: Con presupuesto de tipo ambos (ingresos y gastos), los gastos salen con signo negativo y no cuadran comprometido contra realizado._

In the AAL query (`budget.report._get_aal_query`, `account_budget/reports/budget_report.py:52`, `BudgetReport._get_aal_query`), `budget_type='both'` passes `ELSE TRUE` in the WHERE clause (no account type filtering), but the sign logic only flips for 'expense' (`*-1`) or keeps positive for 'revenue'. For 'both', expenses are NOT sign-flipped in the AAL query, meaning expense analytic lines appear negative. However, in the POL query, 'both' applies `*-1`. This asymmetry can produce confusing committed vs achieved comparisons.

## ABG4: Committed Amount Double-Counts During Invoicing Window _ES: El importe comprometido cuenta dos veces el mismo gasto entre confirmar el pedido de compra y validar la factura, sobre todo con facturas en borrador._

The committed amount includes both achieved (invoiced) + unreceived PO amounts. During the period between PO confirmation and invoice posting, the same economic event appears in committed via the PO line. Once the invoice posts and creates analytic lines, it appears in achieved too. The POL query filters `product_qty > qty_invoiced` to avoid this, but the `qty_invoiced` computation depends on posted invoices only -- draft invoices leave the PO amount in committed AND the draft invoice may already be visible in other contexts.

## ABG5: budget.report Flushes 5 Models on Every Access _ES: El informe presupuestario va muy lento cuando hay muchos datos o se consulta dentro de bucles._

`budget.report._table_query` (`account_budget/reports/budget_report.py:194`, `BudgetReport._get_pol_query`) calls `flush_model()` on `account.move.line`, `budget.line`, `account.analytic.line`, `purchase.order`, and `purchase.order.line` every time the SQL view is queried. With large datasets and many pending writes in the ORM cache, this can cause significant latency. Avoid accessing `budget.report` in loops or tight compute methods.

## ABG6: _compute_all Uses Context Key for Performance _ES: Al leer el informe presupuestario desde código propio o un cron, escanea todas las líneas y se ralentiza enormemente._

`budget.line._compute_all()` (`account_budget/models/budget_line.py:60`, `BudgetLine._compute_all`) passes `budget_report_budget_line_ids` via context to the `budget.report` SQL view. If you call `budget_line._compute_all()` without this context (e.g., via a custom cron), the SQL view will scan ALL budget lines. Always ensure the context key is set when reading `budget.report` programmatically.

## ABG7: analytic_json Stored Field Not Auto-Refreshed _ES: Tras modificar la distribución analítica de una línea de compra por SQL, el informe de presupuesto sigue usando datos antiguos._

`purchase.order.line.analytic_json` (`account_budget/models/purchase_order_line.py:8`, `PurchaseOrderLine.analytic_json`, compute `_compute_analytic_json:12`) is a stored computed field depending on `analytic_distribution`. If `analytic_distribution` is modified via SQL or `write()` on the JSON field directly (bypassing ORM compute triggers), `analytic_json` becomes stale. The budget report will then use outdated distribution data. Always modify `analytic_distribution` through the ORM.

## ABG8: Budget Split Wizard Hardcodes expense Type _ES: El asistente de división de presupuestos crea siempre presupuestos de gasto; no hay forma de generar presupuestos de ingresos con él._

`budget.split.wizard.action_budget_split()` (`account_budget/wizards/budget_split_wizard.py:31`, `BudgetSplitWizard.action_budget_split`) creates all budgets with `budget_type='expense'`. There is no way to create revenue budgets via the wizard. Users must manually change the type after creation, but since `budget_type` has `copy=False`, it won't be preserved on copy either.

## ABG9: is_above_budget on PO Considers Uncommitted Amounts Only Before Confirmation _ES: El aviso de presupuesto excedido en el pedido de compra desaparece una vez confirmado el pedido, aunque se haya pasado del presupuesto._

`PurchaseOrder._compute_above_budget()` (`account_budget/models/purchase_order.py:15`, `class PurchaseOrder`) only adds the uncommitted amount (`price_unit * (product_qty - qty_invoiced)`) when `state not in ('purchase', 'done')`. After PO confirmation, `uncommitted_amount=0`, so the warning only considers existing committed amounts on budget lines. This means the warning disappears after confirmation even if the budget is exceeded.

## ABG10: Budget Line Deletion Blocked by base.group_user ACL _ES: Un usuario sin permiso para borrar líneas de presupuesto igualmente las elimina todas al borrar el presupuesto en borrador._

Internal users (`base.group_user`) have CRUD but NOT delete on `budget.line` (`security/ir.model.access.csv`, row `access_budget`). Only `account.group_account_user` can delete. However, cascade delete from `budget.analytic` (`budget.line.budget_analytic_id` has `ondelete='cascade'`, `account_budget/models/budget_line.py:17`, `BudgetLine.budget_analytic_id`) bypasses ACL checks. Deleting a draft budget deletes all its lines regardless of the user's budget line permissions.

## ABG11: Company NULL on Budget = Cross-Company Matching in SQL _ES: Un presupuesto sin empresa asignada consume importes de todas las compañías y mezcla datos entre empresas._

If `budget.analytic.company_id` is set to False (empty), the SQL report matches analytic lines from ALL companies. The ir.rule allows this (`company_id = False` passes). This is by design but can lead to unexpected cross-company budget consumption if a user clears the company field.

## ABG12: Revised State Logic Is Asymmetric _ES: Al revisar un presupuesto el padre queda como revisado para siempre, aunque se cancelen o borren todas las revisiones._

When confirming a budget that has a parent (`budget.analytic.action_budget_confirm`, `account_budget/models/budget_analytic.py:66`, `BudgetAnalytic.action_budget_confirm`), the parent moves to 'revised'. But there's no reverse: canceling or deleting the child does NOT revert the parent back to 'confirmed'. The parent stays 'revised' permanently, even if all its revisions are canceled.

## ABG13: _read_group Override for Computed Fields Is Fragile _ES: Las vistas dinámicas o tabla pivote sobre líneas de presupuesto con importe realizado o teórico se vuelven muy lentas con muchos registros._

`budget.line` overrides `_read_group_select` and `_read_group_postprocess_aggregate` to support `achieved_amount:sum` and `theoritical_amount:sum`. These fetch full recordsets and aggregate in Python. If you add custom aggregators or use these fields in large pivot views, performance degrades linearly with the number of records.

## ABG14: Date Constraint Only on Budget Lines, Not on Budget Header _ES: En v18 se puede poner fecha final anterior a la inicial en la cabecera del presupuesto sin líneas y no salta error hasta crear líneas._

`budget.line._check_date_range()` (`account_budget/models/budget_line.py:49`, `BudgetLine._check_date_range`) validates `date_to >= date_from`, but since these are related fields from the parent, the constraint actually validates the parent's dates via the line. However, there's no direct constraint on `budget.analytic` itself **in v18**. You can set `date_from > date_to` on the budget header if no lines exist yet, and the error only appears when lines are created or recomputed. **v19 fixed this**: it added `budget.analytic._check_dates` (`odoo_enterprise/19/account_budget/models/budget_analytic.py::_check_dates`) directly on the header.

## ABG15: PO Budget Matching Uses date_order, Not Planned Receipt _ES: Un pedido de compra hecho en un trimestre para entrega en otro se imputa al presupuesto del trimestre del pedido, no al de la entrega._

The committed query for purchase orders (`budget.report._get_pol_query`, `account_budget/reports/budget_report.py:112`, `BudgetReport._get_aal_query`) matches budget lines using `po.date_order` against `bl.date_from/date_to`. It does NOT use the planned receipt date or actual receipt date. A PO ordered in Q1 for Q2 delivery will be committed against Q1's budget, not Q2's.

## ABG16: Currency Conversion Not Applied in Budget Report _ES: En entornos multi-moneda el presupuesto compara importes sin convertir la divisa de los apuntes analíticos y sale mal._

The SQL view divides by `po.currency_rate` for PO lines but does not apply any currency conversion for analytic lines. If analytic lines are in a different currency than the budget's company currency, the amounts are compared as-is. This can produce incorrect budget vs actual comparisons in multi-currency environments.

## ABG17: PO budget matching Uses 'any' Operator on Subquery _ES: Al replicar la lógica de emparejamiento de presupuesto en versiones antiguas o por API externa falla por usar un operador de dominio exclusivo de v18._

The `_compute_budget_line_ids` domain on `purchase.order.line` (`account_budget/models/purchase_order_line.py:24`, `PurchaseOrderLine._compute_budget_line_ids`) uses `('budget_analytic_id', 'any', (...))` which is an Odoo 18 domain operator. If you try to replicate this pattern in older versions or in external API calls, it will fail. The `any` operator generates a SQL subquery.

## ABG18: Dynamic Analytic Fields Require post_init_hook _ES: Al crear un plan analítico después de instalar el módulo, las columnas de plan no aparecen en las vistas de presupuesto hasta refrescar o actualizar el módulo._

The `post_init_hook` (`account_budget/__init__.py:9`, `post_init_hook`) calls `account.analytic.plan._sync_all_plan_column()` (defined in base `analytic`, `odoo18/addons/analytic/models/analytic_plan.py::_sync_all_plan_column`) on all analytic plans. If a new analytic plan is created after module installation, the columns are synced automatically by the plan model. But if the module is installed in a database with no plans, and plans are created later, the budget views may not show plan columns until a view cache clear or module upgrade.

## ABG19: The whole model changed between v17 and v18 (no 1:1 migration) _ES: Tras migrar de v17 a v18 los presupuestos hechos sobre cuentas contables quedan huérfanos porque el modelo cambió por completo a analítico._

v17 uses `crossovered.budget` / `crossovered.budget.lines` / `account.budget.post` (budgetary positions over G/L accounts) and depends only on `account` (Community-available). v18 rewrote it to `budget.analytic` / `budget.line`, 100% analytic, depending on `accountant`+`purchase` (Enterprise-only). A v17 budget built against G/L accounts (no analytic) has NO direct v18 equivalent — those budgets are orphaned in the upgrade. See `legacy_v17.md`.

## ABG20: v17 practical_amount reads account.move.line with `-balance` and posted-only _ES: En v17 el importe realizado del presupuesto solo cuenta asientos contabilizados e ignora los borradores, y los gastos aparecen con signo cambiado._

In v17 (`crossovered.budget.lines._get_query_account_analytic_line`, `v17 account_budget.py L147`), when a line has no analytic account it sums `-balance` of `account.move.line` filtered to `parent_state='posted'`. Draft entries don't count, and the sign flip means expenses (positive debit) show positive in practical. This whole G/L path disappears in v18 (achieved comes only from `account.analytic.line`).

## ABG21: budget_type has copy=False — revisions and splits lose the type _ES: Al duplicar un presupuesto de ingresos se convierte silenciosamente en presupuesto de gasto; el tipo no se conserva en copias ni divisiones._

`budget.analytic.budget_type` has `copy=False` (`account_budget/models/budget_analytic.py:42`, `BudgetAnalytic.budget_type`). So `create_revised_budget()` (`account_budget/models/budget_analytic.py:80`, `BudgetAnalytic.create_revised_budget`) explicitly re-passes `budget_type` to preserve it, but ANY plain `copy()` (or the split wizard, which hardcodes `expense`) drops it back to the default. If you duplicate a revenue budget expecting it to stay revenue, it silently becomes expense. Combined with ABG8 (split wizard hardcode), there is no way to bulk-create revenue budgets.
