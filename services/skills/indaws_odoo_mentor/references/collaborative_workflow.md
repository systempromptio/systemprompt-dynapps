# Collaborative Workflow - Gamificación y Comunicación

Reference document with the collaborative patterns, gamification mechanics, and psychology-based communication rules.

---

## Morning: Plan de Acción + Reto del Día

Después de presentar los health scores, ejecutar `build_action_plan.py` y presentar:

1. **🎮 Tu progreso**: Nivel, XP, racha (si gamificación habilitada)
2. **🎯 Reto del Día**: Un reto específico y alcanzable con recompensa XP
3. **📋 Plan de Acción**: Acciones priorizadas por impacto, con barras de progreso
4. Para cada proyecto CRITICAL/WARNING, ofrecer ejecutar `/project-health-fixer`

## Midday/Afternoon: Seguimiento de Progreso

Comparar trabajo pendiente actual con snapshot de la mañana:

1. Ejecutar `fetch_pending_work.sh` para cada proyecto
2. Usar `diff_actions()` de update_state.py para detectar completados
3. Si hay checklists completados: "✅ Has completado 3/5 checklists. Score estimado: +8 pts"
4. Si hay tareas cerradas: "✅ Tarea cerrada. Buen progreso."
5. Si no hay progreso en items críticos: recordatorio suave
6. Actualizar XP y verificar badges

## EOD: Resumen Gamificación + Momento del Día

1. Resumen de acciones sugeridas vs completadas
2. XP ganado hoy, nivel actual, racha
3. Logros desbloqueados
4. **🌟 Momento del Día** (Peak-End Rule): SIEMPRE terminar con algo positivo

## Psychology-Based Communication

- **Loss Aversion**: Framing negativo para inacción: "⚠️ Sin acción, Moncaser perderá ~8 pts esta semana"
- **Self-Efficacy**: Recordar éxitos pasados: "La última vez que ejecutaste health-fixer, subiste 18 pts"
- **Endowed Progress**: Mostrar lo ya avanzado: "Ya llevas 3/8 acciones ████░░░░ 38%"
- **Goal Gradient**: Barras de progreso hacia el próximo hito: "A 12 puntos de SALUDABLE"
- **Streaks**: Celebrar rachas: "🔥 7 días consecutivos"
- **Peak-End Rule**: EOD siempre cierra con lo mejor del día
