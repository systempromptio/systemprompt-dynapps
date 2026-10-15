<!-- verified: v18.0 @ 2026-07 (OCA mis-builder 18.0.1.8.3 + familia) -->
# OCA mis_builder — Integraciones

## 1. account / account.move.line (la fuente)

mis_builder lee `account.move.line` vía el AEP con `_read_group` sumando debit/credit agrupado por
`account_id, company_id` (`mis_builder/models/aep.py:448-460`, `AccountingExpressionProcessor.do_queries`). Depende de campos estándar del aml: `debit`, `credit`,
`date`, `account_id`, `account_id.include_initial_balance` (BS vs P&L, MISG3/MISG4), `parent_state`
(filtro posted/all, MISG9). Cualquier gotcha de `account` sobre cuándo/cómo se postean apuntes se propaga
al informe: si un asiento está en borrador y el informe es "posted", no aparece (MISG9).

## 2. Analytic (`cross-module/analytic`)

El filtrado analítico se hace por `analytic_domain` sobre el aml (MISG10). En Odoo 18 la analítica del
aml es `analytic_distribution` (JSON), no un m2o — un `analytic_domain` que asuma el modelo viejo
(`analytic_account_id`) fallará o filtrará mal. Cross-ref los gotchas de analytic (ANG) sobre cómo se
consulta `analytic_distribution` (JSON con índice GIN).

## 3. Budget — `mis_builder_budget`

Módulo hermano: añade una fuente de datos presupuestaria (`mis.budget`) consumible como columna
`actuals_alt` o como KPI data. Un informe "real vs presupuesto" = una columna actuals + una columna del
modelo budget (MISG14) + una `cmpcol`/`sumcol` para la desviación (MISG13). El modelo budget debe exponer
los campos que usen las expresiones o salta el `UserError` de campo inexistente (MISG14).

## 4. Cash flow — `mis_builder_cash_flow`

`mis_builder_cash_flow` (en el repo `account-financial-reporting`) añade una fuente de flujo de caja
(previsión sobre facturas/vencimientos). Es un aml-model alternativo más para `actuals_alt`.

## 5. Reporting / export (`cross-module/reporting`)

- **PDF/QWeb**: el informe se renderiza a PDF por el motor QWeb+wkhtmltopdf estándar → aplican los
  gotchas RPTG (CSS viejo de WebKit, paperformat, assets). Cross-ref `cross-module/reporting`.
- **XLSX**: exporta a hoja de cálculo (estilos de `mis_report_style`).
- **Dashboards / KPI**: una `mis.report.instance` se puede embeber como widget/board.

## 6. l10n_es (`ecosystem/l10n_es`)

En despliegues españoles mis_builder se usa para balance y cuenta de PyG con el **PGC** (plan contable
español): las expresiones usan códigos de grupo (`bal[6%]` gastos, `bal[7%]` ingresos, `bale[57%]`
tesorería). El comportamiento P&L-vs-balance de `bale`/`bali` (MISG3) es justo lo que hace que un modelo
de PyG por grupos funcione (acumula al ejercicio) mientras el balance acumula histórico.

## 7. Seguridad / multi-compañía

El informe corre con los permisos del usuario que lo lanza (record rules sobre aml aplican). Multi-company:
fija la moneda del informe si las compañías consultadas no comparten moneda (MISG16). `query_company_ids`
en la instancia controla qué compañías entran.
