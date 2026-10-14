# Informe Mensual de Ausencias - {MES} {ANO}

**CONFIDENTIAL - HR TEAM ONLY**
**Generado:** {fecha_generacion}
**Periodo:** {inicio_periodo} - {fin_periodo}
**Fuentes:** HiBob TimeOff + Odoo hr.leave

---

## 1. Resumen Ejecutivo

| Metrica | Valor | Mes Anterior | Tendencia |
|---------|-------|-------------|-----------|
| Dias laborables del periodo | {working_days} | {working_days_prev} | - |
| Total dias de ausencia | {total_absence_days} | {total_prev} | {tendencia_total} |
| Tasa de absentismo | {absence_rate}% | {rate_prev}% | {tendencia_rate} |
| Empleados con ausencia | {employees_with_absence} | {emp_prev} | {tendencia_emp} |
| Bradford Factor medio | {avg_bf} | {avg_bf_prev} | {tendencia_bf} |
| Alertas activas | {alert_count} | {alert_prev} | {tendencia_alerts} |

**Estado general:** {estado_global} ({icono})

---

## 2. Ausencias por Tipo

| Tipo de Ausencia | Dias | % del Total | Empleados |
|------------------|------|-------------|-----------|
| Vacaciones | {vac_days} | {vac_pct}% | {vac_count} |
| Asuntos propios | {personal_days} | {personal_pct}% | {personal_count} |
| Permisos retribuidos | {permisos_days} | {permisos_pct}% | {permisos_count} |
| Otros | {other_days} | {other_pct}% | {other_count} |
| **Total** | **{total_days}** | **100%** | **{total_emp}** |

> **Nota RGPD**: Las bajas por incapacidad temporal se agrupan en "Otros" para proteger
> la privacidad de los empleados. No se desglosa el tipo de baja medica.

---

## 3. Ausencias por Departamento

| Departamento | Dias Ausencia | Headcount | Tasa Absentismo | Estado |
|-------------|--------------|-----------|-----------------|--------|
| {dept_1} | {days_1} | {hc_1} | {rate_1}% | {estado_1} |
| {dept_2} | {days_2} | {hc_2} | {rate_2}% | {estado_2} |
| ... | ... | ... | ... | ... |
| **Total** | **{total_days}** | **{total_hc}** | **{total_rate}%** | |

**Umbrales:**
- Normal: < 5%
- Warning: 5% - 10%
- Critico: > 10%

---

## 4. Saldos de Vacaciones (Agregado por Departamento)

| Departamento | Dias Totales | Consumidos | Disponibles | % Consumido |
|-------------|-------------|------------|-------------|-------------|
| {dept_1} | {total_1} | {used_1} | {avail_1} | {pct_1}% |
| {dept_2} | {total_2} | {used_2} | {avail_2} | {pct_2}% |
| ... | ... | ... | ... | ... |
| **Total** | **{grand_total}** | **{grand_used}** | **{grand_avail}** | **{grand_pct}%** |

**Alertas de saldos:**
- Empleados con > 80% de vacaciones sin consumir a fecha actual: {high_balance_count}
- Recomendacion: planificar consumo antes de fin de ano para evitar acumulacion

---

## 5. Bradford Factor - Alertas

### Top 10 por Bradford Factor (ultimos 12 meses)

| # | Departamento | BF Score | Episodios (S) | Dias (D) | Estado |
|---|-------------|----------|---------------|----------|--------|
| 1 | {dept_1} | {bf_1} | {s_1} | {d_1} | {estado_1} |
| 2 | {dept_2} | {bf_2} | {s_2} | {d_2} | {estado_2} |
| ... | ... | ... | ... | ... | ... |

> **Nota**: Se muestra el departamento para proteger la identidad individual.
> El detalle por empleado esta disponible para RRHH y managers directos bajo solicitud.

### Resumen de Estados

| Estado | Empleados | % Plantilla |
|--------|-----------|-------------|
| Normal (BF < 200) | {normal_count} | {normal_pct}% |
| Warning (200-500) | {warning_count} | {warning_pct}% |
| Alto (500-1000) | {alto_count} | {alto_pct}% |
| Critico (> 1000) | {critico_count} | {critico_pct}% |

---

## 6. Cumplimiento de Politicas

### Asuntos Propios (2 dias/ano - Convenio TIC)

| Metrica | Valor |
|---------|-------|
| Empleados que han usado los 2 dias | {all_used} |
| Empleados con 1 dia disponible | {one_left} |
| Empleados con 2 dias disponibles | {both_avail} |

### Alertas de Cumplimiento

{alertas_cumplimiento_lista}

---

## 7. Tendencia (3 meses)

| Mes | Tasa Absentismo | Dias Ausencia | BF Medio |
|-----|----------------|---------------|----------|
| {mes_3} | {rate_3}% | {days_3} | {bf_3} |
| {mes_2} | {rate_2}% | {days_2} | {bf_2} |
| {mes_1} (actual) | {rate_1}% | {days_1} | {bf_1} |

**Tendencia:** {descripcion_tendencia}

---

*Informe generado automaticamente por HR Leave & Attendance (ai-claude-hr)*
*Datos extraidos de HiBob TimeOff + Odoo hr.leave. Metricas calculadas por `anomaly_detector.py`.*
*Contacto: rrhh@indaws.es*
