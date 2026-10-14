<!-- verified: v18.0 @ 2026-07 (account_accountant EE + account CE boundary; anclas verificadas vs fuente) -->
# Contabilidad Enterprise (`account_accountant`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**: "¿se puede X?" + **complejidad** + **origen**. Método en [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (AAG*). Diffs v19 en [`v19_changes.md`](v19_changes.md).
> Veredicto: **NATIVO** (core CE `account`) · **EE** (lo añade `account_accountant`) · **EE-hermano** (otro módulo Enterprise) · **CONFIG** · **NO**.
> **Clave de venta**: casi toda la *contabilidad de datos* (asientos, conciliación, statements, lock dates, recurrentes) YA está en CE `account`. Lo que compra Enterprise aquí es sobre todo **la conciliación bancaria visual (widget)**, el **ejercicio fiscal** y automatismos. No vender como EE lo que el cliente ya tiene en CE.

## Conciliación bancaria

**AACAP1. ¿Widget visual de conciliación bancaria (extracto ↔ apuntes)?** — **EE** · media · `account_accountant/models/bank_rec_widget.py::BankRecWidget` (`bank.rec.widget`) · La pantalla interactiva de conciliar líneas de extracto es la joya Enterprise; en CE se concilia a mano sin ese asistente.

**AACAP2. ¿Modelos de conciliación (reglas auto-match)?** — **NATIVO** · baja · `account/models/account_reconcile_model.py` (`account.reconcile.model`, campo `auto_reconcile`) · Las reglas existen en CE; lo que EE añade es aplicarlas dentro del widget (AACAP1).

**AACAP3. ¿Conciliación automática masiva (lote + cron)?** — **EE** · media · `wizard/account_auto_reconcile_wizard.py::auto_reconcile` + cron `data/ir_cron.xml` → `_cron_try_auto_reconcile_statement_lines(batch_size=100)` · Barrido automático de líneas pendientes; el asistente/cron es EE.

**AACAP4. ¿Conciliación con diferencia/write-off y diferencia de cambio?** — **NATIVO (motor) / EE (asistente)** · media · reconciliación parcial y asiento de diferencia de cambio son core `account`; el wizard guiado `wizard/account_reconcile_wizard.py` (`force_partials`, `is_write_off_required`) lo añade EE.

## Contabilidad completa vs facturación

**AACAP5. ¿Desbloquea contabilidad completa frente al modo facturación de CE?** — **EE** · baja · instalar `account_accountant` + usuario en `account.group_account_user` (`account/security/account_security.xml`) activa menús/vistas de contabilidad plena (ejercicio, widget, informes). CE `account` cubre facturación + asientos básicos.

**AACAP6. ¿Definición de ejercicio fiscal con fechas no naturales?** — **EE** · baja · `account_accountant/models/account_fiscal_year.py` (`account.fiscal.year`, `_check_dates`) · NO existe en CE; ejercicios partidos/desfasados solo con Enterprise.

**AACAP7. ¿Fechas de bloqueo (lock dates) con control de cambios?** — **NATIVO (campos) / EE (gestión)** · baja · los `fiscalyear_lock_date`/`tax_lock_date` son core en `res.company`; el asistente de cambio con **excepciones por usuario** es EE `wizard/account_change_lock_date.py` (`fiscalyear_lock_date_for_me`/`_for_everyone`).

## Operativa / cierre

**AACAP8. ¿Dashboard contable por diario (KPIs, pendientes)?** — **NATIVO (base) / EE (a conciliar)** · baja · el kanban de diarios es core; `account_accountant/models/account_journal_dashboard.py::action_open_reconcile`/`action_open_to_check` añade los accesos "a conciliar/a revisar" del widget.

**AACAP9. ¿Asientos recurrentes / periódicos automáticos?** — **NATIVO** · baja · `account/models/account_move.py` (`auto_post` recurrente, `_apply_delta_recurring_entries` :4286) · Asientos y facturas recurrentes son core CE; no requiere Enterprise.

**AACAP10. ¿Digest / resumen contable periódico por email?** — **EE** · baja · `account_accountant/models/digest.py::kpi_account_bank_cash` · KPI de tesorería en el digest; el motor digest base es core, la métrica contable la añade EE.

**AACAP11. ¿Sincronización bancaria online + OCR de extractos/facturas?** — **EE-hermano** · media · módulos aparte: `account_online_synchronization` (feeds bancarios), `account_invoice_extract` / `account_bank_statement_extract` (OCR IAP). No están en `account_accountant`; requieren esos módulos (+ créditos IAP).

**AACAP12. ¿Predicción/sugerencia de cuenta al conciliar o en apuntes?** — **NATIVO (conciliación) / NO-VERIFICADO (factura)** · media · la sugerencia de contrapartida al conciliar sale de `account.reconcile.model` (core). La predicción automática de cuenta/impuesto en líneas de **factura de proveedor** NO la localicé como feature nativa clara en fuente v18 → no prometer sin verificar en el proyecto.

## Multi-divisa

**AACAP13. ¿Multi-divisa con revalorización automática (unrealized gain/loss)?** — **NATIVO (básico) / EE-hermano (revaluación)** · media · tipos de cambio y diferencia realizada al cobrar/pagar son core; el **informe + asiento de revalorización de saldos** (no realizada) es EE en `account_reports/wizard/multicurrency_revaluation.py` (ver dominio `account_reports`).

---

## Frontera CE (`account`) ↔ EE (`account_accountant`) — para el consultor

- **Ya en CE (no vender como EE):** asientos manuales, plan contable, diarios, statements bancarios (`account.bank.statement`), modelos de conciliación (`account.reconcile.model`), asientos recurrentes, lock dates (campos), dashboard de diarios, multi-divisa básica.
- **Solo EE (`account_accountant`):** widget de conciliación bancaria, ejercicio fiscal (`account.fiscal.year`), auto-conciliación masiva (wizard+cron), asistente de lock date con excepciones, KPI contable en digest.
- **Solo EE, módulo hermano:** informes financieros (`account_reports` → dominio propio), sincronización bancaria online, OCR de facturas/extractos, activos (`account_asset`), presupuestos (`account_budget`), seguimiento de cobros (`account_followup`), SEPA/ISO20022, SAF-T.
