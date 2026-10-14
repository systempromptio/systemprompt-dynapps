# Informe Mensual de Recursos Humanos - {MES} {ANO}

**CONFIDENTIAL - HR TEAM ONLY**
**Generado:** {fecha_generacion}
**Periodo:** {inicio_periodo} - {fin_periodo}
**Fuentes:** HiBob + Odoo

---

## 1. Dashboard HR

### HR Health Score

| Dimension | Score | Peso | Contribucion |
|-----------|-------|------|-------------|
| Retencion | {retention_score}/100 | 30% | {retention_contribution} |
| Staffing | {staffing_score}/100 | 25% | {staffing_contribution} |
| Engagement | {engagement_score}/100 | 20% | {engagement_contribution} |
| Compliance | {compliance_score}/100 | 15% | {compliance_contribution} |
| Diversidad | {diversity_score}/100 | 10% | {diversity_contribution} |
| **TOTAL** | **{hr_health_score}/100** | **100%** | **{estado}** |

### KPIs Principales

| KPI | Valor Actual | Mes Anterior | Tendencia |
|-----|-------------|-------------|-----------|
| Headcount | {headcount} | {headcount_prev} | {tendencia_headcount} |
| FTE | {fte} | {fte_prev} | {tendencia_fte} |
| Nuevas altas | {new_hires} | {new_hires_prev} | {tendencia_altas} |
| Bajas | {departures} | {departures_prev} | {tendencia_bajas} |
| Rotacion total (anual.) | {turnover_total}% | {turnover_prev}% | {tendencia_rotacion} |
| Antiguedad media | {avg_tenure}m | {avg_tenure_prev}m | {tendencia_antiguedad} |
| Posiciones abiertas | {open_positions} | {open_positions_prev} | {tendencia_vacantes} |
| Tasa de ausencia | {absence_rate}% | {absence_rate_prev}% | {tendencia_ausencia} |

### Alertas Activas

{alertas_lista}

**Escenario diagnosticado:** {scenario_code} - {scenario_description}

---

## 2. Headcount y Organizacion

### Distribucion por Departamento

| Departamento | Headcount | FTE | % del Total | Variacion |
|-------------|-----------|-----|-------------|-----------|
| {dept_1} | {count_1} | {fte_1} | {pct_1}% | {var_1} |
| {dept_2} | {count_2} | {fte_2} | {pct_2}% | {var_2} |
| ... | ... | ... | ... | ... |
| **Total** | **{total}** | **{total_fte}** | **100%** | **{var_total}** |

### Distribucion por Ubicacion

| Ubicacion | Headcount | % del Total |
|-----------|-----------|-------------|
| {location_1} | {count_1} | {pct_1}% |
| {location_2} | {count_2} | {pct_2}% |
| ... | ... | ... |

### Estructura de Gestion

- Total managers: {total_managers}
- Total contribuidores individuales: {total_ic}
- Ratio manager:empleado: **{ratio}:1**

---

## 3. Analisis de Rotacion

### Resumen

| Metrica | Periodo Actual | Anualizado | Tendencia 3m |
|---------|---------------|------------|-------------|
| Rotacion voluntaria | {vol_count} bajas | {vol_pct}% | {vol_trend} |
| Rotacion involuntaria | {invol_count} bajas | {invol_pct}% | {invol_trend} |
| **Rotacion total** | **{total_count} bajas** | **{total_pct}%** | **{total_trend}** |

### Rotacion por Departamento

| Departamento | Bajas | Rotacion Anual. | Estado |
|-------------|-------|-----------------|--------|
| {dept_1} | {count_1} | {pct_1}% | {estado_1} |
| ... | ... | ... | ... |

### Motivos de Baja (si disponibles)

| Motivo | Conteo | % |
|--------|--------|---|
| {motivo_1} | {count_1} | {pct_1}% |
| ... | ... | ... |

> **Nota**: Los motivos de baja se obtienen del campo `lifecycle.terminationReason` en HiBob.
> Si el campo no esta cumplimentado, se muestra como "No especificado".

