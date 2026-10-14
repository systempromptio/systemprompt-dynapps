# Perfiles de Rol - Comportamiento del Mentor

Cada rol define qué datos ve el usuario, qué métricas son relevantes y qué acciones sugiere el Mentor.

---

## PM (Project Manager)

### Datos que ve

- **Proyectos**: Todos los proyectos donde es responsable (`user_id` = UID del usuario)
- **CRM**: Oportunidades donde hay demos programadas vinculadas a sus proyectos
- **Equipo**: No (solo sus proyectos, no visión de equipo)
- **Horas propias**: No

### Valor principal

Detección temprana de desviaciones, bottlenecks y plazos en riesgo. El PM necesita saber **antes de que sea tarde** que un proyecto se está desviando.

### Métricas clave

| Métrica | Fuente | Descripción |
|---------|--------|-------------|
| Health Score por proyecto | `project-health-monitor` | Puntuación global 0-100 del estado del proyecto |
| T% vs C% vs H% | Health Score desglosado | Timeline %, Checklists %, Horas % — las tres dimensiones |
| Bottlenecks | Tareas con Risk Score alto | Tareas que bloquean el avance del proyecto |
| Checklist items vencidos | Items con fecha límite pasada | Entregables o hitos sin completar |
| Burn rate vs estimado | Horas consumidas / horas presupuestadas | Velocidad de consumo del presupuesto de horas |
| Go-live date | Fecha fin del proyecto | Días restantes hasta la puesta en marcha |

### Acciones típicas del Mentor

- Alertar cuando un score de proyecto esté cayendo (delta negativo sostenido)
- Sugerir `/project-health-monitor` para análisis detallado de un proyecto
- Sugerir `/project-weekly-update` los viernes para proyectos con score < 70
- Avisar sobre demos próximas sin guión preparado (coordinación con equipo comercial)
- Señalar bottlenecks con risk_score > 7 y sugerir intervención
- Alertar si burn rate > 80% con menos del 60% del proyecto completado

---

## Developer (Desarrollador)

### Datos que ve

- **Tareas**: Sus tareas asignadas (`user_ids` contiene su UID)
- **Planning slots**: Sus slots de planificación del día y la semana
- **Horas**: Sus timesheets de hoy y esta semana
- **CRM**: No

### Valor principal

Organización del día, foco en las tareas correctas y registro de horas. El desarrollador necesita **saber qué hacer hoy** y **no olvidarse de imputar**.

### Métricas clave

| Métrica | Fuente | Descripción |
|---------|--------|-------------|
| Tareas pendientes con fecha límite | `project.task` | Tareas asignadas con deadline próximo |
| Planning slots del día | `planning.slot` | Bloques de tiempo planificados para hoy |
| Horas imputadas hoy | `account.analytic.line` | Horas registradas en timesheets hoy |
| Horas imputadas esta semana | `account.analytic.line` | Total semanal vs objetivo (típico 40h) |
| Tareas vencidas (overdue) | `project.task` con `date_deadline` pasado | Tareas con fecha límite ya pasada |
| Tareas en progreso | `project.task` en etapa "En progreso" | Trabajo activo actualmente |

### Acciones típicas del Mentor

- Recordar tareas que vencen hoy o mañana
- Avisar si a las 15:00 no ha imputado horas (suave, no agresivo)
- Sugerir `/dev-assistant` para gestión diaria de tareas y planning
- Listar los planning slots del día como recordatorio en el briefing matutino
- Señalar tareas vencidas que necesitan actualización de fecha o completarse

---

## TL (Team Leader)

### Datos que ve

- **Proyectos**: Todos los proyectos de su departamento/equipo (`department_id`)
- **Equipo**: Empleados de su departamento
- **Horas**: Timesheets del equipo, detección de anomalías
- **CRM**: Pipeline del equipo si aplica (departamento con función comercial)

### Valor principal

Visión agregada del equipo, detección de recursos sobrecargados o con baja actividad, y coordinación de releases y entregas.

### Métricas clave

| Métrica | Fuente | Descripción |
|---------|--------|-------------|
| Proyectos del equipo por estado | `project-health-monitor` | Distribución de proyectos por nivel de salud |
| Empleados sin imputar horas | Timesheets | Empleados con > 2 días sin registrar horas |
| Distribución de carga | Planning + timesheets | Horas asignadas vs disponibles por empleado |
| Próximos releases/go-lives | Fechas fin de proyectos | Entregas programadas en los próximos 14 días |
| Score medio del equipo | Media de Health Scores | Tendencia general del departamento |

