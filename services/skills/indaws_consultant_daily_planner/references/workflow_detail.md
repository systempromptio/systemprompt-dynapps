# Consultant Daily Planner - Workflow Detail

## Paso 1: Extraccion de Datos

### 1a. Identificar proyectos activos

**Modo A** -- Consultar `project.project.plan.hour.line`:

```
Modelo: project.project.plan.hour.line
Dominio: [
  ["employee_id", "=", <employee_id>],
  ["date_start", "<=", "<fin_mes_actual>"],
  ["date_end", ">=", "<inicio_mes_actual>"],
  ["hours", ">", 0]
]
Campos: project_id, hours, employee_id
```

**Modo B** -- El usuario proporciona el ID del proyecto directamente.

### 1b. Por cada proyecto, extraer 7 conjuntos de datos

Ejecutar consultas via `odoo-pilot` para cada proyecto activo:

| # | Modelo | Campos clave | Filtro |
|---|--------|-------------|--------|
| 1 | `project.project` | name, partner_id, date_start, date, x_studio_knowledge, user_id | `id = project_id` |
| 2 | `project.task` | name, planned_date_begin, date_deadline, stage_id, tag_ids, description | `project_id + active=true` |
| 3 | `xx.checklist.item` | name, task_id, weight, date_deadline, date_done, type, state | `task_id.project_id + state != done` (incluye pending, in_progress y waiting) |
| 4 | `x_client_request` | x_name, x_task_id, x_state, x_deadline, x_type, x_priority, x_is_overdue, x_has_response | `project_id` |
| 5 | `project.update` | date, x_studio_last_health_check, x_studio_health_score, x_studio_health_status | `project_id, order desc, limit 1` |
| 6 | `helpdesk.ticket` | name, stage_id, create_date, user_id, partner_id, priority | `project_id + stage no cerrado (fold=false)` |
| 7 | `knowledge.article` | name, id | `parent_id = x_studio_knowledge` |

### 1c. Verificar estructura Knowledge

Para cada proyecto, consultar `references/estructura_knowledge.md` y verificar que la estructura de articulos es correcta.

## Paso 2: Analisis y Priorizacion

Consultar `references/reglas_priorizacion.md` para clasificar las acciones en 3 niveles.

### 2a. Detectar acciones URGENTES

- Client requests vencidos >3 dias: `x_state=sent AND x_deadline < hoy-3`
- Tareas con deadline hoy o vencidas: `date_deadline <= hoy` con checklists incompletos
- Tickets abiertos >24h: `helpdesk.ticket` con `create_date < hoy-1` y stage abierto
- Reunion esta semana sin preparar
- Reunion pasada sin acta: verificar si existe articulo en "Actas de reunion" en Knowledge
- Viernes: Si hoy es viernes, `project-weekly-update` es URGENTE para cada proyecto

### 2b. Detectar acciones IMPORTANTES

- Health score bajo: `x_studio_health_score < 60`
- Weekly update antiguo: `x_studio_last_health_check < hoy-7`
- Client requests draft proximos: `x_state=draft AND x_deadline <= hoy+10` Y tarea ya ha empezado
- Client requests con respuesta: `x_state=sent AND x_has_response=true` sin procesar
- Formacion proxima: Checklist "Formacion:" con fecha <= hoy+2
- Desarrollo entregado: Tarea [DEV] con checklists completados sin validacion

### 2c. Detectar acciones PLANIFICADAS

- Validaciones pendientes de documentar
- Checklists que vencen esta semana
- Dev-assessments por preparar
- Go-Live proximo (<=30 dias)
- Importaciones pendientes (data_file con respuesta)
- Estructura Knowledge incompleta

### 2d. Reglas de contexto cruzado

Consultar `references/reglas_priorizacion.md` seccion "Reglas de Contexto Cruzado":
- Proyectos con health score < 40 -> todas sus acciones suben un nivel
- Go-Live en <14 dias -> acciones del proyecto suben un nivel
- Reunion manana -> preparacion sube a URGENTE

### 2e. Patrones del dia

Consultar `references/patrones_dia.md` para reglas del dia de la semana actual.

## Paso 3: Generacion de Agenda

Generar agenda siguiendo template `templates/agenda_diaria.md`.

### 3a. Construir agenda

Para cada accion, incluir:
1. **Proyecto** entre corchetes
2. **Titulo descriptivo** del problema/accion
3. **Tiempo estimado** (consultar tabla en reglas_priorizacion.md)
4. **Contexto**: que se ha detectado y por que es importante
5. **IDs concretos**: request #X, tarea #Y, ticket #Z, checklist #W
6. **Skill a ejecutar**: nombre del skill + modo + parametros
7. **Prerequisitos**: si necesita otro skill primero (consultar vinculacion_skills.md)

### 3b. Incluir verificacion Knowledge

Tabla resumen con estado de la estructura Knowledge de cada proyecto.

### 3c. Incluir resumen de horas (si disponible)

Tabla con horas planificadas vs consumidas por proyecto este mes.

### 3d. Presentar agenda al consultor

La agenda es informativa. El consultor decide que ejecutar.
Preguntar: "Quieres que ejecute alguna de estas acciones? Indica el numero y arrancamos."
