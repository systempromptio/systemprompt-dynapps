# Post-mortem + KPIs

Objetivo: medir cómo fue la migración frente a lo presupuestado en **Fase 1**
(`migration-estimator`) y dejar aprendizaje para calibrar futuras estimaciones.

## Datos de entrada
- **Estimado Fase 1**: total de horas del informe / qty del sale.order original.
  Idealmente con el desglose en 4 buckets (Code, DB, Unit Testing, PM).
- **Horas reales**: timesheets del proyecto vía `odoo-pilot`. Modelo de partes
  `project.project.hour.line` (usa campo `date`, NO `date_from`/`date_to`).
  Excluir `holiday_id` / `global_leave_id`. Excluir CTOs: David Gómez Quilón (194),
  Jesús Vargas Sánchez (193). Incluir `context: {lang: es_ES}` en las llamadas.
- **Gates**: estado por gate de la migración (informe → entrega → producción).
- **Retrabajos**: tareas reabiertas, correcciones tras reporte de Dynapps.

## KPIs a calcular
| KPI | Fórmula / definición |
|---|---|
| Desvío total | `(horas_reales − estimado) / estimado × 100` (%) |
| Desvío por bucket | mismo cálculo Code / DB / Unit Testing / PM por separado |
| Gates fallados | nº de gates que requirieron una segunda pasada |
| Retrabajos | nº de módulos/tareas reabiertas tras entrega |
| Módulos sobre cap | módulos que superaron el cap ~20h (con motivo) |
| Coste real vs presupuesto | horas_reales × 45 € vs estimado × 45 € |

## Lectura del desvío (guardarraíles de Fase 1)
- Rango sano de estimación era **100-150h**; **>200h** era bandera roja.
- Si el **real** se disparó frente al estimado, identificar la causa (saltos de
  versión grandes, conectores, OCA sin port v19 que volvió a scope, drift de
  módulos parcheados localmente) y anotarla para recalibrar el estimador.
- Si el real fue **muy inferior** al estimado, también es señal: la calibración IA
  puede afinarse más agresiva.

## Salida
Post-mortem en markdown: tabla de KPIs, desvío total y por bucket, lista de causas
del desvío, y 1-3 recomendaciones de calibración para `migration-estimator`.
Formato de horas siempre **HH:MM**, nunca decimales.
