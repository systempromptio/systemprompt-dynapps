---
name: indaws-demo-contabilidad-auditoria
description: >
  Auditoría de cierre contable sobre un trimestre (por defecto el último
  vencido). Ejecuta 9 chequeos sobre account.move, account.move.line,
  account.bank.statement.line, res.partner y res.company: asientos sin
  validar, descuadres de IVA, cuentas contables sin 6 dígitos, facturas
  intracomunitarias con IVA aplicado, asientos manuales sin referencia,
  amortizaciones del período, partners sin VAT en facturas, líneas bancarias
  pendientes de conciliar y verificación de que los periodos anteriores
  están cerrados (fiscalyear_lock_date). Output: informe HTML con estado
  verde/rojo por check y detalle de registros afectados. No genera modelo 303.
  Usar cuando el usuario pida: auditoría de cierre, revisión contable, checklist
  de cierre, verificación fiscal previa al cierre.
version: 1.0.2
model: sonnet
effort: medium
---

# Demo · Contabilidad auditoría

Checklist de 9 puntos sobre el trimestre contable. Solo informe HTML.

## Prerequisitos
- **odoo-pilot** para `account.move`, `account.move.line`, `account.bank.statement.line`, `res.company`, `res.partner`, `account.account`, `account.journal`
- Python 3.8+ stdlib

## Workflow

| Paso | Acción | Output |
|---|---|---|
| 1 | Autenticar. Leer `config/config.json`. | -- |
| 2 | Ejecutar `python scripts/main.py` → analiza último trimestre vencido. Flags: `--quarter 2026-Q1`, `--from YYYY-MM-DD --to YYYY-MM-DD`. Ejecuta los 9 chequeos, guarda `analysis.json` y genera el informe HTML final directamente (no hay fase narrative). | analysis.json + report.html |

No hay fase `prepare + finalize`: el informe se genera en un solo paso con texto estático descriptivo por check.

## Los 9 chequeos

| # | Check | Criterio |
|---|---|---|
| 1 | **Asientos sin validar** | `account.move state='draft'` con `date` en el período |
| 2 | **Descuadres de IVA** | para cada factura, comparar `amount_tax` vs suma de `tax_line_ids.amount_currency`. Diff > `iva_mismatch_threshold_eur` → incidencia |
| 3 | **Cuentas contables sin 6 dígitos** | `account.account` usadas en el período con `len(code) != 6` |
| 4 | **Facturas intracomunitarias con IVA aplicado** | facturas del período a partners con `country_id.code` en `eu_country_codes` (ex-España) Y `amount_tax > 0` |
| 5 | **Asientos manuales sin referencia** | `account.move state='posted'` en journals tipo `general` con `ref` vacío en el período |
| 6 | **Amortizaciones del período** | `account.move.line` en cuentas cuyo código empieza por `68` o `28` en el período (presentar lista, no incidencia per se) |
| 7 | **Partners sin VAT en facturas** | facturas del período cuyo partner tiene `vat = False` |
| 8 | **Líneas de banco pendientes de conciliar** | `account.bank.statement.line` del período con `is_reconciled = False` |
| 9 | **Periodos anteriores cerrados** | `res.company.fiscalyear_lock_date >= último día del trimestre previo al auditado` |

## Período default
- **Último trimestre vencido**. Ej: si hoy es 2026-04-23 → Q1 2026 (enero-marzo). Si hoy fuera 2026-02-05 → Q4 2025.

## Output visual

1. Cabecera: trimestre auditado · estado global (todo OK / hay incidencias)
2. **Checklist resumen**: tabla 9 filas con nombre + estado (🟢/🔴) + nº registros afectados
3. **Detalle por check**: secciones expandibles con lista de registros afectados (tablas con id, fecha, importe, etc.)
4. **Recomendaciones** (textos estáticos por cada check con incidencia)

## RULES

1. **Período default = último trimestre vencido**, calculado en tiempo de ejecución
2. Amortizaciones no son "incidencias" — solo listar como **informativo** (el contable quiere ver que se han contabilizado)
3. Cuentas con letras en el código (`TMP121000`) siempre son alerta — no solo por longitud, también por falta de dígitos limpios
4. Para descuadre IVA, tolerancia de 0,05 € — ajustes de redondeo aceptables
5. Un partner sin VAT en **factura simplificada** puede ser válido (particular) — aun así, alertar y recomendar revisión caso por caso

## References
| File | Content |
|---|---|
| `config/config.json` | Umbrales + códigos UE + prefijos de cuentas |
