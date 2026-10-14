<!-- verified: v18.0 @ 2026-07 (OCA mis-builder 18.0.1.8.3 — mis_builder) -->
# OCA mis_builder — Guía de configuración

## 1. Montar un informe (plantilla → instancia)

1. **`mis.report`** (plantilla): define los KPIs. Cada KPI tiene un `name` técnico (el que referencias
   en otras expresiones — ¡no la etiqueta!, MISG8) y una expresión contable (MISG1/MISG2).
2. **`mis.report.instance`**: elige la plantilla, define periodos/columnas, `target_move`, compañías,
   analítica. Es lo que se ejecuta y se ve.

## 2. Expresiones — recetas (PGC español como ejemplo)

- Ingresos del periodo: `balp[7%]` (o `bal[7%]`) — recuerda: negativo (haber), usa `-bal[7%]` para verlo
  positivo, o `nbal`.
- Gastos del periodo: `bal[6%]`.
- Resultado: `-bal[6%,7%]` o `ingresos - gastos` referenciando KPIs por su `name`.
- Tesorería a fin de periodo: `bale[57%]` (cuenta de balance → acumula histórico, MISG3).
- Resultado ejercicios anteriores: `balu[]` (modo `u`, MISG4).
- Ratio con protección de /0 (MISG7): NO uses `a / b` a pelo; comprueba el denominador o usa estilo
  condicional para no mostrar `#DIV/0`.

## 3. Estado de asientos (posted vs all)

`target_move` en la instancia: **"All Posted Entries" (default)** vs "All Entries" (incluye borradores) —
MISG9. Para conciliar con la contabilidad oficial, deja "posted".

## 4. Periodos y comparación

Cada `mis.report.instance.period` tiene `mode`: `fix` (fechas fijas), `relative` (relativo a la fecha
base del informe: día/semana/mes/año/date_range, con `offset`/`duration`), `none` (sin filtro de fecha).
`accumulation_method` del KPI decide cómo encaja un KPI de otra duración en la columna (SUM/AVG/NONE,
pro-rata temporis — MISG11: saldos→AVG/NONE, flujos→SUM).

## 5. Analítica

`analytic_domain` (Text con un dominio literal) a nivel instancia y/o por columna (MISG10). En v18 filtra
sobre `analytic_distribution` (JSON) — escribe el dominio acorde al modelo v18, no al m2o antiguo.

## 6. Columnas derivadas

- **sumcol**: suma varias columnas (desviaciones agregadas) — no consulta BD.
- **cmpcol**: compara dos columnas (from/to) — real vs presupuesto.
- **actuals_alt**: misma estructura de KPIs pero contra otro modelo de aml (budget/cash-flow) — el modelo
  debe tener los campos usados (MISG14).

## 7. Rendimiento

- `no_auto_expand_accounts` en la instancia si no necesitas el desglose por cuenta (MISG15).
- El AEP ya minimiza queries (1 por dominio+modo). El coste sube con nº de columnas × nº de modos y con
  `smart_end` (2 queries por saldo final, MISG17) y con auto-expand.

## 8. Familia de módulos

`mis_builder` (core) · `mis_builder_budget` (presupuesto) · `mis_builder_cash_flow` (flujo de caja) ·
`mis_builder_demo` (informes de ejemplo — NO instalar en producción).
