## Revisión mediodía - {{hora}}

---

### Cambios desde las {{hora_anterior}}

<!-- Si no hay cambios, mostrar solo esta línea y omitir el resto: -->
{{#if sin_cambios}}
Sin novedades desde esta mañana. ¡Sigue así! 💪
{{/if}}

{{#unless sin_cambios}}

<!-- Solo mostrar los ítems que han cambiado desde el briefing matutino -->

| Proyecto/Opp | Cambio detectado | Impacto |
|--------------|------------------|---------|
| {{item_nombre}} | {{cambio_descripcion}} | {{impacto_icono}} {{impacto_texto}} |

<!-- impacto_icono: 🟢 mejora, 🔴 empeora, 🟡 neutro -->

<!-- Ejemplos de cambios detectados:
| Portal Web      | Score bajó de 74 a 68 (-6)              | 🔴 Pasa a WARNING   |
| Migración ERP   | 2 tareas completadas, score sube a 42   | 🟢 Sigue CRITICAL   |
| Opp Logística   | Cliente confirmó demo para mañana       | 🟢 Avanza           |
-->

---

### Nuevas alertas

<!-- Solo alertas NUEVAS o que han EMPEORADO desde el briefing matutino -->
<!-- Si no hay nuevas alertas: "Sin nuevas alertas." -->

{{#each nuevas_alertas}}
- {{icono}} [{{nivel}}] {{mensaje}}
  → {{accion_sugerida}}
{{/each}}

---

### Recordatorio

<!-- Plazos en las próximas 48 horas -->

{{#if hay_plazos_48h}}
⏰ **Plazos en las próximas 48h:**

{{#each plazos_48h}}
- **{{fecha_limite}}** — {{tipo}}: {{descripcion}} ({{proyecto_opp}})
{{/each}}
{{else}}
Sin plazos urgentes en las próximas 48 horas.
{{/if}}

{{/unless}}

---

### 📋 Progreso en Acciones

<!-- Comparar acciones sugeridas en la mañana con estado actual -->

{{#if actions_tracked}}
Acciones del día: {{action_progress_bar}} {{actions_completed}}/{{actions_total}} ({{actions_pct}}%)

{{#each completed_actions}}
- ✅ {{description}} → +{{xp_earned}} XP
{{/each}}

{{#if daily_challenge_completed}}
🎯 **¡RETO COMPLETADO!** +{{challenge_xp}} XP 🎉
{{/if}}

{{#if new_achievements}}
🏅 **¡Logro desbloqueado!** {{#each new_achievements}}{{icon}} {{name}} {{/each}}
{{/if}}

{{#if pending_high_priority}}
**Pendiente:**
{{#each pending_high_priority}}
- ⏳ {{description}} → **+{{estimated_impact_pts}} pts**
{{/each}}
{{/if}}
{{/if}}

---

> Próxima revisión: ~15:00