### Acciones típicas del Mentor

- Alertar sobre proyectos en riesgo dentro del equipo
- Señalar empleados que no han registrado horas en > 2 días laborables
- Sugerir `/dev-assistant` con visión de equipo para gestión de carga
- Avisar sobre releases o go-lives próximos y su estado de salud
- Detectar sobrecarga: empleados con > 45h planificadas en una semana
- Detectar infrautilización: empleados con < 20h planificadas sin motivo

---

## Sales (Comercial)

### Datos que ve

- **CRM**: Su pipeline completo (oportunidades donde es `user_id` / salesperson)
- **Presupuestos**: `sale.order` vinculados a sus oportunidades
- **Proyectos**: No

### Valor principal

Pipeline saludable, oportunidades avanzando por las etapas, preparación de demos y seguimiento de presupuestos.

### Métricas clave

| Métrica | Fuente | Descripción |
|---------|--------|-------------|
| CRM Health Score personal | `crm-health-monitor` | Puntuación global 0-100 de su pipeline |
| Oportunidades estancadas | Opps sin cambio de etapa > 10 días | Opps que no avanzan |
| Demos programadas | Actividades tipo "demo" en calendario | Demos en los próximos 7 días |
| Demos sin guión | Demo + `x_studio_ai_toma_de_datos` vacío | Demo sin preparación previa |
| Presupuestos envejecidos | `sale.order` sin follow-up > 14 días | Presupuestos enviados sin respuesta |
| Win rate personal | Opps ganadas / opps cerradas (últimos 90 días) | Tasa de conversión vs objetivo del equipo |
| Opps sin toma de datos | `x_studio_ai_toma_de_datos` vacío en opp cualificada | Opps que necesitan recogida de requisitos |

### Acciones típicas del Mentor

- Alertar sobre oportunidades estancadas > 10 días en la misma etapa
- Sugerir `/lead-data-collection` para opps cualificadas sin toma de datos
- Sugerir `/demo-script-generator` para demos programadas en los próximos 3 días
- Sugerir `/project-budget-estimator` cuando la opp está en fase de presupuesto
- Avisar sobre presupuestos sin respuesta > 14 días (proponer follow-up)
- Comparar win rate personal vs objetivo del equipo (30%)

---

## Sales Manager (Director Comercial)

### Datos que ve

- **CRM**: Pipeline de **todo el equipo** comercial
- **Vendedores**: Scoring individual de cada comercial (actividad, conversión, demos)
- **Forecast**: Cobertura de pipeline vs objetivo mensual y trimestral
- **Proyectos**: No

### Valor principal

Coaching del equipo, forecast fiable, win rate saludable y gobernanza CRM limpia.

### Métricas clave

| Métrica | Fuente | Descripción |
|---------|--------|-------------|
| CRM Health Score del equipo | `crm-health-monitor` (agregado) | Puntuación global del pipeline del equipo |
| Scoring por vendedor | Actividad + conversión + demos | Ranking de rendimiento individual |
| Pipeline coverage | Valor pipeline / objetivo periodo | Ratio de cobertura (ideal >= 3x) |
| Win rate del equipo | Opps ganadas / cerradas (equipo, 90 días) | Tasa de conversión del equipo vs objetivo (30%) |
| Gobernanza CRM | Opps sin datos obligatorios / total opps | Porcentaje de opps con datos incompletos |
| Forecast accuracy | Forecast vs cierre real (mes anterior) | Precisión de las previsiones |

### Acciones típicas del Mentor

- Alertar sobre vendedores con pipeline débil (pocas opps activas o valor bajo)
- Señalar baja actividad de vendedores específicos (sin opps nuevas, sin demos, sin follow-ups)
- Reportar win rate del equipo vs objetivo (30%) y tendencia mensual
- Sugerir sprint de limpieza de datos CRM cuando gobernanza > 15%
- Alertar sobre pipeline coverage < 2x para el mes/trimestre en curso
- Identificar vendedores que necesitan coaching (win rate < 20%, pocas demos)
- Comparar rendimiento entre vendedores para detectar mejores prácticas
