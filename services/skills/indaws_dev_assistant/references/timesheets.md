# Partes de Horas

Documentación técnica para consultas de partes de horas (individual y equipo).

## Modelos Odoo

### account.analytic.line (Partes de hora)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `date` | Date | Fecha del parte |
| `employee_id` | Many2one | Empleado |
| `project_id` | Many2one | Proyecto |
| `task_id` | Many2one | Tarea |
| `unit_amount` | Float | Horas imputadas (decimal) |
| `name` | Char | Descripción del trabajo |
| `billable_unit_amount` | Float | Horas facturables |
| `unbillable_unit_amount` | Float | Horas NO facturables |
| `x_studio_bonus_unit_amount` | Float | Horas en tiempo (para bonus). Horas facturables imputadas a tiempo. Si se imputa tarde, se aplica penalización 0.7x |

### project.project.hour.line (Asignación de horas)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `project_id` | Many2one | Proyecto |
| `department_id` | Many2one | Departamento (Desarrollo = 4, false = global) |
| `hours` | Float | Horas asignadas |
| `total_billable_hours` | Float | Horas facturables consumidas |
| `total_unbillable_hours` | Float | Horas no facturables consumidas (exceso histórico) |
| `total_hours_available` | Float | Horas disponibles (si <= 0, proyecto agotado) |

### hr.employee (Empleado)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | Integer | ID del empleado |
| `name` | Char | Nombre |
| `department_id` | Many2one | Departamento |
| `x_studio_team_id` | Many2one → `crm.team` | Equipo |
| `effective_hours_pct` | Float | Objetivo horas facturables (0-1) |
| `resource_calendar_id` | Many2one | Calendario laboral |

### crm.team (Equipos)

El campo `hr.employee.x_studio_team_id` es un Many2one a `crm.team`.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | Integer | ID del equipo |
| `name` | Char | Nombre del equipo |
| `active` | Boolean | Si está activo |

**Equipos conocidos:**

| ID | Nombre |
|----|--------|
| 1 | Indaws |
| 24 | Marketing |
| 28 | Tokyo |
| 29 | Roma |
| 30 | Paris |
| 31 | Programación |
| 32 | Servicios |
| 33 | Mónaco |

**Buscar equipo por nombre:**
```bash
./scripts/get_teams.sh Tokyo    # Devuelve [{"id": 28, "name": "Tokyo"}]
./scripts/get_teams.sh          # Lista todos los equipos
```

### hr.leave (Ausencias)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `employee_id` | Many2one | Empleado |
| `date_from` | Datetime | Inicio ausencia |
| `date_to` | Datetime | Fin ausencia |
| `number_of_days` | Float | Días de ausencia |
| `holiday_status_id` | Many2one | Tipo de ausencia |
| `state` | Selection | Estado ('validate' = aprobada) |

## Cálculos

### ⛔ REGLAS DE CÁLCULO - NO ESTIMAR NUNCA

**CRÍTICO: Los errores de cálculo son INACEPTABLES.**

```
Total horas     = sum(unit_amount)                  de todos los registros
Horas fact.     = sum(billable_unit_amount)         de todos los registros
Horas no fact.  = sum(unbillable_unit_amount)       de todos los registros
Horas en tiempo = sum(x_studio_bonus_unit_amount)   de todos los registros
```

**Horas en tiempo (bonus):**
- Son las horas facturables imputadas dentro del plazo
- Si el parte se imputa tarde, se aplica penalización 0.7x
- Usadas para calcular el bonus trimestral del empleado
- `% en tiempo = (horas_en_tiempo / horas_facturables) × 100`

**Reglas:**
1. NUNCA estimar visualmente - SIEMPRE sumar los campos del JSON
2. NUNCA redondear arbitrariamente
3. NUNCA inventar datos

**Validación obligatoria:**
```
billable + unbillable ≈ total (tolerancia ±0.5h)
```

Si no cuadra, HAY UN ERROR. Revisar cálculos.

**Script recomendado:**
```bash
./scripts/get_timesheets.sh <emp_id> <from> <to> 2>/dev/null | python3 scripts/calculate_hours.py
```

### Formato de Horas

**OBLIGATORIO**: Siempre mostrar horas en formato **HH:MM**, nunca en decimales.

