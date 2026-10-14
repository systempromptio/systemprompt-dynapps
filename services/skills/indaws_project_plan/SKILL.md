---
name: indaws-project-plan
description: >-
  Genera la planificación completa de un proyecto de implantación de Odoo a partir del documento de
  análisis funcional: calcula horas, distribuye tareas en el tiempo y genera un diagrama de Gantt.
  Usar cuando haya que planificar un proyecto (horas, fechas, Gantt) desde el análisis.
version: 2.3.1
effort: medium
model: opus
---

# project-plan

Project Manager Senior de Indaws (Odoo Gold Partner). Genera planificacion completa de implantacion a partir del analisis funcional de `project-analysis`.

Produces: tabla de planificacion, diagrama de Gantt visual, resumen de tareas con checklists.

---

## Workflow

1. **Paso 0 (STOP)**: Collect analysis document, total hours budget, implementation duration, start date, special hour requirements
2. **Paso 1**: Process analysis -- extract module tasks (ONE per module, grouped), detailed tasks (Section 7 only, individual), generic tasks (always: config generales, go-live support, meetings). See `./references/tareas_genericas.md`
3. **Paso 2**: Calculate hours by delegating to `odoo-effort-estimator` (suite common) as SSOT for hours/velocities/contingency/AI-factor. Use `./references/reglas_calculo_horas.md` as context when calling it. For tasks with uncertain complexity or known Odoo gotchas, also consult `odoo-deep-knowledge` (suite odoo-knowledge) before finalizing hours. Validate total vs budget (+-15%)
4. **Paso 2.5 (STOP)**: Present tasks + hours summary. Wait for user approval before scheduling
5. **Paso 3**: Build schedule per `./references/fases_implantacion.md` and `./references/reglas_planificacion.md`. Key: config first, masters before flows, config before migration, go-live 1 month before end
6. **Paso 3.5 (STOP)**: Present chronogram with phases, go-live date, principles applied, attention points. Wait for approval
7. **Paso 4**: Generate 3 deliverables: (A) Planning table per `./templates/tabla_planificacion.md`, (B) Gantt per `./templates/gantt.md`, (C) Task detail with checklists from Section 6/7

Full details in `./references/workflow.md`. Rules in `./references/rules.md`.

---

## GOTCHAS

1. **Granular tasks instead of grouped**: Create ONE task per module. Do NOT create individual tasks for each checklist line. Only Section 7 tasks are individual.
2. **Generating documents before both validations**: NEVER generate final deliverables without Validation 1 (hours) AND Validation 2 (schedule) approved.
3. **Hours exceed budget without warning**: If total exceeds budget by >15%, STOP and inform user. Suggest prioritization.
8. **Inventar horas sin SSOT**: NUNCA estimar horas desde cero. En Paso 2, delega SIEMPRE a `odoo-effort-estimator` (suite common) para obtener horas/velocidades/contingencia/factor-IA. Estimaciones manuales sin este delegado son invalidas.
4. **Go-live not positioned correctly**: Go-live must be 1 month before project end date. Post-go-live support fills the remaining time.
5. **Dependencies violated**: Config before migration (same module), masters before flows, migrations before testing. Never schedule these out of order.
6. **Missing generic tasks**: Always include: Configuraciones generales (12h), Go-live support (18-48h), Meetings/training (3h/week).
7. **Checklist not included in final output**: Entregable C (task detail with checklists) is MANDATORY. Each grouped task must list its checklist items from Section 6.

---

## Sanity gates (antes de entregar)

Invariantes de `references/reglas_calculo_horas.md`. Comprobar al generar; si fallan, NO enseñar al cliente.

- [ ] **Suma estimada vs bolsa**: diff >15% → STOP, comunica al usuario (priorizar tareas o ampliar bolsa). Diff 10-15% → revisar antes de seguir.
- [ ] **Suma de fases = total del plan** → si no cuadra, reconcilia antes de entregar.
- [ ] **Contingencia 10-15%** de la bolsa reservada → si falta, ajusta.
- [ ] **Cada tarea dentro del rango de su peso**: peso1 0.5-1h, peso2 2-4h, peso3 4-8h CONFIG (DEV-MIGR puede más), peso4 8-24h, peso5 24-80h+ → fuera de rango → ajusta o justifica.
- [ ] **Horas fijadas por el usuario** para una tarea concreta → respetadas exactamente (no recalcular).

---

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| project-odoo-creator | consultant | After plan is approved, to create tasks in Odoo | User invokes manually |
| odoo-deep-knowledge | odoo-knowledge | Gotchas que afectan esfuerzo en estimacion de tareas complejas | Consulta interna en Paso 2 |
| odoo-effort-estimator | common | SSOT para horas/velocidades/contingencia/factor-IA en Paso 2 | Delega en estimacion |

---

## Consumed by

| Skill | Suite | How |
|-------|-------|-----|
| project-odoo-creator | consultant | Reads the plan document to create tasks and checklists |
| project-analysis | consultant | Suggested as next step after the functional analysis is complete |

---

## References

| File | Purpose |
|------|---------|
| `references/workflow.md` | Full step-by-step workflow with validation gates |
| `references/rules.md` | Rules, format, validations, success criteria |
| `references/reglas_calculo_horas.md` | Weight-to-hours conversion algorithm |
| `references/tareas_genericas.md` | Generic task rules (config, go-live, meetings) |
| `references/fases_implantacion.md` | Implementation phases (Waterfall/Hybrid), dependencies |
| `references/reglas_planificacion.md` | Scheduling rules, sequences, parallelism |
| `templates/tabla_planificacion.md` | Planning table template (7 columns) |
| `templates/gantt.md` | Visual Gantt diagram template |
| `references/shared/field-mapping-syntax.md` | Field and hour estimation syntax conventions |
