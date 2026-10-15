# Template: Informe de Preparación de Planificación Semanal

Formato de salida para el informe de preparación de planificación semanal de un equipo.

**Destinatario**: Team Leader
**Propósito**: Preparar la planificación de la semana entrante revisando la semana anterior
**Frecuencia**: Semanal (normalmente lunes por la mañana)

---

## ⚠️ PASO PREVIO OBLIGATORIO: Confirmar Fechas con el Usuario

**ANTES de generar el informe, SIEMPRE proponer fechas y pedir confirmación.**

### Cálculo de fechas por defecto

```
HOY = fecha actual
FECHA_FIN = HOY (incluye el día de hoy, las reuniones suelen ser por la tarde)
FECHA_INICIO = lunes de la semana actual = HOY - (día_de_la_semana - 1)
               donde lunes=1, martes=2, miércoles=3, jueves=4, viernes=5, sábado=6, domingo=7
```

**Ejemplo**: Si HOY es jueves 05/02/2026 (día 4 de la semana):
- FECHA_INICIO = 05 - (4-1) = 05 - 3 = **02/02/2026** (lunes)
- FECHA_FIN = **05/02/2026** (jueves)

### Mensaje a mostrar al usuario (usar AskUserQuestion)

```
📋 Preparación de Planificación - Equipo [Nombre]

Voy a generar el informe de la semana actual:
- **Desde:** [FECHA_INICIO] (lunes de esta semana)
- **Hasta:** [FECHA_FIN] (hoy)

¿Quieres usar estas fechas o prefieres indicar un rango diferente?
```

**Opciones a ofrecer:**
1. **Usar fechas propuestas** - Genera el informe con el rango calculado
2. **Indicar fechas personalizadas** - El usuario especifica fecha_inicio y fecha_fin

### Si el usuario elige fechas personalizadas

Solicitar:
- Fecha de inicio (formato DD/MM/YYYY o YYYY-MM-DD)
- Fecha de fin (formato DD/MM/YYYY o YYYY-MM-DD)

### Ejemplo de interacción

```
Usuario: "preparar planificación del equipo Monaco"

Asistente (si hoy es jueves 05/02/2026):
📋 Preparación de Planificación - Equipo Mónaco

Voy a generar el informe de la semana actual:
- **Desde:** 02/02/2026 (lunes de esta semana)
- **Hasta:** 05/02/2026 (hoy jueves)

¿Quieres usar estas fechas o prefieres indicar un rango diferente?

[Usar fechas propuestas] [Indicar fechas personalizadas]
```

---

## ⚠️ RESTRICCIÓN CRÍTICA: SOLO DESARROLLO

**TODO el informe es EXCLUSIVO para empleados del departamento de Desarrollo (`department_id = 4`).**

---

## ⛔ REGLAS DE CÁLCULO - NO ESTIMAR NUNCA

**CRÍTICO: Los errores de cálculo son INACEPTABLES. Seguir estas reglas SIN EXCEPCIÓN.**

### Campos de Odoo

| Modelo | Campo | Qué es |
|--------|-------|--------|
| `planning.slot` | `x_studio_done` | Si la tarea planificada está completada |
| `planning.slot` | `allocated_hours` | Horas planificadas para la tarea |
| `account.analytic.line` | `unit_amount` | Horas imputadas |
| `account.analytic.line` | `billable_unit_amount` | Horas facturables |
| `account.analytic.line` | `unbillable_unit_amount` | Horas no facturables |
| `hr.leave` | `number_of_days` | Días de ausencia |

### Scripts a usar

```bash
# 1. Lista de empleados del equipo
./scripts/get_team_employees.sh <team_id>
# → Obtener employee_ids para usar en otros scripts

# 2. ⭐ Proyectos del equipo (NUEVO - necesario antes de planificación y releases)
./scripts/get_team_projects.sh <team_id>
# → Obtener project_ids para filtrar planificación y releases

# 3. Tareas no completadas de la semana anterior (usa employee_ids)
./scripts/get_team_pending_slots.sh "[emp_ids]" <lunes_semana_anterior>

# 4. Partes de hora del equipo semana anterior
./scripts/get_team_timesheets.sh <team_id> <lunes_semana_anterior> <domingo_semana_anterior>

# 5. Vacaciones próximos 30 días (SOLO futuras: date_from > hoy, usa employee_ids)
./scripts/get_team_leaves.sh "[emp_ids]" <hoy+1> <hoy+30dias>

# 6. Planificación de horas por empleado (FILTRADO POR PROYECTOS DEL EQUIPO)
./scripts/get_employee_project_planning.sh "[emp_ids]" "[project_ids]" <hoy> <hoy+30dias>
# → Usar project_ids del paso 2 para filtrar solo proyectos del equipo

# 7. Proyectos con salida a producción próxima (FILTRADO POR EQUIPO)
./scripts/get_upcoming_releases.sh <team_id> <hoy> <hoy+30dias>
# → Primer parámetro team_id filtra por x_studio_crm_team_id
```

