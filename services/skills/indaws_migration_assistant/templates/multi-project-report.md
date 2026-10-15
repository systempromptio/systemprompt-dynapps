# Template: Portfolio de Migraciones

> Usar cuando PM o COO pregunta: "¿Cómo van las migraciones?",
> "¿Cómo van todos los proyectos?", "Comparar proyectos",
> "¿Cuál va peor?", "Portfolio", "Estado general"
> Funcionalidad F14: Ejecutar `scripts/multi_project_status.py`.

---

## Portfolio de Migraciones

**Fecha:** {fecha} | **Proyectos activos:** {n}

### Resumen

| Indicador | Valor |
|-----------|-------|
| Total proyectos | {n} |
| Tareas totales completadas | {n}/{total} ({pct}%) |
| Horas consumidas (global) | {hh:mm} / {budget_hh:mm} |
| Alertas activas | {n} |

### Vista comparativa

| Proyecto | Tareas | Horas | Gates | Go-Live | Salud |
|----------|--------|-------|-------|---------|-------|
| {nombre} | {n}/{total} ({pct}%) | {consumed}/{budget} ({pct}%) | {n}/{total} | {fecha} ({dias}d) | {indicador} |

### Proyecto más avanzado
**{nombre}** — {pct}% completado

### Proyecto con más riesgo
**{nombre}** — {motivos}

### Alertas por proyecto

| Proyecto | Tipo | Mensaje |
|----------|------|---------|
| {nombre} | {tipo} | {mensaje} |

---

## Reglas de formato

- SIEMPRE ejecutar `scripts/multi_project_status.py` — NO calcular manualmente
- Si el usuario pide un proyecto específico por nombre, filtrar la salida para mostrar solo ese proyecto
- Si el usuario es COO: énfasis en salud general, forecast y proyectos en riesgo
- Si el usuario es PM: énfasis en tareas, horas y próximas acciones por proyecto
- Indicadores de salud: 🟢 verde (>70), 🟡 amarillo (40-70), 🔴 rojo (<40)
- Ordenar proyectos por salud ascendente (peor primero) para visibilidad de problemas
- Máximo 3 alertas por proyecto en la tabla de alertas
- Si solo hay 1 proyecto, recomendar usar F12 (Dashboard de Salud) en su lugar
