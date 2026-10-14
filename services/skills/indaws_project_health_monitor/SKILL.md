---
name: indaws-project-health-monitor
description: >-
  Evalúa el estado de salud de un proyecto Odoo analizando timeline, checklists ponderados y consumo
  de horas. Genera informe con Health Score y plan de acción. Usar cuando pidan evaluar la salud de
  un proyecto o su Health Score.
version: 1.2.0
effort: medium
model: sonnet
allowed-tools: [Read, Write, Bash, Glob]
---

# project-health-monitor

Project Health Analyst de Indaws (Odoo Gold Partner). Conecta a Odoo, extrae datos de un proyecto y evalua su salud comparando avance temporal con progreso real (checklists ponderados + consumo de horas).

Produce informe con Health Score (0-100), bottlenecks y plan de accion priorizado.

---

## Workflow

1. **Paso 0 (STOP)**: Collect Odoo credentials, project ID, optional reference date
2. **Paso 1**: Run `./scripts/fetch_project_data.sh` to extract 6 data sets (project, tasks, checklists, timesheets, hours, departments). Validate post-extraction
3. **Paso 2**: Run `./scripts/calculate_health.py` (MANDATORY -- NEVER calculate manually). Output JSON contains all metrics, penalties, score, scenario, bottlenecks, alerts
4. **Paso 2.5 (STOP)**: Present dashboard (Health Score, T%/C%/H%, scenario, burn rate, bottlenecks). Wait for user validation before generating report
5. **Paso 3**: Evaluate scenario (ON_TRACK, LOW_PRODUCTIVITY, INEFFICIENCY, BLOCKED, OVERCOST, OVERDUE) and generate prioritized action plan per `./references/plan_accion_patrones.md`
6. **Paso 4**: Generate full report using `./templates/informe_salud.md` (8 sections). Only after dashboard validation
7. **Paso 4.5** (optional): Publish the result as an INTERNAL NOTE in the `project.project` chatter via `./scripts/health_note.py` (dry-run first, `--confirm` only after explicit OK). Never writes to `project.update`

Full details in `./references/workflow.md`. Rules in `./references/rules.md`.

See references/shared/odoo-connection.md for connection pattern.

---

## GOTCHAS

1. **Manual metric calculation**: NEVER calculate metrics outside `calculate_health.py`. Use JSON values directly.
2. **Report before dashboard validation**: NEVER generate the full report without the user approving the dashboard first (Paso 2.5).
3. **Missing checklists**: No checklists = -15 point governance penalty. Inform the user, but continue with partial metrics.
4. **No project dates**: Without start/end dates, the analysis CANNOT proceed. This is a hard ERROR.
5. **Weight null or 0**: Treat checklist items with null or 0 weight as weight = 1.
6. **Hours discrepancy**: If task hours vs timesheet hours differ, use timesheets as primary source.
7. **HTML for Odoo**: Odoo does not process external CSS. All styles must be inline. `health_note.py build` already produces it.
8. **Health fields on project.update are gone**: the 16 `x_studio_*` health fields (`x_studio_health_score`, `x_studio_internal_report`, `x_studio_last_health_check`...) were deleted in portal.dynapps.es; writing them fails with "Invalid field". NEVER read or write them. Comparison with the previous evaluation comes from `health_note.py previous` (local snapshot or last `PHM-DATA` note in the chatter).
9. **Internal note, zero emails**: always `health_note.py post` (subtype `mail.mt_note`, no `partner_ids`, no followers added, no template). Never a hand-made `message_post` and never `mail.mt_comment`: that one emails the client followers.

---

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | Fetch project data | Bash inline |

---

## Consumed by

| Skill | Suite | How |
|-------|-------|-----|
| project-health-fixer | consultant | Reads metrics JSON + raw data JSON as input |
| project-weekly-update | consultant | Runs it automatically as Step 1 of every weekly update; its `create_update.py` imports `health_note.py` to post the internal note (do not also run Paso 4.5) |

---

## References

| File | Purpose |
|------|---------|
| `references/workflow.md` | Full step-by-step workflow with STOP points and data model |
| `references/rules.md` | Rules, validations, format guidelines, success criteria |
| `references/metricas_salud.md` | Algorithm documentation (reference only, not for manual calculation) |
| `references/plan_accion_patrones.md` | Action plan patterns by scenario |
| `templates/informe_salud.md` | Health report template (8 sections) |
| `scripts/fetch_project_data.sh` | Data extraction script (6 data sets from Odoo) |
| `scripts/calculate_health.py` | Metric calculation script (Health Score, penalties, bottlenecks) |
| `scripts/health_note.py` | Internal chatter note: build HTML, previous evaluation, post (dry-run by default), local snapshot |