---

## Cabecera

```
📋 **Preparación Planificación - Equipo [Nombre Equipo]**
*Semana anterior: DD/MM/YYYY - DD/MM/YYYY*
*Generado: DD/MM/YYYY (día de la semana)*
```

---

## Sección 1: Tareas No Completadas

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

Lista de tareas planificadas la semana anterior que **NO están hechas** (`x_studio_done = false`), agrupadas por empleado.

```
---

### 🔴 Tareas No Completadas de la Semana Anterior

Tareas planificadas la semana pasada que **NO están hechas** (`x_studio_done = false`).

#### **[Nombre Empleado 1]**
| Tarea | Proyecto | Horas plan. | Estado tarea |
|-------|----------|-------------|--------------|
| #XXXX - Nombre tarea | Nombre proyecto | HH:MM | [Estado de project.task] |
| #XXXX - Nombre tarea | Nombre proyecto | HH:MM | [Estado de project.task] |

**Total no completado:** HH:MM

---

#### **[Nombre Empleado 2]**
| Tarea | Proyecto | Horas plan. | Estado tarea |
|-------|----------|-------------|--------------|
| #XXXX - Nombre tarea | Nombre proyecto | HH:MM | [Estado de project.task] |

**Total no completado:** HH:MM

---

#### **[Nombre Empleado 3]**
*✅ Sin tareas pendientes de la semana anterior*

---
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Tarea** | #ID - Nombre | ID y nombre de la tarea (de `x_studio_task_id`) |
| 2 | **Proyecto** | Texto | Nombre del proyecto |
| 3 | **Horas plan.** | HH:MM | Horas planificadas (`allocated_hours`) |
| 4 | **Estado tarea** | Texto | Estado actual de la tarea en `project.task` |

### Reglas

- Ordenar empleados alfabéticamente
- Dentro de cada empleado, ordenar por horas planificadas (mayor a menor)
- Si un empleado no tiene tareas pendientes, mostrar: `*✅ Sin tareas pendientes de la semana anterior*`
- El estado de la tarea se obtiene de `project.task.stage_id` relacionado con `x_studio_task_id`

---

## Sección 2: Resumen de Horas Semana Anterior

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

Tabla con el resumen de horas de cada empleado del equipo durante la semana anterior.

```
---

### ⏱️ Resumen de Horas Semana Anterior (DD/MM - DD/MM)

| Empleado | Imputadas | Fact. | En tiempo | No Fact. | % Fact. | Objetivo | Estado |
|----------|-----------|-------|-----------|----------|---------|----------|--------|
| Nombre A | HH:MM | HH:MM | HH:MM | HH:MM | XX% | XX% | ✅/🟡/🔴 |
| Nombre B | HH:MM | HH:MM | HH:MM | HH:MM | XX% | XX% | ✅/🟡/🔴 |
| **TOTAL EQUIPO** | **HH:MM** | **HH:MM** | **HH:MM** | **HH:MM** | **XX%** | - | - |
```

### Columnas OBLIGATORIAS (8 columnas)

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Empleado** | Texto | Nombre del empleado |
| 2 | **Imputadas** | HH:MM | Total horas imputadas |
| 3 | **Fact.** | HH:MM | Horas facturables |
| 4 | **En tiempo** | HH:MM | Horas en tiempo para bonus |
| 5 | **No Fact.** | HH:MM | Horas no facturables |
| 6 | **% Fact.** | XX% | Porcentaje facturable |
| 7 | **Objetivo** | XX% | Objetivo del empleado |
| 8 | **Estado** | Emoji | ✅ OK / 🟡 AVISO / 🔴 CRÍTICO |

### Criterios de Estado

| % Facturable vs Objetivo | Estado |
|--------------------------|--------|
| >= objetivo | ✅ OK |
| objetivo - 10% a objetivo | 🟡 AVISO |
| < objetivo - 10% | 🔴 CRÍTICO |

---

## Sección 3: Detalle Horas No Facturables

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

Tabla con el desglose de horas no facturables por proyecto y empleado.

```
---

### 📊 Detalle Horas No Facturables por Proyecto