```
horas = parte_entera
minutos = (parte_decimal * 60) redondeado
Ejemplo: 2.75h → 02:45
```

### Semana Laboral

- **Inicio**: Lunes
- **Fin**: Domingo
- **Días laborables**: Lunes a Viernes (5 días)
- **Horas estándar**: 08:00 por día laborable

```
HOY = fecha actual
DIA_SEMANA = día de la semana (1=lunes, 7=domingo)
LUNES = HOY - (DIA_SEMANA - 1) días
DOMINGO = LUNES + 6 días
```

### Periodos en Curso

Si el periodo solicitado incluye HOY:
- **FECHA_FIN = HOY**, no el final teórico del periodo
- Días laborables = solo L-V desde FECHA_INICIO hasta HOY
- Horas esperadas = días laborables × 8

### Horas Laborables

```
dias_laborables = días de lunes a hoy (solo L-V)
horas_ausencia = sum(number_of_days) * 8 de ausencias validadas
horas_laborables = (dias_laborables * 8) - horas_ausencia
```

### Balance de Imputación

```
horas_imputadas = sum(unit_amount) de partes hasta hoy
balance = horas_imputadas - horas_laborables
Si balance < 0 → faltan horas por imputar
Si balance > 0 → adelantado (trabajo extra, fin de semana)
```

### Porcentajes de Horas Facturables

```
total_facturables = sum(billable_unit_amount)
total_no_facturables = sum(unbillable_unit_amount)
pct_facturables = (total_facturables / total_imputadas) * 100
```

## Indicadores de Alarma

| Icono | Nivel | Cuándo usar |
|-------|-------|-------------|
| 🔴 | **CRÍTICO** | Horas sin imputar, proyecto 100% no facturable, balance muy negativo |
| 🟡 | **AVISO** | Proyecto interno, % no facturable > 15%, día con < 6h |
| ✅ | **OK** | Sin problemas, balance positivo |

### Criterios de Alarma (vs Objetivo Individual)

El objetivo de horas facturables es **individual por empleado** (`effective_hours_pct` de `hr.employee`, valor 0-1).

| % Facturable vs Objetivo | Estado |
|--------------------------|--------|
| >= objetivo | ✅ OK |
| objetivo - 10% a objetivo | 🟡 AVISO |
| < objetivo - 10% | 🔴 CRÍTICO |

**Ejemplo**: Si `effective_hours_pct = 0.75` (objetivo 75%):
- >= 75% = ✅ OK
- 65-74% = 🟡 AVISO
- < 65% = 🔴 CRÍTICO

**Ejemplo**: Si `effective_hours_pct = 0.60` (objetivo 60%):
- >= 60% = ✅ OK
- 50-59% = 🟡 AVISO
- < 50% = 🔴 CRÍTICO

**Interno > 25%**: Se marca con aviso adicional, pero NO cambia el estado general.

## Selección de Línea de Asignación

Para determinar las horas asignadas de un proyecto:

1. Buscar líneas en `project.project.hour.line` con `department_id = 4` (Desarrollo)
2. Si no existe, buscar líneas con `department_id = false` (global)
3. Filtrar por fecha: `date_from <= fecha_fin_periodo` AND (`date_to >= fecha_inicio_periodo` OR `date_to = false`)

**Casos posibles:**
1. **Proyecto con `total_hours_available = 0`**: Horas agotadas
2. **Proyecto NO encontrado**: No tiene horas asignadas (error de consultoría)
3. **Proyecto con `total_hours_available > 0`**: Aún hay margen

---

## Informe Individual

**Script**: `scripts/get_timesheets.sh`
**Template**: `templates/timesheet-individual.md`

### Consultas necesarias

1. **Empleado** (scripts/get_employee.sh)
2. **Partes de hora** (scripts/get_timesheets.sh)
3. **Ausencias** (scripts/get_leaves.sh)
4. **Asignaciones de proyecto** (scripts/get_project_allocations.sh)

### Exclusiones en partes de hora

Excluir registros con:
- `holiday_id != false` (partes automáticos de vacaciones)
- `global_leave_id != false` (partes automáticos de festivos)

---

## Informe de Equipo/Departamento

**Script**: `scripts/get_team_employees.sh`
**Template**: `templates/timesheet-team.md`

