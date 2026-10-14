# Report Generation Rules & Features

## Report Protocol

| Tipo de Informe | Template | Triggers | Rol |
|---|---|---|---|
| Horas individual | `templates/timesheet-individual.md` | "mis horas", "¿cómo voy?" | Todos |
| Horas equipo | `templates/timesheet-team.md` | "informe equipo X" | TL/CTO |
| Preparación planificación | `templates/weekly-planning-prep.md` | "preparar planificación" | TL |
| Horas proyecto | `templates/project-report.md` | "informe proyecto #X" | TL/CTO |
| Planificación | `templates/planning.md` | "¿qué tengo que hacer?" | Todos |
| Info de tarea | `templates/task-info.md` | "¿de qué va esta tarea?" | Todos |

### Reglas OBLIGATORIAS
1. **ANTES** de generar cualquier informe, **LEER el template** con Read
2. Seguir **EXACTAMENTE** el formato (columnas, orden, indicadores)
3. **NO inventar** columnas/secciones. **NO omitir** columnas del template

### Informe NO definido
1. NO generar nada
2. PREGUNTAR al usuario: elegir uno disponible o informe libre (advertir sin formato garantizado)
3. NUNCA generar informe inventado sin confirmación

### Proceso
1. Identificar tipo → consultar tabla
2. LEER template con Read
3. Obtener datos con scripts
4. Generar siguiendo EXACTAMENTE el template

---

## Feature Details

### 1. Planificación Semanal
- Scripts: `get_planning_slots.sh`
- Ref: `references/planning.md` | Template: `templates/planning.md`

### 2. Partes Horas Individual
- Scripts: `get_employee.sh`, `get_timesheets.sh`, `get_leaves.sh`, `get_project_allocations.sh`
- Ref: `references/timesheets.md` | Template: `templates/timesheet-individual.md`
- **Objetivo facturable es individual** (campo `effective_hours_pct` en `hr.employee`)

### 3. Informe Equipo/Departamento
- Scripts: `get_team_employees.sh`, `get_timesheets.sh`, `get_leaves.sh`
- Template: `templates/timesheet-team.md`
- Tipos: mi equipo / equipo específico / departamento completo

### 4. Preparación Planificación (TL)
- Scripts: `get_team_pending_slots.sh`, `get_team_timesheets.sh`, `get_team_leaves.sh`, `get_employee_project_planning.sh`, `get_upcoming_releases.sh`
- Template: `templates/weekly-planning-prep.md`
- Contenido: tareas no completadas, horas facturable/no, vacaciones+conflictos, releases, checklist

### 5. Informe Proyecto
- Scripts: `get_project_timesheets.sh`, `get_team_employees.sh`, `get_project_allocations.sh`
- Template: `templates/project-report.md`

### 6. Información Tarea
- Script: `get_task_details.sh`
- Template: `templates/task-info.md`

---

## Billable Hours Rules

### Objetivo Individual
`effective_hours_pct` en `hr.employee` (0-1, ej: 0.75 = 75%)

### Estado
| % Facturable vs Objetivo | Estado |
|---|---|
| >= objetivo | OK |
| objetivo-10% a objetivo | AVISO |
| < objetivo-10% | CRITICO |

### Periodo en Curso
Si periodo incluye hoy: fecha_fin = HOY (no fin teórico del periodo).
Días laborables y horas esperadas solo hasta HOY.
