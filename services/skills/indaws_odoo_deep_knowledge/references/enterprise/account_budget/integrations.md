<!-- verified: v18.0 @ 2026-06 -->
# Account Budget — Integrations (Odoo 18.0 Enterprise) — EE

Cómo `account_budget` (v18, modelo `budget.analytic`/`budget.line`) se conecta con el resto. Verificado contra `odoo_enterprise/18/account_budget`.

---

## 1. Analytic (núcleo del módulo en v18)

A diferencia de v17 (cuentas G/L + analítica), **v18 es 100% analítico**.

### analytic.plan.fields.mixin
`budget.line` y `budget.report` heredan este mixin → se crean dinámicamente campos M2o a `account.analytic.account`, uno por **plan analítico** (`x_plan1_id`, `x_plan2_id`, ...). El `post_init_hook` (`_sync_all_plan_column`) genera las columnas para los planes existentes; los planes nuevos las generan al crearse.

### account.analytic.account (ext)
`models/account_analytic_account.py` (14 líneas): añade `budget_line_ids` (O2m) y parchea vistas con `_patch_view()`.

### account.analytic.line (consumo real / achieved)
La query AAL del `budget.report` (`account_budget/reports/budget_report.py:52`, `_get_aal_query`) joina `account_analytic_line` a `budget_line` por:
- Solape de fechas (`aal.date BETWEEN bl.date_from AND bl.date_to`).
- Match de columnas de plan (NULL en budget line = comodín, **ABG1**).
- Company match OR budget line sin company (**ABG11**).
- Signo: expense `*-1`, revenue `*1`.

---

## 2. account.move.line (vía analytic.line)

El presupuesto NO lee `account.move.line` directamente en v18 (v17 sí, vía budgetary positions). El `achieved_amount` viene de `account.analytic.line`, que a su vez las genera `account.move.line` al postear (cada apunte con `analytic_distribution` crea analytic lines). Es decir: **factura posteada → analytic lines → budget.report achieved**. Filtrado por tipo de cuenta en el SQL (expense accounts para budget expense, income para revenue).

`action_open_reference()` en `budget.report` (`account_budget/reports/budget_report.py:213`, `BudgetReport._table_query`): para filas de tipo analytic line, navega al asiento (`move_line_id`) del documento de negocio origen vía `action_open_business_doc()`.

---

## 3. purchase (committed amount, v18 — extraído en v19)

### purchase.order.line (ext) — `models/purchase_order_line.py` (63 líneas)
- `analytic_json` (stored computed JSON) — denormaliza `analytic_distribution` a `[{"rate":0.6,"x_plan1_id":5},...]` para permitir `JSONB_TO_RECORDSET` + CROSS JOIN en el SQL del report (**ABG7**: si modificas el JSON por SQL queda stale).
- `budget_line_ids` (computed O2m), `is_above_budget` (Boolean).

### purchase.order (ext) — `models/purchase_order.py` (37 líneas)
- `is_above_budget`, `is_analytic` (computed). `_compute_above_budget` sólo suma el importe no comprometido cuando `state not in ('purchase','done')` (**ABG9**: el aviso desaparece tras confirmar).

### committed en budget.report
Query POL (`account_budget/reports/budget_report.py:126`, `def _get_pol_query`): POs confirmados/done, `product_qty > qty_invoiced`, fórmula
`(price_subtotal/product_qty)*(product_qty-qty_invoiced)/currency_rate*distribution_rate`. Match por `po.date_order` (**ABG15**: no usa fecha de recepción).

⚠️ **v19**: todo esto se mueve a `account_budget_purchase` (auto_install). El core `account_budget` v19 pierde `committed_amount`/`committed_percentage` y el `line_type='committed'`. Ver v19_changes.md.

---

## 4. budget.report (SQL view, `_auto=False`)

UNION de 3 queries: budget lines (`line_type='budget'`) + analytic lines (`achieved`) + PO lines (`committed`). `_table_query` (`account_budget/reports/budget_report.py:194`, `BudgetReport._get_pol_query`) hace `flush_model()` de 5 modelos en cada acceso (**ABG5**). El context key `budget_report_budget_line_ids` añade un WHERE para no escanear toda la tabla (**ABG6**).

Vistas: list/pivot/graph. `_compute_all()` de `budget.line` lee este view vía `_read_group` con agregados `committed:sum`, `achieved:sum`.

---

## 5. Multi-company

3 `ir.rule` globales (`budget_comp_rule`, `budget_lines_comp_rule`, `budget_report_comp_rule`), todas `['|', ('company_id','in',company_ids), ('company_id','=',False)]`. Budget lines con `company_id=False` matchean analytic lines de CUALQUIER company (**ABG11**). El SQL maneja company con dos sub-queries por fuente (match exacto + NULL company).

---

## 6. Tabla resumen de modelos implicados

| Modelo | Rol | Fichero | Nota versión |
|---|---|---|---|
| `budget.analytic` | Cabecera presupuesto | `models/budget_analytic.py` (104) | v18+ |
| `budget.line` | Línea presupuestaria | `models/budget_line.py` (118) | v18+ |
| `budget.report` | SQL view budget/achieved/committed | `reports/budget_report.py` (224) | committed sale del core en v19 |
| `budget.split.wizard` | Genera presupuestos por periodo×plan | wizard | hardcode `expense` (ABG8) |
| `account.analytic.account` (ext) | budget_line_ids | `models/account_analytic_account.py` (14) | v18+ |
| `purchase.order(.line)` (ext) | committed, analytic_json | `models/purchase_order*.py` | → `account_budget_purchase` en v19 |
| `crossovered.budget(.lines)` + `account.budget.post` | **modelo LEGACY** | — | **sólo v17** (ver legacy_v17.md) |

---

## 7. Cross-version (crítico)

- **v17**: `crossovered.budget` + budgetary positions, depende de `account` (¡disponible en Community!). Ver `legacy_v17.md`.
- **v18**: reescrito a `budget.analytic`/`budget.line`, 100% analítico, depende de `accountant`+`purchase` (Enterprise-only).
- **v19**: purchase/committed extraído a `account_budget_purchase` (auto_install); theoretical pasa a SQL; constraint de fechas en la cabecera. Ver `v19_changes.md`.
