# Definiciones de KPIs - HR People Report

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-001-REF-001

---

## KPIs Principales (15+)

### 1. Headcount Total
- **Definicion**: Numero total de empleados activos a fecha de cierre del periodo
- **Formula**: `COUNT(empleados WHERE status = 'active')`
- **Fuente**: HiBob People API (`GET /people`)
- **Frecuencia**: Mensual (snapshot a ultimo dia del mes)

### 2. FTE (Full-Time Equivalent)
- **Definicion**: Equivalente a jornada completa. Un empleado a media jornada = 0.5 FTE
- **Formula**: `SUM(employment_percentage / 100)` para todos los empleados activos
- **Fuente**: HiBob People API (campo `work.employmentType` + `work.weeklyHours`)
- **Nota**: Si no hay dato de horas semanales, asumir FTE = 1.0 para contrato indefinido

### 3. Rotacion Voluntaria (%)
- **Definicion**: Porcentaje anualizado de bajas voluntarias (dimisiones)
- **Formula**: `(bajas_voluntarias_periodo / headcount_medio_periodo) * (12 / meses_periodo) * 100`
- **Fuente**: HiBob People API (empleados con `status = 'inactive'` y `lifecycle.terminationType = 'voluntary'`)
- **Umbral warning**: > 15% anual
- **Umbral critico**: > 25% anual

### 4. Rotacion Involuntaria (%)
- **Definicion**: Porcentaje anualizado de bajas involuntarias (despidos, no renovaciones, fin de contrato)
- **Formula**: `(bajas_involuntarias_periodo / headcount_medio_periodo) * (12 / meses_periodo) * 100`
- **Fuente**: HiBob People API (`lifecycle.terminationType = 'involuntary'` o `'end_of_contract'`)
- **Umbral warning**: > 5% anual
- **Umbral critico**: > 10% anual

### 5. Rotacion Total (%)
- **Definicion**: Porcentaje anualizado de todas las bajas
- **Formula**: `rotacion_voluntaria + rotacion_involuntaria`
- **Nota**: Si menos de 3 bajas en el periodo, marcar como "muestra insuficiente"

### 6. Antiguedad Media (meses)
- **Definicion**: Media de meses desde la fecha de incorporacion hasta hoy
- **Formula**: `AVG((fecha_actual - startDate).days / 30.44)` para todos los empleados activos
- **Fuente**: HiBob People API (campo `work.startDate`)
- **Referencia**: Antiguedad media saludable en tech: 24-48 meses

### 7. Nuevas Incorporaciones
- **Definicion**: Numero de empleados que se incorporaron durante el periodo
- **Formula**: `COUNT(empleados WHERE startDate >= inicio_periodo AND startDate <= fin_periodo)`
- **Fuente**: HiBob People API

### 8. Bajas
- **Definicion**: Numero de empleados que causaron baja durante el periodo
- **Formula**: `COUNT(empleados WHERE lifecycle.terminationDate >= inicio_periodo AND lifecycle.terminationDate <= fin_periodo)`
- **Fuente**: HiBob People API (empleados inactivos con fecha de baja en el periodo)

### 9. Time-to-Fill (dias)
- **Definicion**: Dias promedio desde la publicacion de una vacante hasta su cobertura
- **Formula**: `AVG(fecha_cierre_vacante - fecha_publicacion_vacante)` para vacantes cubiertas en el periodo
- **Fuente**: HiBob Workforce Planning (`GET /workforce-planning/positions`) + Odoo `hr.applicant`
- **Objetivo**: <= 30 dias
- **Umbral warning**: > 30 dias
- **Umbral critico**: > 60 dias

### 10. Distribucion por Departamento
- **Definicion**: Numero y porcentaje de empleados por departamento
- **Formula**: `GROUP BY department -> COUNT(*), COUNT(*)/total * 100`
- **Fuente**: HiBob People API (campo `work.department`)
- **Nota**: Departamentos con < 3 empleados se agrupan como "Otros" (RGPD)

