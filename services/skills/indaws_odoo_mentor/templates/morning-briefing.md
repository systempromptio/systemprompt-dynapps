# Buenos días, {{nombre}} 👋

## Resumen del día - {{fecha}}

---

### Tu Health Score Dashboard

<!-- Sección adaptativa según el rol del usuario -->

{{#if rol_pm_or_tl}}

#### Proyectos

| Proyecto | Score | Tendencia | Estado | Acción sugerida |
|----------|-------|-----------|--------|-----------------|
| {{proyecto_nombre}} | {{score}}/100 | {{tendencia}} | {{estado_icono}} | {{accion}} |

<!-- tendencia: ↑ subiendo, ↓ bajando, → estable -->
<!-- estado_icono: 🟢 >= 80, 🟡 70-79, 🟠 40-69, 🔴 < 40 -->

{{/if}}

{{#if rol_sales_or_pm}}

#### Pipeline CRM

| Métrica | Valor | vs. Objetivo |
|---------|-------|--------------|
| CRM Health Score | {{crm_score}}/100 | {{crm_vs_obj}} |
| Opps activas | {{opps_activas}} | — |
| Tasa de conversión | {{win_rate}}% | {{win_rate_obj}}% |
| Opps estancadas | {{stale_count}} | < {{stale_obj}} |
| Gobernanza CRM | {{governance_pct}}% | < 15% |

{{/if}}

{{#if rol_developer}}

#### Mis tareas

| Tipo | Detalle |
|------|---------|
| Planning slots hoy | {{planning_slots_hoy}} |
| Tareas pendientes | {{tareas_pendientes}} |
| Tareas que vencen hoy | {{tareas_vencen_hoy}} |
| Horas imputadas hoy | {{horas_hoy}}h / {{horas_objetivo}}h |
| Horas imputadas esta semana | {{horas_semana}}h / {{horas_semana_obj}}h |

{{/if}}

---

### Alertas prioritarias

<!-- Ordenadas por urgencia: CRITICAL > WARNING > MONITOR -->
<!-- Solo mostrar alertas que hayan cambiado o sean nuevas -->

{{#each alertas}}
- {{icono}} [{{nivel}}] {{mensaje}}
  → {{accion_sugerida}}
{{/each}}

<!-- Ejemplo:
- 🔴 [CRÍTICO] Proyecto "Migración ERP" ha bajado a score 35 (-18 puntos)
  → Revisar bottlenecks con `/project-health-monitor`
- 🟠 [AVISO] Oportunidad "Logística Norte" estancada 12 días en etapa Demo
  → Contactar al cliente o reagendar demo
- 🟡 [MONITOR] Proyecto "Portal Web" score 74, tendencia bajista ↓
  → Vigilar próximos días
-->

---

### 🎮 Tu progreso

<!-- Gamification: solo si gamification_enabled -->

{{#if gamification_enabled}}
**Nivel {{nivel}}: {{nivel_nombre}}** ({{xp_total}} XP) {{#if xp_to_next}} — {{xp_to_next}} XP para Nivel {{next_level}}{{/if}}
🔥 Racha: {{streak_current}} días | Record personal: {{streak_best}} días

{{#if new_achievements}}
🏅 **¡Nuevos logros!**
{{#each new_achievements}}
- {{icon}} **{{name}}**: {{description}}
{{/each}}
{{/if}}
{{/if}}

---

### 🎯 Reto del día

{{#if daily_challenge}}
> **{{daily_challenge.description}}**
> {{#each daily_challenge.actions}}
> → {{this}}
> {{/each}}
> Recompensa: +{{daily_challenge.xp_reward}} XP
{{#if daily_challenge.badge_progress}}
> Progreso badge "{{daily_challenge.badge_progress.name}}": {{daily_challenge.badge_progress.current}}/{{daily_challenge.badge_progress.target}} {{daily_challenge.badge_progress.icon}}
{{/if}}
{{else}}
¡Todos los proyectos bajo control! Mantén el buen trabajo.
{{/if}}

---

### 📋 Plan de Acción

<!-- Acciones priorizadas por impacto en score, con XP rewards -->

{{#each projects_with_actions}}
**{{project_name}}** — Score: {{current_score}} {{progress_bar}} → {{next_milestone.label}} (a {{next_milestone.gap}} pts)
Potencial: {{potential_score}} {{potential_estado}}

{{#each actions}}
- {{#if (eq priority "high")}}🔴{{else if (eq priority "medium")}}🟡{{else}}🟢{{/if}} {{description}} → **+{{estimated_impact_pts}} pts** (+{{xp_reward}} XP)
  {{fix_command}}
{{/each}}

{{/each}}

{{#if actions_summary}}
**Resumen:** {{actions_summary.total_actions}} acciones disponibles | +{{actions_summary.total_recoverable_pts}} pts recuperables | {{actions_summary.xp_available}} XP disponible
{{/if}}

---

### Skills recomendados

<!-- Basado en situaciones detectadas. Máximo 3 recomendaciones -->

{{#each recomendaciones}}
- 💡 `{{comando}}` en {{tipo}} "{{nombre}}" → {{motivo}}
{{/each}}

<!-- Ejemplo:
- 💡 `/lead-data-collection` en opp "ABC Corp" → falta toma de datos desde hace 6 días
- 💡 `/project-health-monitor` en proyecto "Portal Web" → score < 70
- 💡 `/demo-script-generator` en opp "Logística Norte" → demo programada en 2 días sin guión
-->

---

### Próximas citas y demos

<!-- Eventos de CRM en los próximos 3 días -->

| Fecha | Hora | Tipo | Cliente/Opp | Notas |
|-------|------|------|-------------|-------|
| {{cita_fecha}} | {{cita_hora}} | {{cita_tipo}} | {{cita_cliente}} | {{cita_notas}} |

<!-- Si no hay citas: "Sin citas programadas en los próximos 3 días." -->

---

> Próxima revisión: ~12:00
