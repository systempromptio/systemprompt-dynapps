<!-- verified: v18.0 @ 2026-07 (account_reports EE; anclas verificadas vs fuente) -->
# Informes contables (`account_reports`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**: "¿se puede X?" + **complejidad** + **origen**. Método en [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md)/[`integrations.md`](integrations.md); trampas en [`gotchas.md`](gotchas.md) (ARG*). Diffs v19 en [`v19_changes.md`](v19_changes.md).
> Veredicto: **EE** (todo el módulo `depends: accountant` → Enterprise) · **NO** (no soportado).
> **Clave**: en CE `account` **no hay motor de informes dinámicos**; balance, P&L, libro mayor, aged, impuestos "de verdad" (comparables, drill-down, exportables) son **Enterprise**. Este motor (`account.report`) es además la base sobre la que se montan los **modelos fiscales localizados** (ES → ver `ecosystem/l10n_es`).

## Motor de informes

**ARCAP1. ¿Constructor de informes dinámicos configurable?** — **EE** · media · `account_reports/models/account_report.py::AccountReport` (`account.report`, `custom_handler_model_id`, `_get_lines`) · Informes definidos por líneas/columnas/expresiones y motores de cálculo; permite crear informes a medida sin código. Base de todos los de abajo.

**ARCAP2. ¿Comparativa entre periodos (mes/año anterior)?** — **EE** · baja · `account_report.py::_init_options_comparison` (`filter_period_comparison`, `period_order`) · Columnas comparadas y variación; se activa en las opciones del informe.

**ARCAP3. ¿Drill-down de la línea del informe al asiento/apunte?** — **EE** · baja · motor `account.report` (líneas expandibles + auditoría de celda a `account.move.line`) · Cada saldo se abre hasta los apuntes que lo componen.

**ARCAP4. ¿Anotaciones / notas sobre líneas del informe?** — **EE** · baja · `account_report.py::AccountReportAnnotation` (`account.report.annotation`, campos `text`/`date`/`line_id`) · Notas persistentes por línea/fecha en el informe.

## Informes financieros

**ARCAP5. ¿Balance de situación (Balance Sheet)?** — **EE** · baja · `data/balance_sheet.xml` (`account.report`) + `models/balance_sheet.py`.

**ARCAP6. ¿Pérdidas y ganancias (P&L)?** — **EE** · baja · `data/profit_and_loss.xml` (record `profit_and_loss`, columna + líneas `account.report.line`).

**ARCAP7. ¿Resumen ejecutivo (Executive Summary)?** — **EE** · baja · `data/executive_summary.xml` + `models/executive_summary_report.py` · KPIs de rentabilidad/tesorería en un vistazo.

**ARCAP8. ¿Estado de flujos de efectivo (Cash Flow)?** — **EE** · baja · `data/cash_flow_report.xml` + `models/account_cash_flow_report.py`.

**ARCAP9. ¿Balance de sumas y saldos (Trial Balance)?** — **EE** · baja · `models/account_trial_balance_report.py` (`account.trial.balance.report.handler`) + `data/trial_balance.xml`.

**ARCAP10. ¿Libro mayor y mayor de terceros (General / Partner Ledger)?** — **EE** · baja · `account.general.ledger.report.handler` / `account.partner.ledger.report.handler` (`models/account_general_ledger.py`, `account_partner_ledger.py`).

**ARCAP11. ¿Aged receivable / payable (vencidos de cobro/pago)?** — **EE** · baja · `models/account_aged_partner_balance.py` (`account.aged.receivable.report.handler` / `account.aged.payable.report.handler`) · Antigüedad de saldos por tramos.

**ARCAP12. ¿Libro diario (Journal report)?** — **EE** · baja · `models/account_journal_report.py` + `data/journal_report.xml` · Libro diario oficial exportable (usado por libros ES en `l10n_es_reports`).

## Fiscal

**ARCAP13. ¿Informe de impuestos genérico + asiento de cierre de IVA?** — **EE** · media · `models/account_generic_tax_report.py::_custom_options_initializer` (botón `action_periodic_vat_entries`, "Closing Entry") · Declaración de IVA genérica y generación del asiento de cierre del periodo.

**ARCAP14. ¿Motor para modelos fiscales localizados (ES 303/347/390…)?** — **EE** · media · los modelos son records `account.report` con `custom_handler_model_id` sobre este motor · Por eso las declaraciones AEAT dependen de Enterprise (o de OCA como alternativa). Ver `ecosystem/l10n_es` (ESCAP5-11).

## Otros

**ARCAP15. ¿Revalorización de divisa (multicurrency, no realizada)?** — **EE** · media · `models/account_multicurrency_revaluation_report.py` + `wizard/multicurrency_revaluation.py` · Informe de diferencias no realizadas + asiento de revalorización de saldos en divisa.

**ARCAP16. ¿Ingresos / gastos diferidos (deferred)?** — **EE** · media · `models/account_deferred_reports.py` (`account.deferred.report.handler`, `deferred_start_date`/`deferred_end_date`) · Periodificación de ingresos y gastos con su informe.

**ARCAP17. ¿Extracto de cliente (customer statement)?** — **EE** · baja · `models/account_customer_statement.py` (`account.customer.statement.report.handler`) · Estado de cuenta del cliente exportable/enviable.

**ARCAP18. ¿Exportar PDF/XLSX + envío automático por email?** — **EE** · baja · `account_report.py::export_to_pdf` / `export_to_xlsx`; envío `wizard/account_report_send.py::action_send_and_print` + cron `data/report_send_cron.xml` → `_cron_account_report_send` · Exportación e informes programados por correo.
