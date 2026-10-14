---
name: indaws-consultant-daily-planner
description: >-
  Genera la agenda diaria priorizada del consultor analizando proyectos activos, tareas, peticiones
  al cliente, tickets y reuniones. Propone acciones concretas con skill e IDs para ejecutar
  directamente. Usar cuando el consultor pida su agenda diaria o el plan del día.
version: 1.2.1
effort: medium
model: sonnet
allowed-tools: [Read, Write, Bash, Glob, Grep]
---

# Consultant Daily Planner

Coordinador de Agenda de Indaws (Odoo Gold Partner). Punto de entrada diario del consultor -- analiza todos los proyectos activos cada manana y genera una agenda priorizada con acciones concretas, indicando que skill ejecutar, con que parametros y por que.

## Workflow

Two modes: **(A) Agenda completa** (todos los proyectos) o **(B) Proyecto concreto** (un solo proyecto).

### Paso 0: Recopilacion de Datos

1. **Credenciales Odoo**: URL, DB, API Key, Usuario
2. **Modo**: A (default) o B (requiere project_id)
3. **Empleado**: Automatico desde ODOO_USER via `hr.employee`. Solo pedir si es otro empleado

See references/shared/odoo-connection.md for connection pattern. DETENTE y espera respuesta.

### Paso 1: Extraccion de Datos

Per project, extract 7 data sets via odoo-pilot (task, checklists, client requests, project updates, helpdesk tickets, Knowledge articles). Verify Knowledge structure against `references/estructura_knowledge.md`.

Full extraction details in `references/workflow_detail.md`.

### Paso 2: Analisis y Priorizacion

Classify actions in 3 levels using `references/reglas_priorizacion.md`:

| Level | Examples |
|-------|---------|
| URGENTE | Requests vencidos >3d, tareas con deadline hoy, tickets >24h, viernes (weekly updates), reunion sin preparar |
| IMPORTANTE | Health score <60, weekly >7d antiguo, requests con respuesta sin procesar, formacion proxima |
| PLANIFICADO | Validaciones pendientes, checklists esta semana, go-live <30d, Knowledge incompleto |

Apply cross-context rules (health <40 sube todo un nivel, go-live <14d idem) and day-of-week patterns from `references/patrones_dia.md`.

### Paso 3: Generacion de Agenda

Generate agenda using `templates/agenda_diaria.md`. Each action includes: proyecto, titulo, tiempo estimado, contexto, IDs, skill a ejecutar, prerequisitos.

No interactive validation -- agenda is informational. Ask: "Quieres que ejecute alguna accion?"

Full details in `references/workflow_detail.md`. Rules in `references/rules_and_validations.md`.

## GOTCHAS

1. **Ejecutar skills automaticamente** -- NUNCA ejecutar un skill sin que el consultor lo pida. Solo proponer con IDs concretos
2. **Enviar draft requests de tareas no empezadas** -- Solo proponer envio si `planned_date_begin <= hoy`. Si la tarea no ha empezado, el request draft NO se incluye
3. **Ignorar dia de la semana** -- Consultar `references/patrones_dia.md` siempre. Viernes = weekly updates obligatorios
4. **Mezclar datos entre proyectos** -- En modo B, NUNCA mostrar datos internos de otros proyectos
5. **Obtener empleado manualmente** -- Siempre auto-resolver desde ODOO_USER via hr.employee. Solo pedir si el usuario quiere analizar otro empleado
6. **Olvidar verificar Knowledge** -- SIEMPRE verificar estructura Knowledge de cada proyecto contra la referencia
7. **Tickets con stage cerrado** -- Solo considerar tickets con `fold=false` (stage abierto). Ignorar cerrados

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | Extraccion de datos de 7 modelos por proyecto | Bash inline |

## Consumed by

| Skill | Suite | Purpose |
|-------|-------|---------|
| claude-onboarding-dynapps | common | Onboarding: ritual matutino del consultor nuevo (delegación opcional) |

## References

| File | Content |
|------|---------|
| `references/workflow_detail.md` | Full extraction, analysis and agenda generation steps |
| `references/rules_and_validations.md` | Rules for prioritization, client requests, meetings, tickets |
| `references/reglas_priorizacion.md` | 3-level classification rules and cross-context rules |
| `references/patrones_dia.md` | Day-of-week specific patterns |
| `references/estructura_knowledge.md` | Expected Knowledge article structure per project |
| `references/vinculacion_skills.md` | Skill dependencies and execution order |
| `templates/agenda_diaria.md` | Template for daily agenda output |
| `scripts/fetch_consultant_agenda.sh` | Main data extraction script |
