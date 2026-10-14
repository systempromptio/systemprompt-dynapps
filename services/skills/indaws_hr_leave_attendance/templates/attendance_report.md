# Informe de Anomalias de Asistencia - {MES} {ANO}

**CONFIDENTIAL - HR TEAM ONLY**
**Generado:** {fecha_generacion}
**Periodo:** {inicio_periodo} - {fin_periodo}
**Fuentes:** HiBob Attendance + Odoo hr.attendance

---

## 1. Resumen de Anomalias

| Tipo de Anomalia | Incidencias | Empleados Afectados | % Plantilla |
|------------------|-------------|--------------------|-----------
| Llegadas tarde (> 15 min) | {late_count} | {late_emp} | {late_pct}% |
| Salidas anticipadas (> 30 min) | {early_count} | {early_emp} | {early_pct}% |
| Fichajes omitidos | {missed_count} | {missed_emp} | {missed_pct}% |
| Horas extras excesivas (> 10h/sem) | {overtime_count} | {overtime_emp} | {overtime_pct}% |
| **Total anomalias** | **{total_anomalies}** | **{total_emp_affected}** | **{total_pct}%** |

**Estado general del periodo:** {estado_general}

### Comparativa con Mes Anterior

| Metrica | Mes Actual | Mes Anterior | Variacion |
|---------|-----------|-------------|-----------|
| Total anomalias | {total_current} | {total_prev} | {variation} |
| Empleados afectados | {emp_current} | {emp_prev} | {emp_variation} |
| Horas extras totales | {overtime_hours}h | {overtime_prev}h | {ot_variation} |

---

## 2. Llegadas Tarde por Departamento

| Departamento | Incidencias | Empleados | Media Retraso | Estado |
|-------------|-------------|-----------|--------------|--------|
| {dept_1} | {count_1} | {emp_1} | {avg_1} min | {estado_1} |
| {dept_2} | {count_2} | {emp_2} | {avg_2} min | {estado_2} |
| ... | ... | ... | ... | ... |
| **Total** | **{total}** | **{total_emp}** | **{total_avg} min** | |

### Distribucion por Franja Horaria

| Franja | Incidencias | % |
|--------|-------------|---|
| 15-30 min de retraso | {franja_1} | {pct_1}% |
| 30-60 min de retraso | {franja_2} | {pct_2}% |
| > 60 min de retraso | {franja_3} | {pct_3}% |

---

## 3. Fichajes Omitidos

| Departamento | Sin Fichaje Entrada | Sin Fichaje Salida | Total |
|-------------|--------------------|--------------------|-------|
| {dept_1} | {no_entry_1} | {no_exit_1} | {total_1} |
| {dept_2} | {no_entry_2} | {no_exit_2} | {total_2} |
| ... | ... | ... | ... |
| **Total** | **{total_no_entry}** | **{total_no_exit}** | **{grand_total}** |

**Nota**: Los fichajes omitidos pueden deberse a:
- Olvido del empleado (causa mas frecuente)
- Problemas tecnicos con el sistema de fichaje
- Trabajo remoto sin acceso al sistema de presencia
- Visitas a cliente o eventos externos

---

## 4. Horas Extras

### Resumen Global

| Metrica | Valor |
|---------|-------|
| Total horas extras del periodo | {total_overtime_hours}h |
| Media por empleado con horas extras | {avg_per_employee}h |
| Empleados con horas extras | {emp_with_overtime} |
| Empleados que superan 10h/semana | {over_limit_count} |
| Empleados que superan 80h/mes | {over_legal_count} |

### Horas Extras por Departamento

| Departamento | Horas Extras | Empleados | Media/Empleado | Alerta Legal |
|-------------|-------------|-----------|----------------|-------------|
| {dept_1} | {hours_1}h | {emp_1} | {avg_1}h | {legal_1} |
| {dept_2} | {hours_2}h | {emp_2} | {avg_2}h | {legal_2} |
| ... | ... | ... | ... | ... |

> **ALERTA LEGAL**: El Estatuto de los Trabajadores (Art. 35.2 ET) limita las horas
> extraordinarias a **80 horas anuales**. Los empleados marcados con alerta deben ser
> revisados inmediatamente por RRHH y su manager directo.

### Compensacion de Horas Extras

| Metodo | Empleados | Horas |
|--------|-----------|-------|
| Compensacion con descanso | {comp_rest_emp} | {comp_rest_hours}h |
| Retribucion economica | {comp_pay_emp} | {comp_pay_hours}h |
| Pendiente de definir | {comp_pending_emp} | {comp_pending_hours}h |

> **Nota**: La compensacion con descanso debe realizarse dentro de los 4 meses
> siguientes a la realizacion de las horas extras (Art. 35.1 ET).

---

## 5. Patrones Detectados

### Patrones Recurrentes

{patrones_lista}

Ejemplos de patrones detectados automaticamente:
- Llegadas tarde sistematicas los lunes (> 3 lunes consecutivos)
- Fichajes omitidos en viernes (posible salida anticipada sin registro)
- Picos de horas extras antes de entregas de proyecto
- Ausencias frecuentes de corta duracion en dias especificos

### Correlacion con Ausencias

| Patron | Frecuencia | Departamento Principal | Impacto |
|--------|-----------|----------------------|---------|
| {patron_1} | {freq_1} | {dept_1} | {impact_1} |
| {patron_2} | {freq_2} | {dept_2} | {impact_2} |
| ... | ... | ... | ... |

---

## 6. Recomendaciones

### Acciones Inmediatas

1. {recomendacion_inmediata_1}
2. {recomendacion_inmediata_2}
3. {recomendacion_inmediata_3}

### Acciones a Medio Plazo

1. {recomendacion_medio_1}
2. {recomendacion_medio_2}

### Mejoras del Sistema de Fichaje

1. {mejora_sistema_1}
2. {mejora_sistema_2}

---

## Cumplimiento del Registro de Jornada (RDL 8/2019)

| Metrica | Valor | Objetivo | Estado |
|---------|-------|----------|--------|
| Dias con registro completo | {complete_days} | 100% | {estado_complete} |
| Empleados con 100% fichajes | {full_compliance_emp} | {total_emp} | {compliance_pct}% |
| Dias sin ningun registro | {no_record_days} | 0 | {estado_no_record} |

> **Recordatorio legal**: El registro de jornada es obligatorio desde mayo 2019.
> Los registros deben conservarse durante 4 anos y estar disponibles para la
> Inspeccion de Trabajo, representantes legales de los trabajadores y los propios empleados.

---

*Informe generado automaticamente por HR Leave & Attendance (ai-claude-hr)*
*Datos extraidos de HiBob Attendance + Odoo hr.attendance. Anomalias detectadas por `anomaly_detector.py`.*
*Contacto: rrhh@indaws.es*