| Proyecto | Empleado | No Fact. | Motivo probable |
|----------|----------|----------|-----------------|
| Proyecto A | Nombre | HH:MM | [Motivo] |
| Proyecto B | Nombre | HH:MM | [Motivo] |
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Proyecto** | Texto | Nombre del proyecto |
| 2 | **Empleado** | Texto | Nombre del empleado |
| 3 | **No Fact.** | HH:MM | Horas no facturables |
| 4 | **Motivo probable** | Texto | Motivo inferido (ver tabla abajo) |

### Motivos Probables

| Proyecto | Motivo |
|----------|--------|
| Interno | Formación / reuniones |
| Proyecto con horas agotadas | Horas agotadas en proyecto |
| Proyecto interno (no cliente) | Proyecto interno sin asignación cliente |
| Proyecto con exceso > 30% | Soporte no planificado / incidencias |
| Otros | Pendiente de revisión |

### Reglas

- Ordenar por horas no facturables (mayor a menor)
- Solo mostrar proyectos con >= 01:00 de horas no facturables
- Excluir proyecto "Interno" si < 02:00h

---

## Sección 4: Recomendaciones

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

Recomendaciones generadas automáticamente basadas en los datos.

```
---

### 💡 Recomendaciones para Mejorar Facturabilidad

1. **🔴 [Nombre Empleado] (XX% fact.):**
   - [Recomendación específica sobre proyecto/situación]
   - [Acción sugerida]

2. **🟡 [Nombre Empleado] (XX% fact.):**
   - [Recomendación específica]

3. **📋 Tareas no completadas (XX:XX acumuladas):**
   - [Recomendación sobre tareas pendientes]

4. **⚠️ Proyectos a vigilar:**
   - **[Proyecto]**: [Motivo de vigilancia]
```

### Reglas para Generar Recomendaciones

1. **Empleados en rojo (🔴)**: Siempre generar recomendación
   - Identificar proyecto con más horas no facturables
   - Sugerir acción concreta (escalar, reasignar, revisar)

2. **Empleados en amarillo (🟡)**: Generar recomendación si hay patrón claro
   - Proyecto con horas agotadas → "Escalar a PM para ampliar bolsa"
   - Muchas horas internas → "Verificar si puede reducirse formación"

3. **Tareas no completadas**: Si total > 16:00h
   - Listar las más críticas (por horas planificadas)
   - Sugerir priorización

4. **Proyectos a vigilar**: Incluir si:
   - 100% horas no facturables (posible error de configuración)
   - Horas agotadas (no planificar más trabajo)
   - Desviación > 50% sobre estimación

---

## Sección 5: Próximas Vacaciones (con conflictos de proyectos)

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

Tabla con las ausencias **futuras** del equipo en los próximos 30 días, **incluyendo proyectos afectados**.

**⚠️ SOLO mostrar vacaciones donde `date_from > HOY` (fecha inicio posterior a hoy).**
No mostrar vacaciones en curso ni pasadas.

```
---

### 🏖️ Próximas Vacaciones (próximos 30 días)

| Empleado | Fechas | Días | ⚠️ Proyectos Afectados |
|----------|--------|------|------------------------|
| Nombre A | DD/MM - DD/MM | X días | 🔴 **Proyecto X** (HH:MM planif., salida prod. DD/MM) |
| Nombre B | DD/MM | X día | Proyecto Y (HH:MM planif.), Proyecto Z (HH:MM planif.) |
| Nombre C | DD/MM - DD/MM | X días | - |

**⚠️ Conflictos críticos detectados:**
- **[Empleado] + [Proyecto]**: Vacaciones DD-DD/MM con salida a producción el DD/MM. Solo X días de margen.

**💡 Recomendaciones:**
- [Acción para mitigar el conflicto]
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Empleado** | Texto | Nombre del empleado |
| 2 | **Fechas** | DD/MM - DD/MM | Rango de fechas de vacaciones |
| 3 | **Días** | X días | Número de días |
| 4 | **⚠️ Proyectos Afectados** | Ver formato | Proyectos con horas planificadas que solapan con las vacaciones |

### Formato de Proyectos Afectados

| Condición | Formato |
|-----------|---------|
| Proyecto con salida prod. en 3 días tras vacaciones | 🔴 **Nombre** (HH:MM planif., salida prod. DD/MM) |
| Proyecto con salida prod. en 7 días tras vacaciones | 🟡 **Nombre** (HH:MM planif., salida prod. DD/MM) |
| Proyecto con horas planificadas (sin salida próxima) | Nombre (HH:MM planif.) |
| Sin proyectos afectados | `-` |

### Detección de Proyectos Afectados

Para cada vacación, buscar en `project.project.plan.hour.line`:
```python
# Empleado X tiene vacaciones del DD/MM al DD/MM
proyectos_afectados = search([
    ["employee_id", "=", EMPLOYEE_ID],
    ["date_start", "<=", FECHA_FIN_VACACIONES],
    ["date_end", ">=", FECHA_INICIO_VACACIONES],
    ["remaining_hours", ">", 0]
])

