# Modelos Odoo - Referencia Completa

Documentación detallada de todos los modelos y campos de Odoo utilizados en el skill dev-assistant.

---

## Índice de Modelos

| Modelo | Descripción | Script principal |
|--------|-------------|------------------|
| [hr.employee](#hremployee) | Empleados | `get_employee.sh`, `get_team_employees.sh` |
| [crm.team](#crmteam) | Equipos de trabajo | `get_teams.sh` |
| [account.analytic.line](#accountanalyticline) | Partes de hora | `get_timesheets.sh`, `get_project_timesheets.sh` |
| [project.project](#projectproject) | Proyectos | `odoo_call.sh` |
| [project.project.hour.line](#projectprojecthourline) | Asignación de horas | `get_project_allocations.sh` |
| [project.task](#projecttask) | Tareas | `get_task_details.sh` |
| [hr.leave](#hrleave) | Ausencias/Vacaciones | `get_leaves.sh` |
| [planning.slot](#planningslot) | Planificación semanal | `get_planning_slots.sh` |
| [mail.message](#mailmessage) | Mensajes del chatter | `get_task_details.sh` |

---

## hr.employee

**Descripción**: Empleados de la empresa. Es el modelo central para identificar quién imputa horas.

**Script**: `get_employee.sh`, `get_team_employees.sh`

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único del empleado | Siempre. Es la clave para relacionar con otros modelos |
| `name` | Char | Nombre completo | Para mostrar en informes |
| `department_id` | Many2one → `hr.department` | Departamento al que pertenece | **CRÍTICO**: Filtrar por `department_id = 4` para solo Desarrollo |
| `x_studio_team_id` | Many2one → `crm.team` | Equipo de trabajo (Tokyo, Monaco, etc.) | Para informes de equipo. Usar ID, no nombre |
| `effective_hours_pct` | Float (0-1) | Objetivo de horas facturables del empleado | Para calcular estado (✅🟡🔴) comparando con % real |
| `work_email` | Char | Email corporativo | Para buscar empleado por email |
| `active` | Boolean | Si el empleado está activo | Filtrar `active = true` siempre |
| `resource_calendar_id` | Many2one | Calendario laboral | Para calcular horas laborables esperadas |

### Filtros obligatorios

```python
# Solo empleados de Desarrollo activos
[["department_id", "=", 4], ["active", "=", true]]

# Excluir CTOs
[["id", "not in", [194, 193]]]

# Solo empleados con objetivo de facturabilidad
[["effective_hours_pct", ">", 0]]
```

### Relaciones

```
hr.employee
    ├── department_id → hr.department (Desarrollo = 4)
    ├── x_studio_team_id → crm.team (Tokyo = 28, Monaco = 33, etc.)
    └── resource_calendar_id → resource.calendar
```

---

## crm.team

**Descripción**: Equipos de trabajo. Los empleados pertenecen a un equipo vía `x_studio_team_id`.

**Script**: `get_teams.sh`

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único del equipo | Para filtrar empleados por equipo |
| `name` | Char | Nombre del equipo | Para mostrar en informes y buscar por nombre |
| `active` | Boolean | Si el equipo está activo | Filtrar equipos activos |

### Equipos conocidos

| ID | Nombre | Descripción |
|----|--------|-------------|
| 1 | Indaws | Equipo general |
| 24 | Marketing | Marketing (NO Desarrollo) |
| 28 | Tokyo | Equipo de desarrollo |
| 29 | Roma | Equipo de desarrollo |
| 30 | Paris | Equipo de desarrollo |
| 31 | Programación | Equipo de desarrollo |
| 32 | Servicios | Equipo de servicios/soporte |
| 33 | Mónaco | Equipo de desarrollo |

### Uso típico

```bash
# Usuario dice: "informe de Tokyo"
# Paso 1: Buscar ID del equipo
./scripts/get_teams.sh Tokyo  # → [{"id": 28, "name": "Tokyo"}]

# Paso 2: Usar ID para obtener empleados
./scripts/get_team_employees.sh 28
```

---

## account.analytic.line

**Descripción**: Partes de hora. Cada registro es una imputación de horas de un empleado a un proyecto/tarea.

**Script**: `get_timesheets.sh`, `get_project_timesheets.sh`

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único del parte | Identificador |
| `date` | Date | Fecha del parte | Filtrar por periodo (date >= X AND date <= Y) |
| `employee_id` | Many2one → `hr.employee` | Empleado que imputó | Para agrupar por empleado o filtrar por Desarrollo |
| `project_id` | Many2one → `project.project` | Proyecto | Para agrupar por proyecto |
| `task_id` | Many2one → `project.task` | Tarea (puede ser `false`) | Para desglose por tarea. Si es `false` → "Sin tarea asignada" |
| `unit_amount` | Float | Horas totales imputadas | Sumar para total de horas |
| `billable_unit_amount` | Float | Horas facturables | Para calcular % facturable |
| `unbillable_unit_amount` | Float | Horas NO facturables | Para detectar excesos y alarmas |
| `x_studio_bonus_unit_amount` | Float | Horas en tiempo (para bonus) | Horas facturables imputadas a tiempo. Si se imputa tarde → penalización 0.7x |
| `name` | Char | Descripción del trabajo | Para detalle de horas no facturables |
| `holiday_id` | Many2one | Ausencia relacionada | **Excluir si != false** (partes automáticos de vacaciones) |
| `global_leave_id` | Many2one | Festivo relacionado | **Excluir si != false** (partes automáticos de festivos) |

### Filtros obligatorios

```python
# Excluir partes automáticos de vacaciones/festivos
[["holiday_id", "=", false], ["global_leave_id", "=", false]]

# Filtrar por periodo
[["date", ">=", "2026-01-01"], ["date", "<=", "2026-01-31"]]

# Filtrar por empleado
[["employee_id", "=", EMPLOYEE_ID]]

# Filtrar por proyecto
[["project_id", "=", PROJECT_ID]]
```

### Cálculos comunes

```python
# Porcentaje facturable
pct_facturable = (sum(billable_unit_amount) / sum(unit_amount)) * 100

# Porcentaje de horas en tiempo (para bonus)
# x_studio_bonus_unit_amount = horas facturables imputadas dentro del plazo
pct_en_tiempo = (sum(x_studio_bonus_unit_amount) / sum(billable_unit_amount)) * 100

# Estado vs objetivo individual
objetivo = hr.employee.effective_hours_pct * 100
if pct_facturable >= objetivo:
    estado = "✅ OK"
elif pct_facturable >= objetivo - 10:
    estado = "🟡 AVISO"
else:
    estado = "🔴 CRÍTICO"
```

### Horas en tiempo (bonus)

Las horas "en tiempo" (`x_studio_bonus_unit_amount`) son las horas facturables que se imputaron dentro del plazo establecido. Si un parte de horas se imputa con retraso, Odoo aplica una penalización de 0.7x a esas horas para el cálculo del bonus.

**Formato de visualización:**
- En tabla: columna separada "En tiempo" con formato HH:MM
- En detalle: inline con cursiva: `HH:MM fact. *(HH:MM en tiempo)*`

---

## project.project

**Descripción**: Proyectos. Contiene información básica del proyecto.

**Script**: `odoo_call.sh` (llamada directa)

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único del proyecto | Identificador. Formato en informes: `#ID - Nombre` |
| `name` | Char | Nombre del proyecto | Para mostrar en informes |
| `partner_id` | Many2one → `res.partner` | Cliente | Para mostrar cliente en informe de proyecto |
| `date_start` | Date | Fecha inicio | Información del proyecto |
| `date` | Date | Fecha fin prevista | Información del proyecto |
| `stage_id` | Many2one → `project.project.stage` | Etapa (Done, In Progress, etc.) | Para detectar proyectos cerrados con horas nuevas |
| `x_studio_crm_team_id` | Many2one → `crm.team` | Equipo propietario del proyecto | **Clave**: Para filtrar proyectos por equipo (Tokyo=28, Monaco=33, etc.) |
| `x_studio_fecha_prevista_salida_produccin` | Date | Fecha prevista de salida a producción | **Clave para informe de planificación**: Detectar proyectos próximos a salir a producción |

### Campo x_studio_crm_team_id

Este campo indica a qué equipo pertenece el proyecto. Se usa para:
- Filtrar proyectos por equipo en informes de planificación
- Asegurar que solo se muestran releases del equipo correcto
- Filtrar planificaciones de horas (`project.project.plan.hour.line`) por proyectos del equipo

```python
# Proyectos del equipo Tokyo (id=28)
[["x_studio_crm_team_id", "=", 28], ["active", "=", true]]
```

**Script**: `get_team_projects.sh`

### Campo x_studio_fecha_prevista_salida_produccin

Este campo indica la fecha planificada para poner el proyecto en producción. Se usa en el informe de preparación de planificación para:
- Listar proyectos con salida a producción en los próximos 30 días
- Detectar conflictos con vacaciones del equipo
- Alertar de riesgos cuando hay vacaciones cerca de la fecha de salida

```python
# Proyectos con salida a producción en los próximos 30 días (de un equipo específico)
[
    ["x_studio_crm_team_id", "=", TEAM_ID],
    ["x_studio_fecha_prevista_salida_produccin", ">=", "HOY"],
    ["x_studio_fecha_prevista_salida_produccin", "<=", "HOY+30"],
    ["active", "=", true]
]
```

### Proyectos a excluir en informes

| ID | Nombre | Razón |
|----|--------|-------|
| 1 | Interno | Proyecto interno, se muestra aparte |
| 100 | INDAWS | Proyecto interno |
| 279 | INDAWS FORMACIONES | Proyecto interno |

---

## project.project.hour.line

**Descripción**: Asignación de horas por proyecto y departamento. Define cuántas horas tiene asignadas cada departamento en un proyecto.

**Script**: `get_project_allocations.sh`

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único | Identificador |
| `project_id` | Many2one → `project.project` | Proyecto | Para filtrar por proyecto |
| `department_id` | Many2one → `hr.department` | Departamento | **Prioridad**: Buscar `department_id = 4` (Desarrollo), si no existe usar `department_id = false` (global) |
| `hours` | Float | Horas asignadas | Total de horas presupuestadas para el departamento |
| `total_billable_hours` | Float | Horas facturables consumidas | Acumulado histórico |
| `total_unbillable_hours` | Float | Horas NO facturables consumidas | **Exceso histórico**. Si > 0, el proyecto está en negativo |
| `total_hours_available` | Float | Horas disponibles | Si <= 0, proyecto agotado |

### Lógica de selección de línea

```python
# Para un proyecto, buscar asignación:
# 1. Primero buscar línea de Desarrollo
linea = search([["project_id", "=", PROJECT_ID], ["department_id", "=", 4]])

# 2. Si no existe, buscar línea global
if not linea:
    linea = search([["project_id", "=", PROJECT_ID], ["department_id", "=", false]])

# 3. Si no existe ninguna → "Sin asignación" 🔴
```

### Cálculo de estado del proyecto

```python
if not linea:
    estado = "🔴 Sin asignación"
else:
    pct_exceso = (total_unbillable_hours / hours) * 100
    if pct_exceso > 30:
        estado = f"🔴 -{pct_exceso:.0f}%"
    elif pct_exceso > 0:
        estado = f"🟡 -{pct_exceso:.0f}%"
    else:
        estado = "✅ OK"
```

---

## project.project.plan.hour.line

**Descripción**: Planificación de horas por empleado y proyecto. Define cuántas horas tiene planificadas cada empleado para un proyecto en un periodo específico. Se usa para detectar conflictos entre vacaciones y entregas.

**Script**: `get_employee_project_planning.sh`

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único | Identificador |
| `plan_id` | Many2one → `project.project.plan.hour` | Plan padre | Relación con el plan general |
| `project_id` | Many2one → `project.project` | Proyecto | Para filtrar por proyecto |
| `employee_id` | Many2one → `hr.employee` | Empleado asignado | **Clave**: Para detectar conflictos con vacaciones. Si es `false`, la planificación es para todo el departamento |
| `department_id` | Many2one → `hr.department` | Departamento | Filtrar por `department_id = 4` (Desarrollo) |
| `date_start` | Date | Fecha inicio del periodo | Para filtrar planificaciones activas |
| `date_end` | Date | Fecha fin del periodo | Para detectar solapamiento con vacaciones |
| `hours` | Float | Horas planificadas | Total de horas asignadas al empleado para este proyecto en el periodo |
| `hours_spent` | Float | Horas consumidas (computed) | Horas ya imputadas en el periodo |
| `remaining_hours` | Float | Horas restantes (computed) | `hours - hours_spent`. Si > 0, hay trabajo pendiente |
| `name` | Char | Descripción | Información adicional |

### Uso en detección de conflictos vacaciones-producción

```python
# Buscar planificaciones de un empleado que solapen con sus vacaciones
# Ejemplo: Empleado 183 tiene vacaciones del 11/02 al 13/02

planificaciones_afectadas = search([
    ["employee_id", "=", 183],
    ["date_start", "<=", "2026-02-13"],  # Fin vacaciones
    ["date_end", ">=", "2026-02-11"],    # Inicio vacaciones
    ["remaining_hours", ">", 0]          # Con trabajo pendiente
])

# Para cada planificación, verificar si el proyecto tiene salida a producción próxima
for plan in planificaciones_afectadas:
    proyecto = plan.project_id
    if proyecto.x_studio_fecha_prevista_salida_produccin:
        fecha_salida = proyecto.x_studio_fecha_prevista_salida_produccin
        # Si la salida es en los 7 días posteriores a las vacaciones → ALERTA
```

### Estados de riesgo por vacaciones

| Condición | Estado |
|-----------|--------|
| Vacaciones en los 3 días previos a salida producción | 🔴 **Crítico** |
| Vacaciones en los 7 días previos a salida producción | 🟡 **En riesgo** |
| Sin conflicto | ✅ **OK** |

### Relaciones

```
project.project.plan.hour.line
    ├── project_id ────────→ project.project
    │                              └── x_studio_fecha_prevista_salida_produccin (fecha salida)
    ├── employee_id ───────→ hr.employee
    │                              └──← hr.leave (vacaciones)
    └── department_id ─────→ hr.department
```

---

## project.task

**Descripción**: Tareas de un proyecto. Contiene información de planificación y ejecución.

**Script**: `get_task_details.sh`

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único | Identificador. Formato: `#ID - Nombre` |
| `name` | Char | Nombre de la tarea | Para mostrar en informes |
| `project_id` | Many2one → `project.project` | Proyecto padre | Para agrupar tareas por proyecto |
| `stage_id` | Many2one → `project.task.type` | Etapa actual | Para mostrar estado de la tarea |
| `priority` | Selection (0/1) | Prioridad | 0 = Normal, 1 = Alta |
| `date_deadline` | Date | Fecha límite | Para mostrar en info de tarea |
| `planned_hours` | Float | Horas planificadas/estimadas | Para calcular desviación |
| `effective_hours` | Float | Horas reales (computed) | Total de horas imputadas en la tarea |
| `remaining_hours` | Float | Horas restantes | `planned_hours - effective_hours`. Si < 0, se excedió |
| `description` | Html | Descripción detallada | Para mostrar contexto. Limpiar HTML |
| `user_ids` | Many2many → `res.users` | Usuarios asignados | Para mostrar responsables |
| `x_studio_review_user_id` | Many2one → `res.users` | Usuario que revisa/asigna | Contacto si falta información |

### Cálculo de desviación

```python
desviacion = effective_hours - planned_hours

if planned_hours == 0:
    estado = "⚠️ Sin estimar"
elif desviacion / planned_hours > 0.30:
    estado = f"🔴 +{(desviacion/planned_hours)*100:.0f}%"
elif desviacion / planned_hours > 0:
    estado = f"🟡 +{(desviacion/planned_hours)*100:.0f}%"
else:
    estado = "✅ OK"
```

---

## hr.leave

**Descripción**: Ausencias y vacaciones de empleados.

**Script**: `get_leaves.sh`

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único | Identificador |
| `employee_id` | Many2one → `hr.employee` | Empleado | Para filtrar ausencias de un empleado |
| `date_from` | Datetime | Inicio de ausencia | Para calcular días en el periodo |
| `date_to` | Datetime | Fin de ausencia | Para calcular días en el periodo |
| `number_of_days` | Float | Días de ausencia | Para restar de días laborables |
| `holiday_status_id` | Many2one → `hr.leave.type` | Tipo de ausencia | Vacaciones, enfermedad, etc. |
| `state` | Selection | Estado | **Solo usar `state = 'validate'`** (aprobadas) |

### Cálculo de horas laborables

```python
# Días laborables en el periodo (L-V)
dias_laborables = contar_dias_laborables(fecha_inicio, fecha_fin)

# Restar ausencias aprobadas
horas_ausencia = sum(number_of_days) * 8  # 8h por día

# Horas laborables esperadas
horas_laborables = (dias_laborables * 8) - horas_ausencia
```

---

## planning.slot

**Descripción**: Slots de planificación semanal. Define qué tareas tiene asignadas un empleado y si las ha completado.

**Scripts**: `get_planning_slots.sh`, `get_team_pending_slots.sh`

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único | Identificador |
| `name` | Char | Nombre/descripción del slot | Para mostrar información adicional |
| `employee_id` | Many2one → `hr.employee` | Empleado asignado | Para filtrar por empleado |
| `start_datetime` | Datetime | Inicio del slot | Para filtrar por semana y mostrar cuándo |
| `end_datetime` | Datetime | Fin del slot | Para calcular duración |
| `allocated_hours` | Float | Horas planificadas | Horas asignadas para completar la tarea |
| `project_id` | Many2one → `project.project` | Proyecto | Para contexto y agrupación |
| `x_studio_done` | Boolean | Si está completado | **Clave para informe**: `false` = tarea pendiente |
| `state` | Selection | Estado del slot | `draft`, `published`, `done` |
| `role_id` | Many2one → `planning.role` | Rol del empleado | Opcional, para filtrar por tipo de trabajo |
| `company_id` | Many2one → `res.company` | Compañía | Multicompañía |

### Campos computados/relacionados útiles

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `allocated_percentage` | Float | Porcentaje de tiempo asignado |
| `working_days_count` | Integer | Días laborables del slot |

### Estados del slot (state)

| Valor | Descripción |
|-------|-------------|
| `draft` | Borrador (no publicado) |
| `published` | Publicado (visible para empleado) |
| `done` | Marcado como completado por el sistema |

### Campo x_studio_done

**IMPORTANTE**: Este campo indica si el empleado ha marcado la tarea como completada en la planificación.

- `x_studio_done = false` → Tarea NO completada (pendiente)
- `x_studio_done = true` → Tarea completada

**Uso en informe de preparación semanal:**
```python
# Tareas planificadas semana anterior NO completadas
[
    ["employee_id", "in", [IDS_EMPLEADOS_EQUIPO]],
    ["start_datetime", ">=", "LUNES_SEMANA_ANTERIOR"],
    ["start_datetime", "<", "LUNES_ESTA_SEMANA"],
    ["x_studio_done", "=", false]
]
```

### Filtros comunes

```python
# Slots de un empleado en una semana
[
    ["employee_id", "=", EMPLOYEE_ID],
    ["start_datetime", ">=", "2026-01-27"],  # Lunes
    ["start_datetime", "<", "2026-02-03"]    # Siguiente lunes
]

# Slots pendientes (no completados) de un equipo
[
    ["employee_id", "in", [IDS_EMPLEADOS]],
    ["start_datetime", ">=", "2026-01-27"],
    ["start_datetime", "<", "2026-02-03"],
    ["x_studio_done", "=", false]
]

# Solo slots publicados
[["state", "=", "published"]]
```

### Relaciones

```
planning.slot
    ├── employee_id ────────→ hr.employee
    └── project_id ─────────→ project.project
```

---

## mail.message

**Descripción**: Mensajes del chatter de cualquier modelo. Se usa para obtener comentarios de tareas.

**Script**: `get_task_details.sh` (con `include_chatter = true`)

### Campos

| Campo | Tipo | Descripción | Cuándo usar |
|-------|------|-------------|-------------|
| `id` | Integer | ID único | Identificador |
| `model` | Char | Modelo relacionado | Filtrar por `model = 'project.task'` |
| `res_id` | Integer | ID del registro | Filtrar por `res_id = TASK_ID` |
| `body` | Html | Contenido del mensaje | Limpiar HTML para mostrar |
| `author_id` | Many2one → `res.partner` | Autor | Para mostrar quién escribió |
| `date` | Datetime | Fecha del mensaje | Para ordenar cronológicamente |
| `message_type` | Selection | Tipo de mensaje | **Solo incluir**: `comment`, `email`. **Excluir**: notificaciones automáticas |

### Filtros para chatter útil

```python
# Solo comentarios y emails, no notificaciones automáticas
[
    ["model", "=", "project.task"],
    ["res_id", "=", TASK_ID],
    ["message_type", "in", ["comment", "email"]]
]
```

---

## Relaciones entre Modelos

```
hr.employee
    │
    ├── x_studio_team_id ──────────────────→ crm.team
    │
    ├── department_id ─────────────────────→ hr.department (Desarrollo = 4)
    │
    └──← employee_id ──────────────────────── account.analytic.line (partes)
                                                    │
                                                    ├── project_id ────→ project.project
                                                    │                         │
                                                    │                         └──← project_id ── project.project.hour.line
                                                    │
                                                    └── task_id ───────→ project.task
                                                                              │
                                                                              └──← res_id ────── mail.message
```

---

## Formato de Horas

**OBLIGATORIO**: Siempre mostrar horas en formato **HH:MM**, nunca en decimales.

```python
def formato_horas(decimal):
    horas = int(decimal)
    minutos = round((decimal - horas) * 60)
    return f"{horas:02d}:{minutos:02d}"

# Ejemplos:
# 2.5    → 02:30
# 8.75   → 08:45
# 125.25 → 125:15
```

---

## Resumen de Scripts y Modelos

| Script | Modelos que consulta |
|--------|---------------------|
| `get_employee.sh` | `hr.employee` |
| `get_teams.sh` | `crm.team` |
| `get_team_employees.sh` | `hr.employee` (filtrado por `crm.team`) |
| `get_team_projects.sh` | `project.project` (filtrado por `x_studio_crm_team_id`) |
| `get_timesheets.sh` | `account.analytic.line` |
| `get_team_timesheets.sh` | `account.analytic.line` (filtrado por equipo) |
| `get_project_timesheets.sh` | `account.analytic.line` |
| `get_leaves.sh` | `hr.leave` |
| `get_team_leaves.sh` | `hr.leave` (filtrado por equipo) |
| `get_project_allocations.sh` | `project.project.hour.line` |
| `get_task_details.sh` | `project.task`, `mail.message` |
| `get_planning_slots.sh` | `planning.slot` |
| `get_team_pending_slots.sh` | `planning.slot` (filtrado por equipo, x_studio_done=false) |
| `get_employee_project_planning.sh` | `project.project.plan.hour.line` |
| `get_upcoming_releases.sh` | `project.project` (filtrado por fecha salida y equipo) |
