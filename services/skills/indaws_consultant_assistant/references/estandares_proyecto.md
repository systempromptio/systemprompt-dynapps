# Estándares de Proyecto

## Qué es

Los 12 estándares de calidad que todo proyecto Odoo de Indaws debe cumplir. Cada estándar tiene una severidad (BLOCKING o WARNING), un modelo Odoo asociado, la lógica de validación y la remediación recomendada.

## Cuándo aplica

- Al iniciar la fase de ejecución (verificación inicial)
- Durante el seguimiento semanal (detección temprana de problemas)
- Antes del Go-Live (auditoría final)
- Cuando el health score es bajo y se necesita diagnosticar causas

## Qué debe saber el consultor

### Tabla de Estándares

| # | Estándar | Severidad | Modelo | Lógica pass/fail |
|---|----------|-----------|--------|-------------------|
| 1 | partner_id asignado | BLOCKING | project.project | partner_id != false |
| 2 | Knowledge configurado | BLOCKING | project.project | x_studio_knowledge != false |
| 3 | Estructura Knowledge completa | WARNING | knowledge.article | Hijos del artículo raíz contienen los 7 nombres esperados |
| 4 | Tareas ≥5h tienen checklists | WARNING | project.task + xx.checklist.item | Tareas con allocated_hours ≥ 5 deben tener al menos 1 checklist |
| 5 | Tareas activas tienen deadline | WARNING | project.task | Tareas activas (no cancelled/done) deben tener date_deadline |
| 6 | Sin tareas vencidas sin acción | WARNING | project.task | Tareas con date_deadline < hoy deben tener timesheet o message reciente |
| 7 | Client requests generadas | WARNING | x_client_request | Deben existir requests para tareas activas del proyecto |
| 8 | Health check ejecutado | WARNING | project.update | Debe existir al menos 1 update con x_studio_health_score != false |
| 9 | Weekly updates recientes | WARNING | project.update | Último update con fecha < 14 días |
| 10 | Timesheets registrados | WARNING | account.analytic.line | Existen timesheets en los últimos 7 días para el proyecto |
| 11 | Sin checklists duplicados | WARNING | xx.checklist.item | No hay duplicados por (name, task_id) |
| 12 | Horas tareas ≈ horas proyecto | WARNING | project.task vs project.project | SUM(allocated_hours) vs total_hours (tolerancia ±10%) |

### Severidades

- **BLOCKING**: El proyecto NO puede considerarse correctamente configurado. Debe corregirse antes de cualquier otra acción.
- **WARNING**: El proyecto funciona pero tiene riesgos de calidad. Debe corregirse lo antes posible.

### Estructura Knowledge esperada (Estándar #3)

Los 7 artículos hijos esperados bajo el artículo raíz del proyecto:
1. Kick-Off (opcional pero esperado)
2. Análisis (obligatorio)
3. Cuestionarios de Análisis (opcional)
4. Actas de reunión (obligatorio)
5. Desarrollos (condicional: si hay tareas [DEV])
6. Adaptaciones (condicional: si hay items STUDIO en checklists)
7. Plan de Formaciones (condicional: si hay tarea de formación)

### Tolerancias y umbrales

- **Estándar 6 (tareas vencidas)**: Se considera "sin acción" si no hay timesheet ni message_ids en los últimos 7 días
- **Estándar 9 (weekly updates)**: Umbral de 14 días (2 semanas sin update = warning)
- **Estándar 10 (timesheets)**: Umbral de 7 días (1 semana sin registrar = warning)
- **Estándar 12 (horas)**: Tolerancia ±10% entre suma de tareas y total del proyecto

## Qué debe hacer el consultor

1. Ejecutar la auditoría al menos una vez al mes por proyecto
2. Corregir los BLOCKING inmediatamente
3. Planificar la corrección de WARNING en la agenda semanal
4. Usar los skills sugeridos para cada remediación

## Skills relacionados

| Estándar | Skill de remediación |
|----------|---------------------|
| 1 (partner_id) | — (corrección manual en Odoo) |
| 2 (Knowledge) | project-odoo-creator |
| 3 (estructura Knowledge) | — (crear artículos manualmente o con el skill correspondiente) |
| 4 (checklists) | odoo-configurator |
| 5 (deadlines) | project-health-fixer |
| 6 (tareas vencidas) | project-health-fixer |
| 7 (client requests) | client-request-manager |
| 8 (health check) | project-health-monitor |
| 9 (weekly updates) | project-weekly-update |
| 10 (timesheets) | — (registro manual por el consultor) |
| 11 (duplicados) | — (limpieza manual) |
| 12 (horas) | project-health-fixer |

## Errores comunes

1. **Ignorar los BLOCKING**: Un proyecto sin partner_id o sin Knowledge configurado genera problemas en cascada
2. **No ejecutar auditorías periódicas**: Los problemas se acumulan y se detectan tarde
3. **Corregir WARNING sin priorizar**: Los BLOCKING siempre van primero
4. **No usar los skills de remediación**: Corregir manualmente lo que un skill puede hacer mejor
5. **Auditar solo antes del Go-Live**: La auditoría debe ser continua, no un evento puntual
