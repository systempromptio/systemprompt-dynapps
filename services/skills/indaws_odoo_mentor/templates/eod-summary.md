## Resumen del día - {{fecha}}

---

### Lo que pasó hoy

<!-- Resumen narrativo de todos los cambios relevantes del día -->
<!-- Agrupar por dominio: Proyectos, CRM, Equipo -->

#### Proyectos
{{#each cambios_proyectos}}
- **{{proyecto_nombre}}**: {{resumen_cambio}}
{{/each}}
{{#unless cambios_proyectos}}
Sin cambios relevantes en proyectos hoy.
{{/unless}}

#### Pipeline CRM
{{#each cambios_crm}}
- **{{opp_nombre}}**: {{resumen_cambio}}
{{/each}}
{{#unless cambios_crm}}
Sin cambios relevantes en el pipeline hoy.
{{/unless}}

---

### Evolución de scores

| Proyecto/Pipeline | Mañana | Ahora | Delta | Tendencia |
|-------------------|--------|-------|-------|-----------|
| {{nombre}} | {{score_manana}} | {{score_ahora}} | {{delta}} | {{tendencia}} |

<!-- delta: +N en verde, -N en rojo, 0 neutro -->
<!-- tendencia: ↑ ↓ → -->

---

### Tareas completadas hoy

<!-- Detectadas por cambios de etapa o timesheets registrados -->

{{#if hay_tareas_completadas}}
{{#each tareas_completadas}}
- ✅ {{tarea_nombre}} ({{proyecto_nombre}}) — {{horas_dedicadas}}h
{{/each}}

**Total horas imputadas hoy:** {{total_horas_hoy}}h
{{else}}
Sin tareas completadas registradas hoy.
{{/if}}

---

### Pendiente para mañana

<!-- Acciones que se arrastran o que requieren atención mañana -->

{{#each pendiente_manana}}
- {{icono_prioridad}} {{descripcion}} ({{proyecto_opp}})
{{/each}}

<!-- Ejemplo:
- 🔴 Revisar bottleneck en "Migración ERP" — score sigue en 38
- 🟠 Contactar cliente "Logística Norte" — demo sin confirmar
- 🟡 Imputar horas del día si no se ha hecho
-->

{{#if viernes}}

---

## 📋 Revisión semanal (viernes)

### Resumen de la semana

<!-- Comparación semana actual vs semana anterior -->

| Métrica | Semana anterior | Esta semana | Delta |
|---------|-----------------|-------------|-------|
| Score medio proyectos | {{score_proy_prev}} | {{score_proy_now}} | {{delta_proy}} |
| CRM Health Score | {{crm_score_prev}} | {{crm_score_now}} | {{delta_crm}} |
| Tareas completadas | {{tareas_prev}} | {{tareas_now}} | {{delta_tareas}} |
| Horas imputadas | {{horas_prev}}h | {{horas_now}}h | {{delta_horas}} |
| Opps ganadas | {{opps_won_prev}} | {{opps_won_now}} | {{delta_won}} |
| Opps perdidas | {{opps_lost_prev}} | {{opps_lost_now}} | {{delta_lost}} |

### Tendencias (últimas 4 semanas)

<!-- Datos obtenidos de registros project.update y CRM -->

| Semana | Score Proy. | Score CRM | Tareas | Horas |
|--------|-------------|-----------|--------|-------|
| {{semana_4}} | {{sp_4}} | {{sc_4}} | {{t_4}} | {{h_4}} |
| {{semana_3}} | {{sp_3}} | {{sc_3}} | {{t_3}} | {{h_3}} |
| {{semana_2}} | {{sp_2}} | {{sc_2}} | {{t_2}} | {{h_2}} |
| {{semana_1}} (actual) | {{sp_1}} | {{sc_1}} | {{t_1}} | {{h_1}} |

### Actualización semanal de proyectos

<!-- Sugerir ejecutar /project-weekly-update para proyectos con score < 70 -->

{{#each proyectos_bajo_70}}
💡 **Sugerencia**: Ejecuta `/project-weekly-update` en "{{proyecto_nombre}}" (score: {{score}})
   → Genera el informe semanal para el cliente y el equipo.
{{/each}}

{{#unless proyectos_bajo_70}}
Todos los proyectos por encima de 70. No se requiere actualización semanal urgente.
{{/unless}}

{{/if}}

---

---

### 📊 Resumen Gamificación

{{#if gamification_enabled}}

| Métrica | Hoy | Acumulado |
|---------|-----|-----------|
| XP ganado | +{{xp_today}} | {{xp_total}} XP |
| Nivel | {{level}} | {{level_name}} |
| Racha | {{streak}} días | Record: {{streak_best}} días |
| Acciones completadas | {{actions_done}}/{{actions_total}} | — |
| Reto del día | {{challenge_status}} | — |

{{#if new_achievements}}
### 🏅 Logros desbloqueados hoy
{{#each new_achievements}}
- {{icon}} **{{name}}**: {{description}}
{{/each}}
{{/if}}

{{#if level_up}}
### 🎉 ¡LEVEL UP!
Has subido a **Nivel {{new_level}}: {{new_level_name}}**
{{/if}}

{{/if}}

---

### 🌟 Momento del día

<!-- Peak-End Rule: SIEMPRE terminar con algo positivo -->
<!-- Buscar el mejor logro/mejora del día y destacarlo -->

{{positive_highlight}}

<!-- Ejemplos:
- "¡Gran día! El score de Moncaser subió 12 puntos gracias a tu trabajo en checklists."
- "Has completado 8 checklists hoy. ¡Consistencia que marca la diferencia!"
- "Tu racha de 7 días es impresionante. Mañana puedes llegar al récord de 8."
- "Pipeline CRM limpio: todas las opps tienen toma de datos. Buen trabajo."
-->

---

Hasta mañana. ¡Buen descanso! 🌙
