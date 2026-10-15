# Tareas en Odoo

## Qué es

La creación y estructura de tareas dentro de project.project en Odoo, a partir del plan de proyecto generado. Cada tarea representa un bloque de trabajo agrupado por módulo/app.

## Cuándo aplica

Fase 3 (Pre-proyecto): Después de la planificación y antes de empezar la ejecución.

## Qué debe saber el consultor

### Creación desde el análisis

- Una tarea por módulo/app (NO una tarea por item del checklist)
- Tareas adicionales obligatorias: reuniones/formaciones y salida a producción
- El skill `project-odoo-creator` lee el análisis en Knowledge y crea las tareas automáticamente
- También crea el project.project con los campos necesarios

### Campos por tarea

| Campo | Tipo | Descripción |
|-------|------|-------------|
| name | Char | Nombre descriptivo (ej: "[APP] Contabilidad") |
| allocated_hours | Float | Horas estimadas desde el plan |
| date_start | Date | Fecha de inicio planificada |
| date_deadline | Date | Fecha de fin planificada |
| stage_id | Many2one | Etapa inicial (normalmente "Nuevo" o "En espera") |
| user_ids | Many2many | Consultor(es) asignados (no obligatorio) |
| project_id | Many2one | Proyecto al que pertenece |
| description | Html | Descripción con contexto del análisis |

### Estructura Knowledge asociada

Al crear el proyecto, se debe crear también la estructura Knowledge con los 7 sub-artículos esperados. Ver `knowledge_structure.md` para detalles.

### Tareas de migración

En proyectos con componente de migración, se crean además las siguientes tareas estándar:

| Prefijo | Nombre | Descripción |
|---------|--------|-------------|
| [INFRA] | Despliegue de servidor | Configuración de infraestructura |
| [MIG] | Migración de módulos | Instalación y adaptación de módulos |
| [MIG] | Migración de base de datos | Importación de datos históricos |
| [MIG] | Testing funcional | Verificación de funcionalidades tras migración |
| [INFRA] | Validación de infraestructura | Comprobación de entorno de producción |
| [GATE] | Gate 0 | Hito de aprobación antes del go-live |

Además de las anteriores, se añaden las tareas transversales habituales: reuniones/formaciones y soporte/salida a producción.

Referencia: https://github.com/indawsodoo/ai-claude-common/blob/master/skills/migration-assistant/references/task-template.md

### Relación tarea ↔ checklists

Cada tarea puede tener items de checklist en `xx.checklist.item` (inverso: `xx_checklist_ids`):
- Se crean desde el análisis funcional (categorías CONFIG, STUDIO, DEV, etc.)
- Todas las tareas deben tener al menos un checklist
- Los checklists tienen pesos (1-5) que determinan la prioridad

## Qué debe hacer el consultor

1. Toda comunicación interna sobre la tarea y cambios debe ser registrada en el chatter de la tarea
2. Ejecutar `project-odoo-creator` con el plan como input
2. Verificar que se crearon todas las tareas esperadas
3. Verificar campos obligatorios (horas, deadlines, asignados)
4. Verificar que se creó la estructura Knowledge
5. Crear checklists si el skill no los creó automáticamente

## Skills relacionados

- `project-odoo-creator`: Crea el proyecto y tareas en Odoo a partir del análisis en Knowledge
- `project-plan`: Genera el plan de horas y fechas
- `odoo-configurator`: Ejecuta los checklists de cada tarea
- `client-request-manager (GENERAR)`: Analiza todas las tareas al inicio del proyecto y genera las peticiones necesarias

## Errores comunes

1. **Crear tareas sin horas estimadas**: Sin allocated_hours, el health monitor no puede calcular correctamente
2. **No asignar fechas de inicio y fin**: Tareas sin fechas generan warning en la auditoría
3. **No vincular Knowledge**: El proyecto queda sin documentación estructurada
4. **Crear tareas demasiado granulares**: Debe ser una tarea por módulo, no por funcionalidad individual
5. **No crear checklists**: Todas las tareas deben tener al menos un checklist
