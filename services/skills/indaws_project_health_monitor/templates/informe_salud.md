# Template: Informe de Salud del Proyecto

> Usa este template para generar el informe final. Reemplaza los placeholders `{...}` con los datos calculados.

---

```markdown
# Informe de Salud del Proyecto: {nombre_proyecto}

**Fecha del análisis**: {fecha_referencia}
**Generado por**: Project Health Monitor - Indaws

---

## 1. Dashboard de Progreso

### Health Score

{icono_estado} **{estado}** {barra_visual} **{score}/100**

| Métrica | Valor | Objetivo | Delta | Estado |
|---------|-------|----------|-------|--------|
| Timeline (T%) | {T_pct}% | - | - | Referencia |
| Checklists (C%) | {C_pct}% | {T_pct}% | {delta_C:+}pp | {icono_C} |
| Horas (H%) | {H_pct}% | {T_pct}% | {delta_H:+}pp | {icono_H} |

**Escenario detectado**: {escenario} - {descripcion_escenario}

{avisos_especiales}

---

## 2. Resumen del Proyecto

| Campo | Valor |
|-------|-------|
| **Proyecto** | {nombre_proyecto} |
| **Cliente** | {nombre_cliente} |
| **Responsable** | {responsable} |
| **Fecha inicio** | {fecha_inicio} |
| **Fecha fin planificada** | {fecha_fin} |
| **Días transcurridos** | {dias_transcurridos} de {dias_totales} |
| **Horas presupuestadas** | {horas_presupuestadas}h |
| **Horas consumidas** | {horas_consumidas}h |
| **Tareas totales** | {num_tareas} |
| **Items de checklist** | {num_checklists} ({num_completados} completados) |

---

## 3. Progreso de Checklists por Tarea

| # | Tarea | Items | Completados | Peso Total | Peso Completado | C% | Estado |
|---|-------|-------|-------------|------------|-----------------|-----|--------|
| 1 | {tarea_1_nombre} | {items} | {completados} | {peso_total} | {peso_completado} | {C_pct}% | {icono} |
| 2 | {tarea_2_nombre} | ... | ... | ... | ... | ...% | ... |
| ... | ... | ... | ... | ... | ... | ... | ... |
| **TOTAL** | | **{total_items}** | **{total_completados}** | **{total_peso}** | **{total_peso_completado}** | **{C_pct_global}%** | |

---

## 4. Detalle de Items Críticos

### Items Vencidos (fecha planificada superada)

| Item | Tarea | Peso | Fecha Planificada | Días Retraso |
|------|-------|------|-------------------|--------------|
| {item_nombre} | {tarea} | {peso} | {fecha} | {dias} |

### Items de Alto Riesgo (peso >= 4, pendientes)

| Item | Tarea | Peso | Fecha Planificada |
|------|-------|------|-------------------|
| {item_nombre} | {tarea} | {peso} | {fecha} |

> Si no hay items vencidos ni de alto riesgo, indicar: "No se identificaron items críticos en este momento."

---

## 5. Consumo de Horas

### Distribución por Tarea

| # | Tarea | Horas Asignadas | Horas Consumidas | H% | Desviación |
|---|-------|-----------------|------------------|-----|------------|
| 1 | {tarea_nombre} | {asignadas}h | {consumidas}h | {H_pct}% | {desviacion} |
| ... | ... | ... | ... | ... | ... |
| **TOTAL** | | **{total_asignadas}h** | **{total_consumidas}h** | **{H_pct_global}%** | |

### Indicadores de Horas

| Indicador | Valor |
|-----------|-------|
| **Burn Rate actual** | {burn_rate}h/semana |
| **Burn Rate esperado** | {burn_rate_esperado}h/semana |
| **EAC (Estimate at Completion)** | {EAC}h |
| **Sobrecosto/Ahorro proyectado** | {delta_EAC}h |
| **Semanas restantes** | {semanas_restantes} |
| **Horas restantes disponibles** | {horas_restantes}h |
| **Ritmo requerido** | {ritmo_requerido}h/semana |

---

## 6. Evaluación de Riesgos

### Bottlenecks Identificados

| Prioridad | Tarea | Risk Score | Items Pendientes | Peso Pendiente | Fecha Límite | Estado |
|-----------|-------|------------|------------------|----------------|--------------|--------|
| 🔴 CRÍTICO | {tarea} | {risk_score} | {pendientes} | {peso} | {fecha} | {estado} |
| 🟡 ALTO | {tarea} | {risk_score} | {pendientes} | {peso} | {fecha} | {estado} |

### Factor de Aceleración

| Indicador | Valor |
|-----------|-------|
| **Velocidad actual** | {velocidad} puntos/semana |
| **Velocidad requerida** | {velocidad_req} puntos/semana |
| **Factor de aceleración** | {factor}x |
| **Interpretación** | {interpretacion_factor} |

---

## 7. Plan de Acción Priorizado

### Escenario: {escenario} - {descripcion_escenario}

#### Acciones Inmediatas (esta semana)

| # | Acción | Responsable | Prioridad |
|---|--------|-------------|-----------|
| 1 | {accion_1} | {responsable} | ALTA |
| 2 | {accion_2} | {responsable} | ALTA |

#### Acciones a Corto Plazo (próximas 2 semanas)

| # | Acción | Responsable | Prioridad |
|---|--------|-------------|-----------|
| 1 | {accion_1} | {responsable} | MEDIA |
| 2 | {accion_2} | {responsable} | MEDIA |

#### Acciones Estratégicas

| # | Acción | Responsable | Prioridad |
|---|--------|-------------|-----------|
| 1 | {accion_1} | {responsable} | BAJA |
| 2 | {accion_2} | {responsable} | BAJA |

---

## 8. Próximos Pasos Recomendados

**Resumen ejecutivo para el Project Manager**:

1. **Estado actual**: {resumen_estado_1_linea}
2. **Riesgo principal**: {riesgo_principal}
3. **Acción más urgente**: {accion_urgente}
4. **Próxima revisión recomendada**: {fecha_proxima_revision}

---

*Informe generado automáticamente por Project Health Monitor - Indaws*
*Datos extraídos de {odoo_url} el {fecha_referencia}*
```