# Para cada proyecto, verificar si tiene salida a producción próxima
for plan in proyectos_afectados:
    fecha_salida = plan.project_id.x_studio_fecha_prevista_salida_produccin
    if fecha_salida:
        dias_margen = (fecha_salida - FECHA_FIN_VACACIONES).days
        if dias_margen <= 3:
            estado = "🔴 Crítico"
        elif dias_margen <= 7:
            estado = "🟡 En riesgo"
```

### Reglas

- **SOLO mostrar vacaciones donde `date_from > HOY`** (no mostrar vacaciones en curso ni pasadas)
- Ordenar por fecha de inicio (más próxima primero)
- Si no hay vacaciones futuras: `*✅ Sin vacaciones programadas en los próximos 30 días*`
- Siempre incluir sección "Conflictos críticos" si hay proyectos con 🔴
- Siempre incluir "Recomendaciones" si hay conflictos

---

## Sección 6: Próximas Puestas en Producción

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

Tabla con los proyectos que tienen salida a producción en los próximos 30 días.

```
---

### 🚀 Próximas Puestas en Producción (próximos 30 días)

| Proyecto | Fecha Salida | Responsable | Horas Planif. | Horas Restantes | Estado |
|----------|--------------|-------------|---------------|-----------------|--------|
| Proyecto A | DD/MM/YYYY | Nombre | HH:MM | HH:MM | 🔴 Crítico (vacaciones DD-DD/MM) |
| Proyecto B | DD/MM/YYYY | Nombre | HH:MM | HH:MM | 🟡 En riesgo (vacaciones DD/MM) |
| Proyecto C | DD/MM/YYYY | Nombre | HH:MM | HH:MM | ✅ OK |

**Leyenda de estados:**
- 🔴 **Crítico**: Hay vacaciones del responsable en los 3 días previos a la salida
- 🟡 **En riesgo**: Hay vacaciones del responsable en los 7 días previos a la salida
- ✅ **OK**: Sin conflictos detectados

**⚠️ Alertas de producción:**
1. **[Proyecto] (DD/MM)**: [Descripción del riesgo]
```

### Columnas OBLIGATORIAS

| # | Columna | Formato | Descripción |
|---|---------|---------|-------------|
| 1 | **Proyecto** | Texto | Nombre del proyecto |
| 2 | **Fecha Salida** | DD/MM/YYYY | Fecha de `x_studio_fecha_prevista_salida_produccin` |
| 3 | **Responsable** | Texto | Empleado con más horas planificadas en el proyecto |
| 4 | **Horas Planif.** | HH:MM | Total horas planificadas del responsable |
| 5 | **Horas Restantes** | HH:MM | Horas pendientes (`remaining_hours`) |
| 6 | **Estado** | Emoji + texto | Estado según conflictos con vacaciones |

### Detección de Responsable y Estado

```python
# Para cada proyecto con salida a producción
for proyecto in proyectos_con_salida:
    # Buscar planificaciones de empleados de Desarrollo
    planificaciones = search([
        ["project_id", "=", proyecto.id],
        ["department_id", "=", 4],  # Solo Desarrollo
        ["remaining_hours", ">", 0]
    ])

    # El responsable es el empleado con más horas planificadas
    responsable = max(planificaciones, key=lambda p: p.hours)

    # Verificar si el responsable tiene vacaciones cerca de la salida
    vacaciones = search([
        ["employee_id", "=", responsable.employee_id.id],
        ["date_from", "<=", proyecto.x_studio_fecha_prevista_salida_produccin],
        ["date_to", ">=", proyecto.x_studio_fecha_prevista_salida_produccin - 7 días],
        ["state", "=", "validate"]
    ])

    if vacaciones en 3 días previos:
        estado = "🔴 Crítico"
    elif vacaciones en 7 días previos:
        estado = "🟡 En riesgo"
    else:
        estado = "✅ OK"
```

### Reglas

- Ordenar por fecha de salida (más próxima primero)
- Si no hay proyectos con salida próxima: `*✅ Sin puestas en producción programadas en los próximos 30 días*`
- Siempre incluir "Alertas de producción" si hay proyectos con 🔴 o 🟡
- Excluir proyectos internos (id=1, 100, 279)

---

## Sección 7: Checklist

### ⚠️ FORMATO OBLIGATORIO - NO MODIFICAR

Checklist generado automáticamente con las acciones derivadas del informe.

```
---