---

## 4. Antiguedad y Seniority

### Distribucion por Bandas de Antiguedad

| Banda | Empleados | % del Total |
|-------|-----------|-------------|
| < 1 ano | {band_1} | {pct_1}% |
| 1 - 3 anos | {band_2} | {pct_2}% |
| 3 - 5 anos | {band_3} | {pct_3}% |
| 5 - 10 anos | {band_4} | {pct_4}% |
| > 10 anos | {band_5} | {pct_5}% |

### Antiguedad Media por Departamento

| Departamento | Antiguedad Media | vs Media Global |
|-------------|-----------------|-----------------|
| {dept_1} | {tenure_1}m | {vs_avg_1} |
| ... | ... | ... |

### Empleados en Periodo de Prueba

- Total en periodo de prueba (< 6 meses): **{probation_count}**
- Departamento con mas incorporaciones recientes: **{dept_mas_reciente}**

---

## 5. Staffing

### Posiciones Abiertas

| Departamento | Vacantes | Dias Abierta (avg) | Estado |
|-------------|----------|-------------------|--------|
| {dept_1} | {count_1} | {days_1}d | {estado_1} |
| ... | ... | ... | ... |
| **Total** | **{total_vacantes}** | **{avg_days}d** | |

### Time-to-Fill

- Promedio del periodo: **{avg_ttf} dias** (objetivo: 30 dias)
- Vacantes criticas (> 60 dias): **{critical_count}**

{if_target_headcount}
### Gap de Plantilla

- Headcount actual: {headcount}
- Objetivo: {target}
- Gap: **{gap}** ({gap_pct}%)
{/if_target_headcount}

---

## 6. Resumen de Ausencias

### Tasa de Ausencia Global

- Dias laborables del periodo: {working_days}
- Dias de ausencia totales: {absence_days}
- **Tasa de ausencia: {absence_rate}%**

### Ausencias por Tipo

| Tipo | Dias | % del Total |
|------|------|-------------|
| Vacaciones | {vac_days} | {vac_pct}% |
| Asuntos propios | {personal_days} | {personal_pct}% |
| Otros | {other_days} | {other_pct}% |

### Ausencias por Departamento

| Departamento | Dias Ausencia | Tasa | Estado |
|-------------|--------------|------|--------|
| {dept_1} | {days_1} | {rate_1}% | {estado_1} |
| ... | ... | ... | ... |

> **Nota RGPD**: No se desglosan bajas medicas por privacidad. Se agrupan como "Otros".

---

## 7. Detalle por Departamento

{for_each_department}
### {nombre_departamento}

| Metrica | Valor |
|---------|-------|
| Headcount | {dept_headcount} |
| FTE | {dept_fte} |
| Nuevas altas (mes) | {dept_new_hires} |
| Bajas (mes) | {dept_departures} |
| Rotacion anualizada | {dept_turnover}% |
| Antiguedad media | {dept_tenure}m |
| Posiciones abiertas | {dept_vacancies} |
| Tasa de ausencia | {dept_absence}% |

**Alertas:** {dept_alerts}

{/for_each_department}

---

## 8. Plan de Accion

### Escenario: {scenario_code} - {scenario_description}

### Acciones Inmediatas (esta semana)

1. {accion_inmediata_1}
2. {accion_inmediata_2}
3. {accion_inmediata_3}

### Acciones a Corto Plazo (proximas 2-4 semanas)

1. {accion_corto_1}
2. {accion_corto_2}
3. {accion_corto_3}

### Acciones Estrategicas (proximo trimestre)

1. {accion_estrategica_1}
2. {accion_estrategica_2}
3. {accion_estrategica_3}

---

*Informe generado automaticamente por HR People Report (ai-claude-hr)*
*Datos extraidos de HiBob + Odoo. Metricas calculadas por `calculate_hr_health.py`.*
*Contacto: rrhh@indaws.es*
