# Template: Informe de Progreso de Migración (TL/PM)

> Usar cuando un TL o PM pregunta: "¿Cómo vamos?", "Estado de la migración", "Informe de progreso"
> Funcionalidad F3: Leer tareas + timesheets, calcular métricas.

---

## Informe de Migración — {nombre_proyecto}

**Fecha:** {fecha} | **Fase actual:** Fase {fase_num} — {fase_nombre}
**Migración:** v{version_origen} → v{version_destino}

### Progreso global

| Métrica | Valor |
|---|---|
| Tareas completadas | {completadas} / {totales} ({porcentaje_tareas}%) |
| Horas consumidas | {horas_consumidas}h / {horas_presupuesto}h ({porcentaje_horas}%) |
| Horas facturables | {horas_facturables}h ({porcentaje_facturables}%) |
| Horas no facturables | {horas_no_facturables}h ({porcentaje_no_facturables}%) |

```
Progreso tareas: [████████████░░░░░░░░] {porcentaje_tareas}%
Consumo horas:   [██████████░░░░░░░░░░] {porcentaje_horas}%
```

### Estado por tarea

| # | Tarea | Módulo | Estado | Horas | Asignado | Última actualización |
|---|---|---|---|---|---|---|
| 1 | {tarea} | {modulo} | ✅ Completada | {horas}h | {nombre} | {fecha} |
| 2 | {tarea} | {modulo} | 🔄 En progreso | {horas}h | {nombre} | {fecha} |
| 3 | {tarea} | {modulo} | 🔴 Bloqueada | {horas}h | {nombre} | {fecha} |
| 4 | {tarea} | {modulo} | ⬜ Pendiente | {horas}h | {nombre} | — |

**Leyenda:** ✅ Completada | 🔄 En progreso | 🔴 Bloqueada | ⬜ Pendiente

### Desviaciones

{si_desviacion}
> ⚠️ **ALERTA:** Horas consumidas ({porcentaje_horas}%) > Progreso tareas ({porcentaje_tareas}%).
> El proyecto consume más horas de las que avanza. Revisar con TL.

{si_no_desviacion}
> ✅ Sin desviaciones significativas. Progreso alineado con consumo de horas.

### Distribución de horas por desarrollador

| Desarrollador | Horas fact. | Horas no fact. | Total | % del presupuesto |
|---|---|---|---|---|
| {nombre} | {h_fact}h | {h_nofact}h | {h_total}h | {porcentaje}% |

### Forecast

| Métrica | Valor |
|---|---|
| Velocidad actual | {tareas_por_semana} tareas/semana |
| Tareas restantes | {tareas_restantes} |
| Semanas estimadas | {semanas_restantes} |
| Fecha estimada fin | {fecha_estimada} |
| Fecha planificada | {fecha_planificada} |
| Variación | {variacion} |

---

## Reglas de formato

- Cálculos SIEMPRE con calculate_progress.py — NO calcular manualmente
- Datos de tareas vía API (project.task search_read)
- Datos de horas vía API (account.analytic.line search_read)
- Alerta automática si: horas consumidas > 80% AND tareas completadas < 80%
- Forecast basado en velocidad de últimas 2 semanas (no de todo el proyecto)
- Si no hay timesheets, indicar que no se puede calcular el informe de horas
