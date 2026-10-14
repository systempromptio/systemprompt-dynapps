# Informe de Horas de Proyecto

Documentación técnica para el informe de desglose de horas y tareas de un proyecto.

## ⚠️ RESTRICCIÓN CRÍTICA: SOLO DESARROLLO

**Este informe muestra EXCLUSIVAMENTE horas de empleados del departamento de Desarrollo (`department_id = 4`).**

Aunque otros departamentos (Consultoría, Diseño, etc.) hayan imputado horas en el proyecto, este informe SOLO muestra las de Desarrollo.

---

## Modelos Odoo

### project.project (Proyecto)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | Integer | ID del proyecto |
| `name` | Char | Nombre del proyecto |
| `partner_id` | Many2one | Cliente |
| `date_start` | Date | Fecha inicio |
| `date` | Date | Fecha fin prevista |
| `stage_id` | Many2one | Etapa del proyecto |

### project.project.hour.line (Asignación de horas)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `project_id` | Many2one | Proyecto |
| `department_id` | Many2one | Departamento (4 = Desarrollo, false = global) |
| `hours` | Float | Horas asignadas |
| `total_billable_hours` | Float | Horas facturables consumidas |
| `total_unbillable_hours` | Float | Horas no facturables (exceso histórico) |
| `total_hours_available` | Float | Horas disponibles |

### account.analytic.line (Partes de hora)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `date` | Date | Fecha del parte |
| `employee_id` | Many2one | Empleado |
| `project_id` | Many2one | Proyecto |
| `task_id` | Many2one | Tarea (puede ser false) |
| `unit_amount` | Float | Horas imputadas |
| `billable_unit_amount` | Float | Horas facturables |
| `unbillable_unit_amount` | Float | Horas no facturables |
| `name` | Char | Descripción del trabajo |

### project.task (Tarea)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | Integer | ID de la tarea |
| `name` | Char | Nombre |
| `project_id` | Many2one | Proyecto |
| `planned_hours` | Float | Horas planificadas |
| `effective_hours` | Float | Horas reales (computed) |
| `remaining_hours` | Float | Horas restantes |
| `stage_id` | Many2one | Etapa |

### hr.employee (Empleado)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | Integer | ID del empleado |
| `name` | Char | Nombre |
| `department_id` | Many2one | Departamento (4 = Desarrollo) |

---

## Consultas Necesarias

### 1. Información del proyecto

```bash
./scripts/odoo_call.sh "project.project" "search_read" \
  "[[\"id\", \"=\", PROJECT_ID]]" \
  "[\"id\", \"name\", \"partner_id\", \"date_start\", \"date\", \"stage_id\"]"
```

### 2. Asignación de horas

```bash
./scripts/get_project_allocations.sh "[PROJECT_ID]"
```

### 3. Empleados de Desarrollo

```bash
./scripts/get_team_employees.sh
```

Obtener lista de IDs para filtrar partes.

### 4. Partes de hora del proyecto

```bash
./scripts/get_project_timesheets.sh <project_id> <date_from> <date_to>
```

**Devuelve partes del proyecto en el periodo, SIN filtrar por departamento.**
El filtro por Desarrollo se aplica después comparando `employee_id` con la lista de empleados de Desarrollo.

### 5. Tareas del proyecto con horas planificadas

```bash
./scripts/odoo_call.sh "project.task" "search_read" \
  "[[\"project_id\", \"=\", PROJECT_ID]]" \
  "[\"id\", \"name\", \"planned_hours\", \"effective_hours\", \"stage_id\"]"
```

---

## ⚠️ USO OBLIGATORIO DE SCRIPTS

**SIEMPRE usar los scripts de `./scripts/` para obtener datos. NUNCA hacer llamadas directas a la API.**

| Script | Uso |
|--------|-----|
| `scripts/get_project_timesheets.sh <id> <from> <to>` | Partes del proyecto |
| `scripts/get_project_allocations.sh "[id]"` | Asignación de horas |
| `scripts/get_team_employees.sh` | Empleados de Desarrollo |

---

## Proceso de Filtrado

### Filtrar partes por Desarrollo

1. Obtener lista de empleados de Desarrollo con `get_team_employees.sh`
2. Extraer array de IDs: `[emp1_id, emp2_id, ...]`
3. Para cada parte del proyecto, verificar si `employee_id[0]` está en la lista
4. Solo procesar partes que pasen el filtro

```
empleados_desarrollo = get_team_employees.sh → extraer IDs
partes_proyecto = get_project_timesheets.sh <id> <from> <to>

partes_filtrados = []
for parte in partes_proyecto:
    if parte.employee_id[0] in empleados_desarrollo:
        partes_filtrados.append(parte)
```

---

## Cálculos

### Formato de Horas

**OBLIGATORIO**: Siempre mostrar horas en formato **HH:MM**, nunca en decimales.

```
horas = parte_entera
minutos = (parte_decimal * 60) redondeado
Ejemplo: 2.75h → 02:45
```

### Totales por Desarrollador

```
Para cada employee_id único en partes_filtrados:
    total = sum(unit_amount)
    facturables = sum(billable_unit_amount)
    no_facturables = sum(unbillable_unit_amount)
    pct_facturable = (facturables / total) * 100
```

### Totales por Tarea

```
Para cada task_id único en partes_filtrados:
    total = sum(unit_amount)
    facturables = sum(billable_unit_amount)
    no_facturables = sum(unbillable_unit_amount)
    planificadas = project.task.planned_hours
    desviacion = total - planificadas
    pct_desviacion = (desviacion / planificadas) * 100 si planificadas > 0
```

### Estado del Proyecto

```
asignacion = get_project_allocations.sh "[id]"
if asignacion is empty:
    estado = "🔴 Sin asignación"
else:
    exceso_pct = (total_unbillable_hours / hours) * 100
    if exceso_pct > 30:
        estado = "🔴 -XX%"
    elif exceso_pct > 0:
        estado = "🟡 -XX%"
    else:
        estado = "✅ OK"
```

---

## Indicadores de Alarma

### Estado del Proyecto

| Condición | Estado |
|-----------|--------|
| Sin asignación de horas | 🔴 Sin asignación |
| Exceso histórico > 30% | 🔴 **-XX%** |
| Exceso histórico 1-30% | 🟡 **-XX%** |
| Exceso histórico <= 0% | ✅ OK |

### Estado de Tarea

| Condición | Estado |
|-----------|--------|
| Sin horas planificadas | ⚠️ Sin estimar |
| Desviación > 30% | 🔴 **+XX%** |
| Desviación 1-30% | 🟡 **+XX%** |
| Desviación <= 0% | ✅ OK |

---

## Exclusiones

### Partes automáticos

Excluir registros con:
- `holiday_id != false` (partes de vacaciones)
- `global_leave_id != false` (partes de festivos)

### Empleados

Solo incluir empleados de Desarrollo (`department_id = 4`).

---

## Notas

- El informe siempre muestra horas de Desarrollo, aunque el proyecto tenga horas de otros departamentos
- Las tareas sin `task_id` se agrupan como "Sin tarea asignada"
- El exceso histórico (`total_unbillable_hours`) es un campo calculado por Odoo, no se recalcula
- Si no hay asignación de horas para Desarrollo, buscar asignación global (`department_id = false`)
