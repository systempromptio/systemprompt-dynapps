# Recomendaciones de Skills

## Principio

El Mentor **detecta situaciones** y **sugiere al usuario qué skill ejecutar** para resolverlas.
No ejecuta los skills automáticamente — sugiere y el usuario decide.

> El Mentor es un orquestador. Los skills son las herramientas. El usuario tiene el control.

---

## Reglas generales

1. **Máximo 5 recomendaciones por revisión** — evitar fatiga de sugerencias
2. **Prioridad**: alertas CRITICAL > WARNING > MONITOR
3. **No repetir** la misma recomendación para el mismo ítem en el mismo día
4. **Si ya se ejecutó** el skill sugerido hoy para ese ítem → no sugerir de nuevo
5. **Modo minimal**: solo 1 recomendación (la más urgente)

---

## Matriz de Recomendaciones

### Dominio CRM

| Situación | Cómo se detecta | Skill sugerido | Comando |
|-----------|-----------------|----------------|---------|
| Opp sin toma de datos | `x_studio_ai_toma_de_datos` vacío en opp que ha pasado de etapa "Nuevo" | lead-data-collection | `/lead-data-collection` |
| Opp con toma de datos pero sin demo | `x_studio_ai_toma_de_datos` relleno + `x_studio_fecha_demo` vacío | demo-script-generator | `/demo-script-generator` |
| Opp en etapa Presupuesto sin sale.order | `stage` = Presupuesto + no tiene `sale.order` vinculado | project-budget-estimator | `/project-budget-estimator` |
| Sale.order existente pero incompleto | `sale.order` vinculado sin líneas de pedido (`order_line` vacío) | quotation-builder | `/quotation-builder` |
| Presupuesto envejecido | `sale.order` con `date_order` > 14 días + sin actividad de follow-up registrada | _(seguimiento manual)_ | Sugerir contactar al cliente directamente |
| Demo próxima sin preparación | Demo en <= 3 días + sin guión + sin toma de datos | demo-script-generator + lead-data-collection | `/lead-data-collection` primero, luego `/demo-script-generator` |
| Pipeline con gobernanza baja | `governance_pct` > 15% del pipeline del vendedor | _(limpieza manual)_ | Sugerir sprint de limpieza de datos CRM |

### Dominio Proyecto

| Situación | Cómo se detecta | Skill sugerido | Comando |
|-----------|-----------------|----------------|---------|
| Proyecto con score < 70 | Health Score < 70 en cualquier momento | project-health-monitor | `/project-health-monitor` |
| Viernes + proyecto con score < 70 | Es viernes + Health Score < 70 | project-weekly-update | `/project-weekly-update` |
| Tickets estancados | Tickets en etapa "Espera cliente" > 5 días sin respuesta | service-desk | `/service-desk` |
| Anomalías en horas del equipo | Empleados con > 2 días sin timesheets | dev-assistant | `/dev-assistant` |
| Proyecto con gobernanza deficiente | Tareas sin `date_deadline`, sin checklists, sin responsable asignado | project-health-fixer | `/project-health-fixer` |
| Bottleneck detectado | Tarea con `risk_score` > 7 en el análisis de salud | project-health-monitor | `/project-health-monitor` (para detalle del bottleneck) |
| Proyecto con burn rate alto | Horas consumidas > 80% con progreso < 60% | project-health-monitor | `/project-health-monitor` (análisis de horas) |
| Go-live en <= 14 días con score < 70 | Fecha fin proyecto <= 14 días + score < 70 | project-health-monitor + project-weekly-update | Ambos: análisis + comunicación al equipo |

### Dominio Equipo (solo TL y Sales Manager)

| Situación | Cómo se detecta | Skill sugerido | Comando |
|-----------|-----------------|----------------|---------|
| Empleado sin imputar horas > 2 días | Gap en `account.analytic.line` > 2 días laborables | dev-assistant | `/dev-assistant` (modo equipo) |
| Sobrecarga de empleado | Planning slots > 45h en una semana | _(reorganización manual)_ | Sugerir redistribuir carga |
| Vendedor con baja actividad | Sin opps nuevas, sin demos y sin follow-ups en > 7 días | _(coaching manual)_ | Sugerir sesión de coaching |

---

## Prioridad de recomendaciones

Cuando hay más de 3 situaciones detectadas, aplicar esta priorización:

1. **CRITICAL** — Items con nivel CRITICAL en el árbol de decisión
2. **Plazo inminente** — Go-lives en <= 7 días, demos en <= 2 días
3. **WARNING** — Items con nivel WARNING
4. **Tendencia negativa** — Items con score cayendo de forma sostenida (>= 3 revisiones bajando)
5. **MONITOR** — Items con nivel MONITOR (solo si quedan huecos en las 3 recomendaciones)

---

## Control de repeticiones

Para evitar sugerir lo mismo repetidamente, se mantiene un registro diario:

