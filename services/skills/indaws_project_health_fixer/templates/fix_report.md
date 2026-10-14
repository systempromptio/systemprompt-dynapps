# Template: Informe de Correccion

> Usa este template para generar el informe final de correcciones ejecutadas. Reemplaza los placeholders `{...}` con los datos reales.

---

```markdown
# Informe de Correccion: {proyecto}

**Fecha**: {fecha}
**Consultor**: {consultor}
**Score antes**: {score_antes}/100 ({estado_antes})
**Score proyectado**: {score_despues}/100 ({estado_despues})

---

## 1. Diagnostico Inicial

**Escenario detectado**: {escenario} - {descripcion_escenario}

| Categoria | Issues | Penalizacion recuperable |
|-----------|--------|--------------------------|
| Gobernanza | {governance_fixable} problemas | {governance_penalty}pts |
| Progreso | {progress_issues} problemas | N/A (requiere decision) |
| Estrategico | {strategic_issues} problemas | N/A (requiere replanteo) |

{resumen_diagnostico_adicional}

---

## 2. Correcciones de Gobernanza Ejecutadas

### 2.1 Fechas Asignadas

| # | Tarea | ID | Deadline Asignado | Resultado |
|---|-------|----|-------------------|-----------|
| 1 | {tarea_nombre} | {task_id} | {fecha_dd_mm_yyyy} | {ok_o_error} |

> Si no hubo tareas sin fecha, indicar: "No se detectaron tareas sin fecha limite."

### 2.2 Horas Redistribuidas

| # | Tarea | ID | Horas Antes | Horas Despues | Razon | Resultado |
|---|-------|----|-------------|---------------|-------|-----------|
| 1 | {tarea_nombre} | {task_id} | {antes}h | {despues}h | {razon} | {ok_o_error} |

> Si no hubo desajuste, indicar: "No se detecto desajuste de horas significativo."

### 2.3 Checklists Creados

| # | Tarea | Item Creado | Peso | Fecha Fin | Resultado |
|---|-------|-------------|------|-----------|-----------|
| 1 | {tarea_nombre} | {item_nombre} | {peso} | {fecha} | {ok_o_error} |

> Si no se crearon checklists, indicar: "No se detectaron tareas sin checklists."

**Resumen ejecucion**: {n_exitosas} de {n_total} operaciones exitosas.

---

## 3. Decisiones de Progreso

### 3.1 Tareas Vencidas

| # | Tarea | ID | Dias Retraso | Decision | Nueva Fecha / Accion | Resultado |
|---|-------|----|-------------|----------|---------------------|-----------|
| 1 | {tarea_nombre} | {task_id} | {dias} | {reprogramar/cerrar/cancelar} | {detalle} | {ok_o_error} |

### 3.2 Items de Checklist Vencidos

| # | Item | Tarea | Decision | Detalle | Resultado |
|---|------|-------|----------|---------|-----------|
| 1 | {item_nombre} | {tarea} | {completar/reprogramar} | {detalle} | {ok_o_error} |

### 3.3 Bottlenecks Atendidos

| # | Tarea | Risk Score | Accion Tomada |
|---|-------|------------|---------------|
| 1 | {tarea_nombre} | {risk_score} | {accion} |

> Si no hubo problemas de progreso, indicar: "No se identificaron problemas de progreso."

---

## 4. Plan de Desviacion

> Seccion condicional. Si no se activo la evaluacion de desviacion, indicar:
> "No se requiere plan de desviacion. El proyecto puede reconducirse con las correcciones ejecutadas."

### Opcion Elegida: {opcion_a_b_c}

| Aspecto | Antes | Despues |
|---------|-------|---------|
| Fecha fin | {fecha_fin_antes} | {fecha_fin_despues} |
| Tareas en scope | {tareas_antes} | {tareas_despues} |
| Score proyectado | {score_antes}/100 | {score_despues}/100 |

### Tareas Aplazadas a Fase 2 (si aplica)

| # | Tarea | Horas | Razon de aplazamiento |
|---|-------|-------|----------------------|
| 1 | {tarea_nombre} | {horas}h | {razon} |

### Nuevo Cronograma (si aplica)

| Tarea | Fecha Anterior | Fecha Nueva |
|-------|----------------|-------------|
| {tarea} | {antes} | {despues} |

---

## 5. Comparacion con Analisis

> Seccion condicional. Solo si se proporciono documento de analisis.
> Si no se proporciono, indicar: "No se proporciono documento de analisis para comparacion."

### Cobertura de Tareas

| Aspecto | Planificado (analisis) | Real (Odoo) | Diferencia |
|---------|----------------------|-------------|------------|
| Tareas | {n_analisis} | {n_odoo} | {diferencia} |
| Items checklist | {items_analisis} | {items_odoo} | {diferencia} |

### Diferencias Detectadas

**Tareas en analisis sin equivalente en Odoo**:
{lista_tareas_faltantes}

**Tareas en Odoo sin equivalente en analisis**:
{lista_tareas_extra}

---

## 6. Recomendacion y Proxima Evaluacion

### Resumen de Impacto

| Metrica | Antes | Despues (proyectado) |
|---------|-------|---------------------|
| Health Score | {score_antes}/100 | {score_despues}/100 |
| Estado | {estado_antes} | {estado_despues} |
| Tareas sin fecha | {sin_fecha_antes} | {sin_fecha_despues} |
| Desajuste horas | {desajuste_antes}h | {desajuste_despues}h |
| Tareas vencidas | {vencidas_antes} | {vencidas_despues} |

### Accion Inmediata Recomendada

{accion_inmediata}

### Proxima Evaluacion

- **Fecha sugerida**: {fecha_proxima_revision}
- **Re-ejecutar**: `project-health-monitor` para verificar mejora del score
- **Foco**: {foco_proxima_revision}

---

*Informe generado por Project Health Fixer - Indaws*
*Proyecto: {proyecto} (ID: {project_id})*
```
