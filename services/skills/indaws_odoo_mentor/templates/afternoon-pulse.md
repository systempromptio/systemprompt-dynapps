## Pulso vespertino - {{hora}}

---

### Plazos próximos

#### ⏰ Vence en 3 días

<!-- Tareas, hitos y compromisos que vencen en los próximos 3 días naturales -->

{{#if hay_plazos_3d}}
| Tipo | Ítem | Proyecto/Opp | Fecha límite | Responsable |
|------|------|--------------|--------------|-------------|
| {{tipo}} | {{nombre_item}} | {{proyecto_opp}} | {{fecha_limite}} | {{responsable}} |

<!-- tipo: Tarea, Hito, Demo, Entrega, Presupuesto, etc. -->
{{else}}
Sin plazos en los próximos 3 días. 👍
{{/if}}

#### 🚀 Go-live en 14 días

<!-- Proyectos con fecha de go-live en los próximos 14 días naturales -->

{{#if hay_golive_14d}}
| Proyecto | Fecha go-live | Días restantes | Score actual | Estado |
|----------|---------------|----------------|--------------|--------|
| {{proyecto_nombre}} | {{golive_fecha}} | {{dias_restantes}} | {{score}}/100 | {{estado_icono}} |

<!-- Incluir notas de riesgo si score < 70 -->
{{#each golive_riesgo}}
> ⚠️ **{{proyecto_nombre}}**: Go-live en {{dias_restantes}} días con score {{score}}. Revisar con `/project-health-monitor`.
{{/each}}
{{else}}
Sin go-lives en los próximos 14 días.
{{/if}}

---

### Cambios desde mediodía

<!-- Solo deltas desde la revisión de mediodía -->
<!-- Si no hay cambios: omitir esta sección completamente -->

{{#unless sin_cambios_desde_mediodia}}
| Proyecto/Opp | Cambio detectado | Impacto |
|--------------|------------------|---------|
| {{item_nombre}} | {{cambio_descripcion}} | {{impacto_icono}} {{impacto_texto}} |
{{/unless}}

---

### Estado general

<!-- Resumen de una sola línea con emojis de estado -->

{{estado_general_resumen}}

<!-- Ejemplos:
"🟢🟢🟡🔴 — 2 proyectos sanos, 1 en vigilancia, 1 crítico. Pipeline CRM estable."
"🟢🟢🟢🟢 — Todo en orden. ¡Gran día!"
"🟠🔴🟡 — 1 proyecto en aviso, 1 crítico, 1 en vigilancia. Revisar Migración ERP."
-->

---

> Próxima revisión: ~18:00 (resumen del día)