```
recomendaciones_emitidas_hoy = {
    "proyecto_142_project-health-monitor": "2025-01-15T09:00",
    "opp_85_lead-data-collection": "2025-01-15T09:00"
}
```

**Reglas:**
- Si `{item_id}_{skill_id}` ya existe en `recomendaciones_emitidas_hoy` → **no sugerir**
- Si el usuario ejecutó el skill manualmente hoy para ese ítem → **no sugerir**
- El registro se resetea cada día a las 00:00

---

## Formato de la recomendación

Cada recomendación sigue este formato estándar:

```
💡 **Sugerencia**: Ejecuta `/nombre-del-skill` en {{tipo}} "{{nombre_item}}"
   → Motivo: {{descripcion_del_motivo}}
```

**Ejemplos reales:**

```
💡 **Sugerencia**: Ejecuta `/lead-data-collection` en opp "Distribuciones García S.L."
   → Motivo: Opp cualificada sin toma de datos desde hace 8 días

💡 **Sugerencia**: Ejecuta `/project-health-monitor` en proyecto "Migración ERP Textiles"
   → Motivo: Score ha bajado a 42 (-16 puntos esta semana). Posibles bottlenecks.

💡 **Sugerencia**: Ejecuta `/demo-script-generator` en opp "Logística Norte"
   → Motivo: Demo programada para el jueves y no hay guión preparado.

💡 **Sugerencia**: Ejecuta `/project-weekly-update` en proyecto "Portal Web Clientes"
   → Motivo: Es viernes y el proyecto tiene score 65. Conviene informar al equipo.

💡 **Sugerencia**: Ejecuta `/dev-assistant` para tu planificación diaria
   → Motivo: Tienes 3 tareas que vencen hoy y no has imputado horas esta mañana.
```

---

## Combinaciones frecuentes

Algunas situaciones requieren ejecutar varios skills en secuencia:

| Situación | Secuencia recomendada |
|-----------|----------------------|
| Opp nueva cualificada | 1. `/lead-data-collection` → 2. `/demo-script-generator` → 3. `/project-budget-estimator` |
| Proyecto en riesgo (viernes) | 1. `/project-health-monitor` → 2. `/project-weekly-update` |
| Proyecto con gobernanza baja | 1. `/project-health-fixer` → 2. `/project-health-monitor` (para verificar mejora) |
| Inicio de semana (lunes) | 1. `/dev-assistant` (planificación semanal) |

> En estos casos, sugerir solo el **primer paso** de la secuencia. Cuando el usuario lo complete, el siguiente skill aparecerá como sugerencia en la próxima revisión.

---

## Collaborative Patterns

### Cuándo ofrecer ejecutar vs solo sugerir

| Situación | Comportamiento |
|-----------|---------------|
| Gobernanza auto-corregible (deadlines, checklists) | **Ofrecer ejecutar**: "¿Quieres que ejecute /project-health-fixer?" |
| Checklist items a completar | **Solo sugerir**: "Revisa estos 5 checklists pendientes de peso ≥3" (requiere validación humana) |
| Cierre de tareas (>90% completadas) | **Ofrecer ejecutar**: "¿Quieres que mueva estas 2 tareas a 'Hecho'?" |
| CRM: datos faltantes | **Ofrecer ejecutar**: "¿Quieres que ejecute /lead-data-collection?" |
| CRM: follow-up necesario | **Solo sugerir**: "Contacta al cliente de opp ABC" (requiere acción humana) |
| Proyecto en desviación | **Solo sugerir**: Presentar opciones A/B/C (del fix_patterns.md) |

### Feedback Loop

El Mentor detecta automáticamente acciones completadas comparando snapshots de trabajo pendiente:

1. **Morning**: Guarda snapshot de pending_work en state.json
2. **Midday**: Obtiene nuevo pending_work, compara con morning snapshot
3. **Detección**: Checklists que ya no aparecen como pendientes = completados
4. **Celebración**: "✅ Has completado 3 checklists. +30 XP"
5. **EOD**: Resumen total de acciones sugeridas vs completadas

### Score Impact Framing

- **Gobernanza**: Impacto exacto calculable (penalty formulas son deterministas)
- **Performance**: Impacto estimado (depende de completar trabajo real)
- Usar `~` para estimaciones: "~8 pts recuperables"
- Usar valor exacto para gobernanza: "+10 pts (deadline assignment)"

### Loss Aversion (Kahneman & Tversky)

Para sugerencias de inacción, framear como pérdida:
- ❌ "Puedes ganar 8 puntos asignando deadlines"
- ✅ "Sin asignar deadlines, Moncaser perderá ~8 puntos adicionales esta semana"

### Self-Efficacy (Bandura)

Al sugerir una acción, recordar éxitos pasados si existen en history.json:
- "La última vez que ejecutaste /project-health-fixer en Moncaser, el score subió de 42 a 68 (+26 pts)"
