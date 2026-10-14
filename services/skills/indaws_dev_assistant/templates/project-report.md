# Template: Informe de Horas de Proyecto

Formato de salida para el desglose de horas y tareas de un proyecto específico.

## ⚠️ RESTRICCIÓN CRÍTICA: SOLO DESARROLLO

**Todo el informe muestra EXCLUSIVAMENTE horas imputadas por empleados del departamento de Desarrollo (`department_id = 4`).**

NUNCA incluir horas de empleados de otros departamentos en este informe.

---

## Cabecera

```
📊 **Proyecto #[ID] - [Nombre Proyecto]**
*Cliente: [nombre_cliente]*
*Periodo analizado: [DD/MM/YYYY] - [DD/MM/YYYY]*
*(Solo horas de Desarrollo)*
```

---

## Resumen de Horas del Proyecto

```
### 📈 Resumen de Horas

| Métrica | Valor |
|---------|-------|
| **Horas asignadas (Desarrollo)** | 65:00 |
| **Horas consumidas (periodo)** | 48:30 |
| **Horas facturables** | 44:45 (92%) |
| **Horas no facturables** | 03:45 (8%) |
| **Exceso histórico** | -178:45 |
| **Estado** | 🔴 **-275%** |
```

### Campos del resumen

| Campo | Origen | Descripción |
|-------|--------|-------------|
| Horas asignadas | `project.project.hour.line.hours` | Campo `hours` de línea con `department_id = 4` o global |
| Horas consumidas | `sum(account.analytic.line.unit_amount)` | Solo empleados de Desarrollo en el periodo |
| Horas facturables | `sum(account.analytic.line.billable_unit_amount)` | Solo empleados de Desarrollo |
| Horas no facturables | `sum(account.analytic.line.unbillable_unit_amount)` | Solo empleados de Desarrollo |
| Exceso histórico | `project.project.hour.line.total_unbillable_hours` | Acumulado histórico |

### Criterios de Estado

| Condición | Estado |
|-----------|--------|
| Sin línea de asignación | 🔴 Sin asignación |
| Exceso histórico > 30% | 🔴 **-XX%** |
| Exceso histórico 1-30% | 🟡 **-XX%** |
| Exceso histórico <= 0% | ✅ OK |

---

## Desglose por Desarrollador

**SOLO empleados del departamento de Desarrollo que han imputado horas en el proyecto.**

Ordenar por horas totales (mayor a menor).

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

**La tabla DEBE tener EXACTAMENTE estas 5 columnas. NO omitir ninguna.**

```
### 👥 Horas por Desarrollador

| Desarrollador | Total | Fact. | No Fact. | % Fact. |
|---------------|-------|-------|----------|---------|
| Juan García | 18:30 | 17:00 | 01:30 | 92% |
| María López | 15:45 | 14:30 | 01:15 | 92% |
| Pedro Ruiz | 10:15 | 09:45 | 00:30 | 95% |
| Ana Martín | 04:00 | 03:30 | 00:30 | 88% |
| **TOTAL** | **48:30** | **44:45** | **03:45** | **92%** |
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Desarrollador** | Texto | Nombre del empleado (SOLO de Desarrollo) |
| 2 | **Total** | HH:MM | `sum(unit_amount)` del empleado en el proyecto |
| 3 | **Fact.** | HH:MM | `sum(billable_unit_amount)` del empleado |
| 4 | **No Fact.** | HH:MM | `sum(unbillable_unit_amount)` del empleado |
| 5 | **% Fact.** | XX% | Porcentaje facturable sobre total |

---

## Desglose por Tarea

**SOLO tareas donde empleados de Desarrollo han imputado horas.**

Ordenar por horas totales (mayor a menor).

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

**La tabla DEBE tener EXACTAMENTE estas 7 columnas. NO omitir ninguna.**

```
### 📋 Horas por Tarea