### 📝 Checklist para Planificación Semanal

- [ ] Reasignar tareas no completadas de la semana anterior
- [ ] [Acción específica derivada de recomendaciones]
- [ ] [Acción específica derivada de vacaciones]
- [ ] [Otras acciones relevantes]
```

### Reglas para Generar Checklist

1. **Siempre incluir** si hay tareas pendientes: "Reasignar tareas no completadas"
2. **Incluir** una acción por cada empleado en rojo
3. **Incluir** acciones de escalado para proyectos con horas agotadas
4. **Incluir** cobertura para vacaciones si hay ausencias >= 3 días
5. **🔴 URGENTE** para cada conflicto crítico vacaciones-producción
6. **Incluir** revisión de estado para proyectos con salida próxima en riesgo

---

## Proceso de Generación del Informe

```
1. Identificar equipo → obtener team_id con get_teams.sh si es necesario

2. ⚠️ CONFIRMAR FECHAS CON EL USUARIO (OBLIGATORIO):
   a. Calcular fechas por defecto:
      - FECHA_FIN = HOY (incluye hoy, las reuniones suelen ser por la tarde)
      - FECHA_INICIO = lunes de la semana actual (las tareas se planifican con fecha del lunes)
   b. Proponer fechas al usuario con AskUserQuestion
   c. Esperar confirmación o fechas personalizadas

3. Calcular fechas adicionales:
   - Hoy (para vacaciones)
   - Hoy + 30 días (para vacaciones y releases)

4. Obtener datos (ORDEN IMPORTANTE):
   a. Empleados del equipo: get_team_employees.sh <team_id>
      → Obtener employee_ids = [id1, id2, ...]

   b. ⭐ Proyectos del equipo: get_team_projects.sh <team_id>
      → Obtener project_ids = [id1, id2, ...]
      → IMPORTANTE: Necesario ANTES de planificación y releases

   c. Slots pendientes: get_team_pending_slots.sh "[emp_ids]" <lunes_semana>
      → Usa employee_ids del paso 4a (planning.slot no tiene team_id)

   d. Partes de hora: get_team_timesheets.sh <team_id> <fecha_inicio> <fecha_fin>

   e. Vacaciones futuras: get_team_leaves.sh "[emp_ids]" <hoy+1> <hoy+30>
      → Usa employee_ids del paso 4a (hr.leave no tiene team_id)

   f. Planificación de proyectos (FILTRADO POR EQUIPO):
      get_employee_project_planning.sh "[emp_ids]" "[project_ids]" <hoy> <hoy+30>
      → Usar project_ids del paso 4b para filtrar solo proyectos del equipo

   g. Proyectos con salida a producción (FILTRADO POR EQUIPO):
      get_upcoming_releases.sh <team_id> <hoy> <hoy+30>
      → Solo releases de proyectos del equipo

5. Procesar datos con calculate_hours.py para cada empleado

6. Detectar conflictos vacaciones-producción:
   a. Para cada vacación, buscar proyectos con horas planificadas que solapen
      → Solo considerar proyectos del equipo (ya filtrados en paso 4f)
   b. Para cada proyecto con salida próxima, verificar vacaciones del responsable
      → Solo considerar releases del equipo (ya filtrados en paso 4g)
   c. Marcar como 🔴 si vacaciones en 3 días previos a salida
   d. Marcar como 🟡 si vacaciones en 7 días previos a salida

7. Generar cada sección siguiendo el formato EXACTO del template

8. Generar recomendaciones basadas en reglas (incluyendo conflictos)

9. Generar checklist basado en reglas (incluyendo urgentes por conflictos)
```

---

## Triggers del Informe

| Frase del usuario | Acción |
|-------------------|--------|
| "preparar planificación" | Generar informe |
| "informe semanal del equipo X" | Generar informe |
| "¿cómo fue la semana del equipo X?" | Generar informe |
| "tareas pendientes del equipo" | Generar sección 1 |
| "revisar semana anterior" | Generar informe |

---

## Notas Importantes

1. **Fechas por defecto**: Lunes de la semana actual hasta HOY (las tareas se planifican con fecha del lunes)
2. **Siempre confirmar fechas**: NUNCA generar el informe sin confirmación del usuario
3. **Fecha de generación**: Normalmente lunes, pero puede ser cualquier día
3. **Solo Desarrollo**: NUNCA incluir empleados de otros departamentos
4. **Formato de horas**: SIEMPRE HH:MM, nunca decimal
5. **Cálculos**: SIEMPRE usar calculate_hours.py, NUNCA estimar