### ⚠️ RESTRICCIÓN CRÍTICA

**SOLO empleados del departamento de Desarrollo (`department_id = 4`).**

NUNCA incluir empleados de otros departamentos en los informes de equipo, aunque pertenezcan al mismo equipo en Odoo.

### Tipos de Informe

| Tipo | Descripción | Quién lo pide |
|------|-------------|---------------|
| **Por equipo** | Empleados de Desarrollo de un equipo específico | Team Leader / CTO |
| **Departamento completo** | Todos los empleados de Desarrollo | CTO |

### Filtros obligatorios

**Siempre aplicar:**
- `[["department_id", "=", 4]]` (SOLO Desarrollo)
- `[["x_studio_team_id", "in", [IDS_EQUIPOS]]]` (equipos solicitados)
- `[["active", "=", true]]`
- `[["effective_hours_pct", ">", 0]]`

### ⚠️ USO OBLIGATORIO DE SCRIPTS

**SIEMPRE usar los scripts de `./scripts/` para obtener datos. NUNCA hacer llamadas directas a la API.**

| Script | Uso |
|--------|-----|
| `scripts/get_team_employees.sh 33` | Empleados de un equipo |
| `scripts/get_team_employees.sh "[33,34]"` | Empleados de varios equipos |
| `scripts/get_team_employees.sh` | Todos los empleados de Desarrollo |
| `scripts/get_timesheets.sh <emp_id> <from> <to>` | Partes de hora |
| `scripts/get_leaves.sh <emp_id> <from> <to>` | Ausencias |
| `scripts/get_project_allocations.sh "[ids]"` | Asignaciones de proyectos |

### Exclusiones de empleados

Siempre excluir:
- CTOs: David Gómez Quilón (id=194), Jesús Vargas Sánchez (id=193)
- Empleados con `effective_hours_pct = 0`
- Empleados de otros departamentos (NO Desarrollo)

### Criterios de Estado por Empleado

Comparar **% facturable real** vs **objetivo individual** (`effective_hours_pct`):

| % Facturable vs Objetivo | Estado |
|--------------------------|--------|
| >= objetivo | ✅ OK |
| objetivo - 10% a objetivo | 🟡 AVISO |
| < objetivo - 10% | 🔴 CRÍTICO |

**Ejemplo**: Si objetivo = 75%:
- >= 75% = ✅ OK
- 65-74% = 🟡 AVISO
- < 65% = 🔴 CRÍTICO

### Tabla de Proyectos con Horas No Facturables

**IMPORTANTE**: Mostrar TODOS los proyectos donde los empleados de Desarrollo han imputado horas NO FACTURABLES en el periodo.

**⚠️ SOLO contar horas de empleados del departamento de Desarrollo (`department_id = 4`).**

**Proceso de generación:**

1. Obtener lista de empleados que pertenecen a Desarrollo (`department_id = 4`)
2. Obtener partes de hora SOLO de esos empleados en el periodo
3. Agrupar por proyecto y sumar `unbillable_unit_amount`
4. Filtrar proyectos con horas no facturables > 0
5. Para cada proyecto, buscar su línea de asignación en `project.project.hour.line`:
   - Primero buscar con `department_id = 4` (Desarrollo)
   - Si no existe, usar línea con `department_id = false` (global)
   - Si no existe ninguna → "Sin asignación"
6. Calcular estado según % de exceso histórico:
   - Sin asignación → 🔴
   - Exceso > 30% → 🔴
   - Exceso 1-30% → 🟡
   - Exceso <= 0% → ✅
7. **Orden**: primero "Sin asignación", luego por % exceso descendente
8. **Excluir**: Proyecto Interno (id=1), INDAWS (id=100), INDAWS FORMACIONES (id=279)

### Informe de Departamento Completo

Agrupar empleados por equipo con subtotales por equipo.

---

## Notas

- Consultoría asigna horas por departamento, pero no siempre actualiza
- Desarrollo suele necesitar más horas de las asignadas
- `total_hours_available <= 0` indica proyecto sin margen
- Fines de semana: si hay horas imputadas, mostrar como trabajo extra
- Festivos: verificar en hr.leave o calendario de la empresa
- Proyecto "Interno": 🟡 si <= 25% del total, 🔴 si > 25%
