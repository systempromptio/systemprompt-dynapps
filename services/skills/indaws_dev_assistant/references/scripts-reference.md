# Scripts Reference

**SIEMPRE usar estos scripts para consultas a Odoo. NUNCA llamadas directas a la API.**

Los scripts incluyen: filtro `department_id=4`, contexto `es_ES`, exclusiones CTOs, paginación automática.

## Base Scripts

### odoo_call.sh
Llamada genérica. Auto-detecta JSON-RPC (<19.0) o JSON2 (>=19.0).
```bash
./scripts/odoo_call.sh <model> <method> <domain> <fields> [kwargs]
```

### odoo_search_all.sh
Búsqueda paginada (TODOS los registros, 100 en 100).
```bash
./scripts/odoo_search_all.sh <model> <domain> <fields> [order]
```

### calculate_hours.py
**USAR SIEMPRE para cálculos. NUNCA calcular manualmente.**
```bash
./scripts/get_timesheets.sh 175 2026-01-01 2026-01-31 2>/dev/null | python3 scripts/calculate_hours.py
```
Output: JSON con total, billable, unbillable, bonus, pct_billable, validation, formato HH:MM.

## Employee & Timesheets

| Script | Uso | Ejemplo |
|---|---|---|
| `get_employee.sh` | Buscar por nombre/email | `./scripts/get_employee.sh "David"` |
| `get_timesheets.sh` | Partes de un empleado | `./scripts/get_timesheets.sh 175 2026-01-01 2026-01-31` |
| `get_leaves.sh` | Ausencias validadas | `./scripts/get_leaves.sh 175 2026-01-01 2026-01-31` |
| `get_project_allocations.sh` | Horas asignadas | `./scripts/get_project_allocations.sh "[925,997]"` |

## Team Scripts

| Script | Uso | Ejemplo |
|---|---|---|
| `get_teams.sh` | Buscar equipo por nombre | `./scripts/get_teams.sh Tokyo` → id=28 |
| `get_team_employees.sh` | Empleados del equipo | `./scripts/get_team_employees.sh 28` |
| `get_team_timesheets.sh` | Partes del equipo | `./scripts/get_team_timesheets.sh 33 2026-01-27 2026-02-02` |
| `get_team_leaves.sh` | Vacaciones equipo | `./scripts/get_team_leaves.sh 33 2026-02-01 2026-03-01` |
| `get_team_projects.sh` | Proyectos del equipo | `./scripts/get_team_projects.sh 28` |
| `get_team_pending_slots.sh` | Slots NO completados | `./scripts/get_team_pending_slots.sh 33 2026-01-27` |

**Equipos conocidos:** Tokyo(28), Roma(29), Paris(30), Programación(31), Servicios(32), Mónaco(33)

**Si usuario da nombre, buscar ID primero:** `get_teams.sh Tokyo` → id=28 → `get_team_employees.sh 28`

## Planning & Projects

| Script | Uso | Ejemplo |
|---|---|---|
| `get_planning_slots.sh` | Planificación empleado | `./scripts/get_planning_slots.sh 175 2026-01-27 [true]` |
| `get_employee_project_planning.sh` | Planif. horas emp-proy | `./scripts/get_employee_project_planning.sh "[51,183]" "[100]" 2026-02-05 2026-03-07` |
| `get_upcoming_releases.sh` | Salida producción | `./scripts/get_upcoming_releases.sh 28 2026-02-05 2026-03-07` |
| `get_project_timesheets.sh` | Partes de proyecto | `./scripts/get_project_timesheets.sh 997 2026-01-01 2026-01-31` |
| `get_task_details.sh` | Detalles tarea | `./scripts/get_task_details.sh 1234 [true]` |