| Tarea | Planif. | Real | Desv. | Fact. | No Fact. | Estado |
|-------|---------|------|-------|-------|----------|--------|
| #4521 - Migración datos clientes | 12:00 | 18:30 | +06:30 | 17:00 | 01:30 | 🔴 +54% |
| #4522 - API integración | 08:00 | 10:15 | +02:15 | 09:45 | 00:30 | 🟡 +28% |
| #4523 - Testing | 06:00 | 05:45 | -00:15 | 05:00 | 00:45 | ✅ OK |
| #4524 - Documentación | 04:00 | 04:00 | 00:00 | 03:30 | 00:30 | ✅ OK |
| Sin tarea asignada | - | 10:00 | - | 09:30 | 00:30 | ⚠️ |
| **TOTAL** | **30:00** | **48:30** | **+18:30** | **44:45** | **03:45** | |
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Tarea** | #ID - Nombre | `#ID - nombre` de la tarea |
| 2 | **Planif.** | HH:MM o `-` | `project.task.planned_hours` |
| 3 | **Real** | HH:MM | `sum(unit_amount)` de partes en esa tarea |
| 4 | **Desv.** | +/-HH:MM o `-` | Real - Planificadas |
| 5 | **Fact.** | HH:MM | `sum(billable_unit_amount)` |
| 6 | **No Fact.** | HH:MM | `sum(unbillable_unit_amount)` |
| 7 | **Estado** | Ver formato | Según desviación sobre planificadas |

### Criterios de Estado de Tarea

| Condición | Estado |
|-----------|--------|
| Sin horas planificadas y con horas reales | ⚠️ Sin estimar |
| Desviación > 30% | 🔴 **+XX%** |
| Desviación 1-30% | 🟡 **+XX%** |
| Desviación <= 0% | ✅ OK |

### Horas sin tarea asignada

Si hay partes de hora con `task_id = false`, mostrar una fila "Sin tarea asignada" con el total.

---

## Detalle de Horas No Facturables

**Solo si hay horas no facturables en el periodo.**

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

**La tabla DEBE tener EXACTAMENTE estas 5 columnas. NO omitir ninguna.**

```
### ⚠️ Detalle Horas No Facturables

| Fecha | Desarrollador | Tarea | Horas | Descripción |
|-------|---------------|-------|-------|-------------|
| 15/01/2026 | Juan García | #4521 - Migración | 01:00 | Corrección bug en import |
| 16/01/2026 | María López | #4522 - API | 00:45 | Refactor solicitado por cliente |
| 20/01/2026 | Pedro Ruiz | Sin tarea | 00:30 | Reunión interna proyecto |
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Origen |
|---|---------|---------|--------|
| 1 | **Fecha** | DD/MM/YYYY | `account.analytic.line.date` |
| 2 | **Desarrollador** | Texto | `account.analytic.line.employee_id[1]` |
| 3 | **Tarea** | #ID - Nombre | `account.analytic.line.task_id[1]` o "Sin tarea" |
| 4 | **Horas** | HH:MM | `account.analytic.line.unbillable_unit_amount` |
| 5 | **Descripción** | Texto | `account.analytic.line.name` |

---

## Acciones Recomendadas

```
---

💡 **Acciones recomendadas:**
1. Revisar tareas con desviación > 30% sobre planificación
2. Asignar tareas a partes sin tarea (10:00 sin asignar)
3. Solicitar ampliación de horas al PM (exceso histórico -275%)
```

---

## Proceso de Generación

1. Obtener info del proyecto: `project.project` con ID indicado
2. Obtener asignación de horas: `scripts/get_project_allocations.sh "[ID]"`
3. Obtener empleados de Desarrollo: `scripts/get_team_employees.sh`
4. Obtener partes del proyecto: `scripts/get_project_timesheets.sh <project_id> <date_from> <date_to>`
5. Filtrar partes SOLO de empleados de Desarrollo
6. Obtener tareas del proyecto con horas: agrupar partes por `task_id`
7. Para cada tarea, obtener `planned_hours` de `project.task`
8. Calcular totales y desviaciones
9. Generar informe según template

---

## Scripts Necesarios

| Script | Uso |
|--------|-----|
| `scripts/get_project_timesheets.sh <project_id> <from> <to>` | Partes de hora del proyecto |
| `scripts/get_project_allocations.sh "[project_id]"` | Asignación de horas |
| `scripts/get_team_employees.sh` | Lista de empleados de Desarrollo (para filtrar) |

---

## Notas

- **SIEMPRE filtrar** partes por empleados de Desarrollo antes de calcular totales
- Las horas planificadas de tareas pueden estar a 0 (tareas sin estimar)
- Los partes sin `task_id` deben mostrarse como "Sin tarea asignada"
- El exceso histórico viene de `project.project.hour.line`, no se calcula