### 11. Distribucion por Ubicacion
- **Definicion**: Numero y porcentaje de empleados por sede/ubicacion
- **Formula**: `GROUP BY site -> COUNT(*), COUNT(*)/total * 100`
- **Fuente**: HiBob People API (campo `work.site`)

### 12. Tasa de Ausencia (%)
- **Definicion**: Porcentaje de dias laborables perdidos por ausencias
- **Formula**: `(dias_ausencia_periodo / (dias_laborables_periodo * headcount)) * 100`
- **Fuente**: HiBob TimeOff API
- **Umbral warning**: > 5%
- **Umbral critico**: > 10%
- **Nota**: No incluir vacaciones planificadas. Solo ausencias no planificadas y bajas.

### 13. Posiciones Abiertas
- **Definicion**: Numero de vacantes activas sin cubrir
- **Formula**: `COUNT(positions WHERE status = 'open')`
- **Fuente**: HiBob Workforce Planning API
- **Detalle**: Desglosar por departamento y dias abierta

### 14. Tasa de Promocion (%)
- **Definicion**: Porcentaje de empleados que recibieron un cambio de puesto o nivel en el periodo
- **Formula**: `(empleados_promovidos / headcount_inicio_periodo) * 100`
- **Fuente**: HiBob People API (campo `lifecycle` con tipo `promotion` o cambio de `work.title`)
- **Referencia**: Tasa saludable: 5-15% anual

### 15. Movilidad Interna (%)
- **Definicion**: Porcentaje de empleados que cambiaron de departamento en el periodo
- **Formula**: `(cambios_departamento / headcount_inicio_periodo) * 100`
- **Fuente**: HiBob People API (historial de cambios en `work.department`)

### 16. Ratio Manager:Empleado
- **Definicion**: Numero medio de reports directos por manager
- **Formula**: `headcount_no_managers / count_managers`
- **Fuente**: HiBob People API (campo `work.isManager` o inferido de `work.reportsTo`)
- **Referencia**: Ratio saludable: 5-8 empleados por manager

---

## HR Health Score - Calculo

### Pesos por Dimension

| Dimension | Peso | Componentes |
|-----------|------|-------------|
| Retencion | 30% | Rotacion total, rotacion voluntaria, tendencia 3m |
| Staffing | 25% | Cobertura plantilla, time-to-fill, vacantes criticas |
| Engagement | 20% | Antiguedad media, promociones, movilidad interna |
| Compliance | 15% | Evaluaciones completadas, formaciones, documentacion |
| Diversidad | 10% | Distribucion genero, ubicacion, tipo contrato |

### Escala de Scoring

| Rango | Estado | Icono | Descripcion |
|-------|--------|-------|-------------|
| 80-100 | Excelente | Verde | Metricas saludables en todas las dimensiones |
| 60-79 | Bueno | Amarillo | Algunas areas requieren atencion |
| 40-59 | Regular | Naranja | Multiples areas con problemas |
| 20-39 | Critico | Rojo | Intervencion urgente necesaria |
| 0-19 | Emergencia | Rojo oscuro | Situacion critica generalizada |

### Penalizaciones

El score base de cada dimension (0-100) se reduce por:
- Rotacion > umbral warning: -10 puntos en Retencion
- Rotacion > umbral critico: -25 puntos en Retencion
- Vacantes > 60 dias: -5 puntos por vacante en Staffing
- Time-to-fill > 30 dias: -10 puntos en Staffing
- Ausencia > 5%: -10 puntos en Engagement
- Evaluaciones < 95% completadas: -15 puntos en Compliance

---

## Notas de Implementacion

- Todos los KPIs se calculan en `scripts/calculate_hr_health.py`, nunca manualmente
- Los umbrales se definen en `lib/config.py` y pueden ajustarse
- Si un dato no esta disponible, el KPI se marca como "N/A" y no penaliza el score
- La tendencia se calcula comparando con los 2 meses anteriores (si hay datos)
- Los porcentajes se anualizan cuando el periodo es menor a 12 meses
