# ¡Bienvenido/a a Odoo Mentor! 🎓

Soy tu asistente proactivo de **Indaws**. A partir de mañana recibirás seguimiento inteligente adaptado a tu rol, sin que tengas que pedirlo.

---

## ¿Qué voy a hacer por ti?

### Si eres {{rol}}:

{{#if rol_pm}}
Como **Project Manager**, te ayudaré a:
- 📊 Monitorizar el Health Score de todos tus proyectos cada día
- 🔴 Alertarte cuando un proyecto baje de 70 o tenga caídas bruscas
- ⏰ Recordarte plazos, go-lives próximos y hitos pendientes
- 📋 Los viernes, preparar la revisión semanal de tus proyectos
- 💡 Sugerirte skills como `/project-health-monitor` o `/project-weekly-update` cuando detecte situaciones que lo requieran
{{/if}}

{{#if rol_developer}}
Como **Desarrollador/a**, te ayudaré a:
- 📋 Organizar tu día: tareas pendientes, planning slots, deadlines
- ⏱️ Recordarte imputar horas si detecta que faltan
- 🔴 Alertarte sobre tareas vencidas o que vencen hoy/mañana
- 💡 Sugerirte `/dev-assistant` para gestionar tu trabajo diario
{{/if}}

{{#if rol_tl}}
Como **Team Leader**, te ayudaré a:
- 👥 Visión agregada de todos los proyectos de tu equipo
- 🔴 Alertarte sobre proyectos en riesgo en tu departamento
- ⏱️ Detectar empleados que no han imputado horas (> 2 días)
- 📊 Distribución de carga del equipo
- 💡 Sugerirte skills para gestión de equipo y proyectos
{{/if}}

{{#if rol_sales}}
Como **Comercial**, te ayudaré a:
- 📈 Monitorizar tu pipeline CRM y Health Score
- 🔴 Alertarte sobre oportunidades estancadas
- 📅 Recordarte demos próximas y preparación necesaria
- 💰 Vigilar presupuestos envejecidos sin respuesta
- 💡 Sugerirte `/lead-data-collection`, `/demo-script-generator` o `/project-budget-estimator` según la situación
{{/if}}

{{#if rol_sales_manager}}
Como **Director/a Comercial**, te ayudaré a:
- 👥 Visión del pipeline de todo el equipo comercial
- 📊 Scoring individual de cada vendedor/a
- 📈 Cobertura de pipeline vs objetivo mensual/trimestral
- 🔴 Alertarte sobre vendedores con pipeline débil o baja actividad
- 💡 Señalar problemas de gobernanza CRM (datos incompletos)
{{/if}}

---

## ¿Cómo funciono?

| Hora | Qué hago | Detalle |
|------|----------|---------|
| ⏰ 09:00 | Briefing matutino completo | Scores, alertas, recomendaciones, citas próximas |
| 🔄 ~12:00 | Revisión mediodía | Solo cambios desde la mañana. Si no hay novedades, silencio |
| 📊 ~15:00 | Pulso vespertino | Plazos próximos (3 días), go-lives (14 días), cambios |
| 🌙 ~18:00 | Resumen del día | Evolución del día, tareas completadas, pendiente mañana |

> **Los viernes** a las 18:00 incluyo además una **revisión semanal** con tendencias y comparativa semana a semana.

---

## Configuración completada

- ✅ Credenciales Odoo verificadas
- ✅ Rol: **{{rol}}**
- ✅ Proyectos/Pipeline detectados: **{{count}}**
- ✅ Tarea programada: **Lunes a Viernes, 09:00**

---

## Comandos útiles

- Puedes hablarme en cualquier momento durante la sesión activa (09:00 a ~18:00)
- Si necesitas un análisis fuera de horario, lanza manualmente el skill que necesites
- Los viernes genero una revisión semanal automática
- Puedes configurar preferencias en `preferences.json` (modo minimal, exclusiones, etc.)

---

¡Nos vemos mañana a las 9! 🚀
