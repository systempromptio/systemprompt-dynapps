# Template: Resumen TL → PM (semanal)

> Usar cuando un TL pide: "Resumen para PM", "Informe semanal"
> Ver references/communication.md para protocolo completo.

---

## Resumen Semanal TL → PM — Semana {num_semana}, {fecha}

**Proyecto:** {nombre_proyecto}
**Migración:** v{version_origen} → v{version_destino}
**Fase actual:** {fase_num} — {fase_nombre}

### % Progreso global
**{porcentaje_global}%** ({tareas_completadas}/{tareas_totales} tareas completadas)

```
[████████████░░░░░░░░] {porcentaje_global}%
```

### Horas consumidas vs presupuesto
| Métrica | Valor |
|---|---|
| Horas consumidas | {horas_consumidas}h / {horas_presupuesto}h ({porcentaje_horas}%) |
| Horas facturables | {horas_facturables}h ({porcentaje_facturables}%) |
| Horas no facturables | {horas_no_facturables}h ({porcentaje_no_facturables}%) |

**Estado:** {estado_horas}

> {estado_horas}: "Alineado" si % horas ≈ % progreso (±10pp). "Alerta" si % horas > % progreso + 10pp.

### Desviaciones
| Desviación | Impacto | Plan de acción |
|---|---|---|
| {descripcion_desviacion} | +{horas}h | {plan_accion} |

> Si no hay desviaciones: "Sin desviaciones significativas esta semana."

### Bloqueos escalados
| # | Bloqueo | Origen | Acción requerida del PM |
|---|---|---|---|
| 1 | {bloqueo} | {origen} | {accion_pm} |

> Si no hay bloqueos: "Ningún bloqueo que requiera intervención del PM."

### Forecast fecha de entrega
**Fecha estimada:** {fecha_estimada}
**Variación vs plan:** {variacion}

---

## Reglas de formato

- OBLIGATORIO: TL envía 1x/semana (ej. viernes)
- Calcular métricas con calculate_progress.py — NO calcular manualmente
- El estado de horas debe ser objetivo: "Alineado" / "Alerta" / "Crítico"
- Incluir forecast basado en velocidad actual (tareas completadas por semana)
- Si PM necesita actuar (bloqueo, desviación >15%), marcarlo claramente
