<!-- verified: v18.0 @ 2026-07 (OCA mis-builder 18.0.1.8.3 — mis_builder; verificado vs $OCA_SRC/v18/mis-builder/mis_builder) -->
# OCA mis_builder — Reporting financiero (v18.0)

`mis_builder` (OCA `mis-builder`) construye informes financieros configurables (balance, P&L,
cash-flow, ratios) definidos como **KPIs con expresiones contables** evaluadas contra
`account.move.line`. Es la alternativa OCA a los Account Reports de Enterprise. Familia:
`mis_builder` (core), `mis_builder_budget` (presupuestos), `mis_builder_cash_flow`, `mis_builder_demo`.
Manifest v18: `version 18.0.1.8.3`, licencia AGPL-3.

El valor está en **cómo se parsean y evalúan las expresiones** (el AEP) y en las trampas del sentinel
`AccountingNone` y del eval sandboxed. Ver `gotchas.md` (MISG).

## Modelos

- **`mis.report`** (`models/mis_report.py`, 1010 líneas) — la PLANTILLA: define los `mis.report.kpi`
  (nombre, expresión, tipo num/pct/str, `compare_method`, `accumulation_method`, `auto_expand_accounts`,
  `multi`/subKPIs) y `mis.report.subkpi`. Reutilizable entre instancias.
- **`mis.report.instance`** (`models/mis_report_instance.py`, 1025 líneas) — la EJECUCIÓN: periodos
  (`mis.report.instance.period`), columnas, `target_move` (posted/all), `analytic_domain`, compañías,
  `source` (actuals/actuals_alt/sumcol/cmpcol). Produce la matriz (`_compute_matrix`).
- **`AccountingExpressionProcessor` (AEP)** (`models/aep.py`, 675) — NO es un modelo Odoo: parsea las
  variables contables de las expresiones, agrupa dominios+modos+cuentas, hace el mínimo de queries
  (`_read_group` sobre aml) y sustituye cada variable por su importe.
- Soporte: `kpimatrix.py` (matriz), `mis_report_style.py` (formato/estilo), `accounting_none.py`
  (sentinel), `mis_safe_eval.py` (eval sandbox), `expression_evaluator.py`, `simple_array.py` (subKPIs),
  `aggregate.py`, `prorata_read_group_mixin.py`, `mis_kpi_data.py`.

## Sintaxis de expresión contable (AEP)

Forma: **`<field><mode>(.fieldname)?[accounts][ml_domain opcional]`** (`mis_builder/models/aep.py:81-144`, `class AccountingExpressionProcessor`, regex `_ACC_RE`).

- **field**: `bal` (saldo = deb−crd), `pbal` (solo saldos positivos), `nbal` (solo negativos),
  `deb`, `crd`, `fld` (suma de un campo custom, requiere `.fieldname`).
- **mode**: `p` variación en el periodo (**DEFAULT** — `bal[70]`==`balp[70]`), `i` saldo inicial,
  `e` saldo final, `u` P&L no distribuido (resultado de ejercicios previos), `s` = legacy de `e`.
- **accounts**: lista de códigos separados por coma con comodín `%` (→ `=like`), o un dominio sobre
  `account.account`, o `[]` = todas las cuentas. Ej: `bal[70]`, `bale[1%]`, `bali[70,60]`,
  `bal[('user_type_id.name','=','Income')]`, `fldp.quantity[60%]`.
- **ml_domain**: dominio opcional sobre `account.move.line` (filtrar por analítica, diario, etc.).

Flujo del AEP: `parse_expr()` por cada expresión → `done_parsing()` (resuelve dominios de cuenta a ids)
→ por periodo `do_queries()` → `replace_expr()` sustituye variables por importes (`mis_builder/models/aep.py:107-129`, `class AccountingExpressionProcessor`).

## Ciclo de evaluación

`ExpressionEvaluator.eval_expressions` → AEP `replace_expr` (mete los números) → `mis_safe_eval`
(eval sandboxed de la expresión ya sustituida, `mis_builder/models/mis_safe_eval.py:13-33`, `def mis_safe_eval`). Los errores NO explotan:
se convierten en valores tipo Excel `#NAME`/`#DIV/0`/`#ERR`. Ver MISG7/MISG8.

## Fuentes de columna (`source`, `mis_builder/models/mis_report_instance.py:21-24`, `SRC_ACTUALS`)

`actuals` (aml estándar) · `actuals_alt` (OTRO modelo de aml, p.ej. budget) · `sumcol` (suma de otras
columnas, sin query) · `cmpcol` (compara dos columnas from/to, sin query). Ver MISG13/MISG14.
