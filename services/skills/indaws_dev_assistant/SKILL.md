---
name: indaws-dev-assistant
description: Asistente personal del desarrollador para gestionar su día a día en Odoo corporativo. Usa este skill cuando el usuario pregunte sobre su planificación semanal ("¿Qué tengo que hacer?", "¿Qué tareas tengo?", "¿Qué hago hoy?"), partes de horas ("¿Cómo voy de horas?", "informe de horas", "horas facturables/no facturables"), o información sobre tareas asignadas ("¿De qué va esta tarea?", "¿Qué tarea estoy haciendo?"). Solicita las credenciales de conexión al usuario si no están configuradas.
version: 1.0.2
effort: low
model: sonnet
---

# Dev Assistant — Asistente del Desarrollador

Consulta información del desarrollador en el Odoo corporativo de Indaws.

## References (read on demand)

| Ref | When to read |
|-----|-------------|
| [references/credentials-and-roles.md](references/credentials-and-roles.md) | **First call** — Credential flow + role detection + welcome messages |
| [references/scripts-reference.md](references/scripts-reference.md) | **Always** — All scripts with usage + examples |
| [references/report-rules.md](references/report-rules.md) | **Before any report** — Report protocol, features, billable rules |
| [references/models.md](references/models.md) | **Before any query** — All Odoo models, fields, filters, relations |
| [references/timesheets.md](references/timesheets.md) | When generating timesheet reports |
| [references/planning.md](references/planning.md) | When generating planning reports |
| [references/projects.md](references/projects.md) | When generating project reports |
| [references/tasks.md](references/tasks.md) | When querying task details |

## Critical Rules

### SOLO Departamento de Desarrollo
- Filtro obligatorio: `[["department_id", "=", 4]]`
- Excluir CTOs: David Gómez Quilón (194), Jesús Vargas Sánchez (193)
- Excluir empleados con `effective_hours_pct = 0`

### Idioma es_ES
TODAS las llamadas API: `"context": {"lang": "es_ES"}`. Los scripts lo incluyen automáticamente.

### Scripts obligatorios
NUNCA hacer llamadas directas a la API. SIEMPRE usar scripts de `scripts/`. Ver [scripts-reference.md](references/scripts-reference.md).

### Cálculos de horas
SIEMPRE usar `calculate_hours.py`. NUNCA calcular manualmente.

## Quick Start

1. Read [credentials-and-roles.md](references/credentials-and-roles.md) → solicitar credenciales + detectar rol
2. Read [report-rules.md](references/report-rules.md) → identificar tipo de informe
3. Read template correspondiente en `templates/`
4. Read [scripts-reference.md](references/scripts-reference.md) → ejecutar scripts
5. Generar informe siguiendo EXACTAMENTE el template

## Available Reports

| Report | Template | Trigger |
|--------|----------|---------|
| Horas individual | `templates/timesheet-individual.md` | "mis horas", "¿cómo voy?" |
| Horas equipo | `templates/timesheet-team.md` | "informe equipo X" |
| Prep. planificación | `templates/weekly-planning-prep.md` | "preparar planificación" |
| Horas proyecto | `templates/project-report.md` | "informe proyecto #X" |
| Planificación | `templates/planning.md` | "¿qué tengo que hacer?" |
| Info tarea | `templates/task-info.md` | "¿de qué va esta tarea?" |

## Prohibitions

- NUNCA inventar modelos o campos (si no está en `models.md`, no existe)
- NUNCA generar informe sin leer el template primero
- NUNCA incluir empleados fuera de Desarrollo
- NUNCA calcular horas manualmente (usar calculate_hours.py)
- NUNCA generar informe no definido sin confirmación del usuario

See references/shared/odoo-connection.md for connection pattern and references/shared/batch-limits.md for pagination.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Fetch tasks, timesheets from Odoo |

## Consumed by

None — end-user skill for developers.

## GOTCHAS
1. **Hours must be HH:MM, never decimals** — 1.5h must display as 1:30.
2. **Exclude holiday_id and global_leave_id** from timesheet queries — they're not worked hours.
3. **x_studio_team_id is Many2one** — filter by ID, not name string.
4. **project.project.hour.line has no date_from/date_to** — use the `date` field.
5. **Exclude CTOs from reports** — employee IDs 193, 194.
