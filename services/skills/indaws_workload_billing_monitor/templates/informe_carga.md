# Informe Carga de Trabajo vs Facturación — {mes_nombre} {año}

**Fecha de generación**: {fecha_generacion}
**Período analizado**: {date_start} — {date_end}
**Departamentos**: Consultoría, Desarrollo, Marketing externo

---

## 1. Resumen Ejecutivo

| Indicador | Valor |
|-----------|-------|
| Empleados analizados | {total_empleados} |
| Proyectos con planificación | {total_proyectos} |
| Proyectos excluidos (internos) | {proyectos_excluidos} |
| Horas totales planificadas | {total_horas}h |
| Facturación objetivo (×75€/h) | {facturacion_objetivo} € |
| Facturación real (hitos) | {facturacion_real} € |
| **Gap de facturación** | **{gap_total} €** |

### Distribución de Alertas

| Estado | Cantidad | % |
|--------|----------|---|
| 🔴 Sin facturación | {n_rojas} | {pct_rojas}% |
| 🟡 Facturación insuficiente | {n_amarillas} | {pct_amarillas}% |
| 🟢 OK | {n_verdes} | {pct_verdes}% |
| ⚪ Excluidos | {n_excluidos} | - |

---

## 2. 🔴 Alertas Rojas — Proyectos Sin Facturación

Proyectos con horas planificadas pero **sin ningún hito de facturación** en el mes.

| Proyecto | Fecha Inicio | Fecha Fin | Horas Plan | Empleados | Fact. Objetivo | Gap |
|----------|--------------|-----------|-----------|-----------|---------------|-----|
{filas_rojas}

---

## 3. 🟡 Alertas Amarillas — Facturación Insuficiente

Proyectos con facturación por debajo de las horas planificadas (cobertura < 100%).

| Proyecto | Fecha Inicio | Fecha Fin | Horas Plan | Fact. Objetivo | Fact. Real | Cobertura | Gap |
|----------|--------------|-----------|-----------|---------------|-----------|-----------|-----|
{filas_amarillas}

---

## 4. 🟢 Proyectos OK

Proyectos con facturación que cubre las horas planificadas.

| Proyecto | Fecha Inicio | Fecha Fin | Horas Plan | Fact. Objetivo | Fact. Real | Cobertura |
|----------|--------------|-----------|-----------|---------------|-----------|-----------|
{filas_verdes}

---

## 5. 🔵 Proyectos con Facturación Sin Horas Planificadas

Proyectos que tienen hitos de facturación en el mes pero **no aparecen en la planificación** de ningún empleado.

| Proyecto | Fecha Inicio | Fecha Fin | Hitos | Facturación |
|----------|--------------|-----------|-------|-------------|
{filas_huerfanos}

> Total facturación sin planificación: **{total_huerfanos} €**

---

## 6. Vista por Empleado

{seccion_empleados}

---

## 7. Recomendaciones

{recomendaciones}

---

*Generado automáticamente por workload-billing-monitor v1.0.0*
*Precio de referencia: 75 €/hora*
