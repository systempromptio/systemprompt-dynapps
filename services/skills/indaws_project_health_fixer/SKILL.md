---
name: indaws-project-health-fixer
description: >-
  Ayuda al consultor a corregir problemas del proyecto Odoo detectados por project-health-monitor:
  corrige gobernanza, propone plan de acción y planifica desviaciones. Usar cuando haya que corregir
  problemas del proyecto detectados por el health monitor.
version: 1.1.1
effort: medium
model: opus
allowed-tools: [Read, Write, Bash, Glob]
---

# project-health-fixer

Project Health Fixer de Indaws (Odoo Gold Partner). Lee el diagnostico de `project-health-monitor`, clasifica problemas, propone correcciones concretas y ejecuta las aprobadas en Odoo via odoo-pilot.

Principio clave: El script clasifica y cuantifica. Tu propones, presentas al consultor y ejecutas lo aprobado. Todo cambio en Odoo requiere aprobacion explicita.

Prerequisito obligatorio: Haber ejecutado `project-health-monitor` previamente. Este skill lee su JSON de metricas, nunca recalcula.

---

## Workflow

Two scopes: "Solo gobernanza" (quick, Steps 2-3 only) or "Revision completa" (governance + progress + deviation).

1. **Paso 0 (STOP)**: Collect credentials, project ID, JSON paths (metrics + raw data), optional analysis doc, scope
2. **Paso 1**: Run `diagnose_issues.py` to classify issues into 3 categories (governance, progress, strategic)
3. **Paso 1.5 (STOP)**: Present diagnosis summary with score actual vs potential, escenario, problem tables
4. **Paso 2**: Propose governance corrections (deadlines, hours, checklists) using `fix_patterns.md`
5. **Paso 2.5 (STOP)**: Present correction batch for approval
6. **Paso 3**: Execute approved corrections via odoo-pilot (one by one, continue on failure)
7. **Paso 3.5 (STOP)**: Report execution results
8. **Pasos 4-5** (full scope only): Plan + execute progress actions (overdue tasks, bottlenecks)
9. **Paso 6** (conditional): Deviation evaluation if Score < 40, acceleration > 2.0, or T% > 100 with C% < 80
10. **Paso 7**: Generate correction report using `./templates/fix_report.md`

Full details in `./references/workflow.md`. Rules in `./references/rules.md`.

See references/shared/odoo-connection.md for connection pattern.

---

## GOTCHAS

1. **Recalculating metrics manually**: NEVER recalculate. Always use values from the `calculate_health.py` JSON output.
2. **Inventing task/item IDs**: NEVER invent IDs. Always use IDs from the raw data JSON.
3. **Executing without approval**: NEVER execute Odoo changes without explicit consultant approval at each STOP point.
4. **Reducing allocated_hours below effective_hours**: When redistributing hours, never reduce a task's allocated hours if it already has effective_hours > allocated_hours.
5. **Date format mismatch**: Present DD/MM/YYYY to the consultant, but send YYYY-MM-DD to Odoo. Mixing these up causes silent failures.
6. **Missing JSON files**: If JSONs are not available, look for the diagnosis in the most recent `project.update` record (`x_studio_internal_report`), then re-extract data.
7. **Skipping semantic comparison**: When analysis doc is provided, always do the semantic comparison (analysis vs Odoo tasks) -- it reveals scope creep and missing tasks.

---

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | Read/write project data in Odoo | Bash inline |
| project-health-monitor | consultant | Re-extract data if JSONs are stale | User runs separately |

---

## Consumed by

See DEPENDENCIES.md

---

## References

| File | Purpose |
|------|---------|
| `references/workflow.md` | Full step-by-step workflow with all STOP points |
| `references/rules.md` | Rules, validations, correction guidelines, success criteria |
| `references/fix_patterns.md` | Fix patterns: governance (sec 1), progress (sec 2), deviation formulas (sec 3) |
| `templates/fix_report.md` | Correction report template (6 sections) |
| `scripts/diagnose_issues.py` | Diagnosis script (classifies issues from metrics + raw data) |
